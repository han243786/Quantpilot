use anyhow::Result;
use qrpc_core::CoreStrategyIr;
use qrpc_core::{
    DataKind, DataSourceConfig, Exchange, NormalizedMarketData, RuntimeEvent, SourceStatus,
};
use reqwest::Client;

use crate::circuit_breaker::CircuitBreaker;
use std::{collections::BTreeMap, sync::Mutex, time::Duration};

mod collection_orchestration;
mod exchange_surface;
mod historical_cache;
mod http_transport;
mod mock_data_generation;
mod normalization;
mod quality_diagnostics;
mod source_mapping;
use exchange_surface::{bar_interval_ms, endpoint_for_source, provider_key_for_source};
pub(crate) use historical_cache::historical_kline_bars_for_backtest;
pub(super) use http_transport::block_on_http;
#[cfg(target_os = "windows")]
pub(super) use http_transport::fetch_json_via_powershell;
use http_transport::PingProbe;
pub(crate) use mock_data_generation::mock_kline_bars_for_backtest;
pub use mock_data_generation::MOCK_VOLATILITY;
use mock_data_generation::{mock_raw_klines, mock_raw_quote};
#[allow(unused_imports)]
pub(crate) use normalization::quote_snapshot_from_price;
use normalization::{normalize_kline_series, normalize_quote};
use quality_diagnostics::apply_snapshot_status;
#[allow(unused_imports)]
pub(crate) use quality_diagnostics::{
    attach_data_quality_snapshot, build_data_quality_summary, market_data_preview,
    market_data_quality, MarketDataPreview,
};
pub(crate) use source_mapping::data_sources_from_core_ir;

const HTTP_TIMEOUT_SECS: u64 = 10;
const QUOTE_CACHE_TTL_MS: u64 = 5_000;
const KLINE_CACHE_TTL_MS: u64 = 60_000;
const HISTORICAL_CACHE_TTL_MS: u64 = 6 * 60 * 60 * 1000;
const HISTORICAL_CACHE_MAX_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Debug)]
pub struct DataCollectionRequest<'a> {
    pub cycle_name: &'a str,
    pub core_ir: &'a CoreStrategyIr,
    pub data_fetch_counts: &'a mut BTreeMap<String, u32>,
    pub now_ms: u64,
    pub trace_id: &'a str,
}

#[derive(Debug)]
pub struct DataCollectionOutput {
    pub normalized_data: Vec<NormalizedMarketData>,
    pub events: Vec<RuntimeEvent>,
}

pub trait DataModuleProvider: Send + Sync {
    fn provider_key(&self) -> &'static str {
        "builtin.data.okx_v5_http"
    }

    fn collect(&self, request: DataCollectionRequest<'_>) -> Result<DataCollectionOutput>;
}

#[derive(Debug, Clone)]
pub(crate) struct FetchDiagnostics {
    pub(crate) provider_key: &'static str,
    pub(crate) source_status: SourceStatus,
    pub(crate) source_latency_ms: u64,
    pub(crate) endpoint: Option<String>,
    pub(crate) ping_latency_ms: Option<u64>,
    pub(crate) ping_endpoint: Option<String>,
    pub(crate) ping_error: Option<String>,
    pub(crate) fallback: Option<&'static str>,
    pub(crate) error: Option<String>,
}

impl FetchDiagnostics {
    fn with_ping(mut self, ping: PingProbe) -> Self {
        self.ping_latency_ms = ping.latency_ms;
        self.ping_endpoint = ping.endpoint;
        self.ping_error = ping.error;
        self
    }
}

#[derive(Debug, Clone)]
struct CachedSnapshot {
    data: NormalizedMarketData,
    captured_at_ms: u64,
}

#[derive(Debug)]
pub struct BuiltinDataModule {
    client: Client,
    cache: Mutex<BTreeMap<String, CachedSnapshot>>,
    /// v2.1.0: 断路器，连续失败5次→熔断60s
    breaker: Mutex<CircuitBreaker>,
}

impl Default for BuiltinDataModule {
    fn default() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
            .user_agent("quantpilot/0.1")
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            cache: Mutex::new(BTreeMap::new()),
            breaker: Mutex::new(CircuitBreaker::new(5, 60_000)),
        }
    }
}

impl BuiltinDataModule {
    fn fetch_and_normalize(
        &self,
        source: &DataSourceConfig,
        now_ms: u64,
        data_fetch_counts: &mut BTreeMap<String, u32>,
    ) -> Result<(NormalizedMarketData, FetchDiagnostics)> {
        *data_fetch_counts.entry(source.data_id.clone()).or_default() += 1;
        if let Some((cached, diagnostics)) = self.rate_limited_snapshot(source, now_ms) {
            return Ok((cached, diagnostics));
        }
        let ping = self.probe_ping(source);

        if matches!(source.exchange, Exchange::Okx) {
            // v2.1.0: 断路器检查 — 熔断时跳过HTTP直接走回退
            {
                let mut breaker = self.breaker.lock().unwrap_or_else(|e| e.into_inner());
                breaker.try_half_open(now_ms);
                if breaker.is_open() {
                    // 断路器打开: 跳过HTTP请求, 直接走缓存→mock回退
                    if let Some((cached, diagnostics)) = self.cached_snapshot(source, now_ms) {
                        return Ok((
                            cached,
                            FetchDiagnostics {
                                error: Some("断路器已熔断，使用缓存数据".to_string()),
                                ..diagnostics.with_ping(ping)
                            },
                        ));
                    }
                    let diagnostics = FetchDiagnostics {
                        provider_key: "builtin.data.okx_v5_http",
                        source_status: SourceStatus::Error,
                        source_latency_ms: 0,
                        endpoint: Some(endpoint_for_source(source)),
                        ping_latency_ms: None,
                        ping_endpoint: None,
                        ping_error: None,
                        fallback: Some("mock"),
                        error: Some("断路器已熔断，使用模拟数据".to_string()),
                    }
                    .with_ping(ping);
                    let data = self.mock_data(source, now_ms, SourceStatus::Error);
                    self.store_cache(source, &data, now_ms);
                    return Ok((data, diagnostics));
                }
            }

            match self.fetch_okx(source, now_ms) {
                Ok((data, diagnostics)) => {
                    self.breaker
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .on_success();
                    self.store_cache(source, &data, now_ms);
                    return Ok((data, diagnostics.with_ping(ping)));
                }
                Err(error) => {
                    self.breaker
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .on_failure(now_ms);
                    if let Some((cached, diagnostics)) = self.cached_snapshot(source, now_ms) {
                        return Ok((
                            cached,
                            FetchDiagnostics {
                                error: Some(format!("{error:#}")),
                                ..diagnostics.with_ping(ping)
                            },
                        ));
                    }

                    let diagnostics = FetchDiagnostics {
                        provider_key: "builtin.data.okx_v5_http",
                        source_status: SourceStatus::Error,
                        source_latency_ms: 0,
                        endpoint: Some(endpoint_for_source(source)),
                        ping_latency_ms: None,
                        ping_endpoint: None,
                        ping_error: None,
                        fallback: Some("mock"),
                        error: Some(format!("{error:#}")),
                    }
                    .with_ping(ping);
                    let data = self.mock_data(source, now_ms, SourceStatus::Error);
                    self.store_cache(source, &data, now_ms);
                    return Ok((data, diagnostics));
                }
            }
        }

        // v2.3.5: Binance 实盘数据尚未实现，返回明确错误而非静默 mock
        let fallback_data = self.mock_data(source, now_ms, SourceStatus::Error);
        self.store_cache(source, &fallback_data, now_ms);
        Ok((
            fallback_data,
            FetchDiagnostics {
                provider_key: "builtin.data.mock",
                source_status: SourceStatus::Error,
                source_latency_ms: 0,
                endpoint: None,
                ping_latency_ms: None,
                ping_endpoint: None,
                ping_error: None,
                fallback: Some("mock"),
                error: Some("Binance 实盘数据尚未支持，已回退到模拟数据。请使用 OKX 或 deterministic_mock 回放源。".to_string()),
            }
            .with_ping(ping),
        ))
    }

    fn rate_limited_snapshot(
        &self,
        source: &DataSourceConfig,
        now_ms: u64,
    ) -> Option<(NormalizedMarketData, FetchDiagnostics)> {
        let min_interval_ms = source.request_interval_ms?;
        let cache = self.cache.lock().unwrap_or_else(|e| {
            eprintln!("[data_module] 缓存锁中毒, 使用空缓存继续");
            e.into_inner()
        });
        let cached = cache.get(&source.data_id)?.clone();
        let age_ms = now_ms.saturating_sub(cached.captured_at_ms);
        if age_ms >= min_interval_ms {
            return None;
        }
        Some((
            apply_snapshot_status(&cached.data, now_ms, SourceStatus::Healthy),
            FetchDiagnostics {
                provider_key: provider_key_for_source(source),
                source_status: SourceStatus::Healthy,
                source_latency_ms: age_ms,
                endpoint: Some(endpoint_for_source(source)),
                ping_latency_ms: None,
                ping_endpoint: None,
                ping_error: None,
                fallback: Some("request_interval"),
                error: None,
            },
        ))
    }

    fn probe_ping(&self, source: &DataSourceConfig) -> PingProbe {
        http_transport::probe_ping(&self.client, source)
    }

    fn fetch_okx(
        &self,
        source: &DataSourceConfig,
        now_ms: u64,
    ) -> Result<(NormalizedMarketData, FetchDiagnostics)> {
        http_transport::fetch_okx(&self.client, source, now_ms)
    }

    fn mock_data(
        &self,
        source: &DataSourceConfig,
        now_ms: u64,
        source_status: SourceStatus,
    ) -> NormalizedMarketData {
        match source.kind {
            DataKind::KlineSeries => NormalizedMarketData::KlineSeries(normalize_kline_series(
                source,
                mock_raw_klines(source, now_ms).expect("mock kline generation should succeed"),
                now_ms,
                0,
                source_status,
            )),
            DataKind::Quote => NormalizedMarketData::Quote(normalize_quote(
                source,
                mock_raw_quote(source, now_ms).expect("mock quote generation should succeed"),
                0,
                source_status,
            )),
        }
    }

    fn cached_snapshot(
        &self,
        source: &DataSourceConfig,
        now_ms: u64,
    ) -> Option<(NormalizedMarketData, FetchDiagnostics)> {
        let cache = self.cache.lock().unwrap_or_else(|e| {
            eprintln!("[data_module] 缓存锁中毒, 使用空缓存继续");
            e.into_inner()
        });
        let cached = cache.get(&source.data_id)?.clone();
        let age_ms = now_ms.saturating_sub(cached.captured_at_ms);
        if age_ms > cache_ttl_ms(source.kind.clone()) {
            return None;
        }
        Some((
            apply_snapshot_status(&cached.data, now_ms, SourceStatus::Stale),
            FetchDiagnostics {
                provider_key: "builtin.data.okx_v5_http",
                source_status: SourceStatus::Stale,
                source_latency_ms: age_ms,
                endpoint: Some(endpoint_for_source(source)),
                ping_latency_ms: None,
                ping_endpoint: None,
                ping_error: None,
                fallback: Some("cache"),
                error: None,
            },
        ))
    }

    fn store_cache(&self, source: &DataSourceConfig, data: &NormalizedMarketData, now_ms: u64) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(
                source.data_id.clone(),
                CachedSnapshot {
                    data: data.clone(),
                    captured_at_ms: now_ms,
                },
            );
        }
    }
}

fn cache_ttl_ms(kind: DataKind) -> u64 {
    match kind {
        DataKind::Quote => QUOTE_CACHE_TTL_MS,
        DataKind::KlineSeries => KLINE_CACHE_TTL_MS,
    }
}

#[cfg(test)]
mod test_harness;
