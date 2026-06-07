use anyhow::Result;
use qrpc_core::CoreStrategyIr;
#[cfg(test)]
use qrpc_core::RuntimeEventType;
use qrpc_core::{
    DataKind, DataSourceConfig, Exchange, NormalizedMarketData, RuntimeEvent, SourceStatus,
};
use reqwest::Client;

use crate::circuit_breaker::CircuitBreaker;
#[cfg(test)]
use serde_json::json;
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
#[cfg(test)]
use exchange_surface::{parse_okx_candles, parse_okx_ticker};
pub(crate) use historical_cache::historical_kline_bars_for_backtest;
pub(super) use http_transport::block_on_http;
#[cfg(target_os = "windows")]
pub(super) use http_transport::fetch_json_via_powershell;
use http_transport::PingProbe;
pub(crate) use mock_data_generation::mock_kline_bars_for_backtest;
#[cfg(test)]
use mock_data_generation::pseudo_random;
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
mod tests {
    use super::*;
    use qrpc_core::{MarketType, Symbol};
    use qrpc_core_ir::{
        CoreMetadata, CoreSourceKind, CoreStrategyIr, CoreTimeInForce, DataBinding,
        DataBindingKind, ExecutionRule, ExecutionSizingKind,
    };
    use std::collections::BTreeMap;

    fn sample_core_ir_with_quote_binding() -> CoreStrategyIr {
        let mut source_hints = BTreeMap::new();
        source_hints.insert("exchange".into(), "binance".into());
        source_hints.insert("symbol".into(), "BTCUSDT".into());
        CoreStrategyIr {
            ir_version: qrpc_core::CORE_IR_V1_VERSION.to_string(),
            metadata: CoreMetadata {
                strategy_id: "data_test".into(),
                name: "Data Test".into(),
                source_kind: CoreSourceKind::RuntimeProtocol,
            },
            data_bindings: vec![DataBinding {
                data_id: "binance_btc_quote".into(),
                kind: DataBindingKind::Quote,
                source_hints,
            }],
            indicators: vec![],
            signal_rules: vec![],
            agent_policies: vec![],
            risk_policies: vec![],
            edges: vec![],
            execution: ExecutionRule {
                execution_id: "exec".into(),
                venue_kind: "paper".into(),
                sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
                slippage_bps: 5.0,
                taker_fee_bps: 10.0,
                total_cost_buffer_bps: 20.0,
                time_in_force: CoreTimeInForce::Gtc,
                params: BTreeMap::new(),
            },
        }
    }

    fn sample_core_ir_with_quote_and_kline_bindings() -> CoreStrategyIr {
        let mut quote_hints = BTreeMap::new();
        quote_hints.insert("exchange".into(), "binance".into());
        quote_hints.insert("symbol".into(), "BTCUSDT".into());

        let mut kline_hints = BTreeMap::new();
        kline_hints.insert("exchange".into(), "okx".into());
        kline_hints.insert("symbol".into(), "BTCUSDT".into());
        kline_hints.insert("timeframe".into(), "1m".into());

        let mut core_ir = sample_core_ir_with_quote_binding();
        core_ir.data_bindings.push(DataBinding {
            data_id: "okx_btc_kline".into(),
            kind: DataBindingKind::KlineSeries,
            source_hints: kline_hints,
        });
        core_ir.data_bindings[0].source_hints = quote_hints;
        core_ir
    }

    #[test]
    fn okx_candles_are_parsed_in_time_order() {
        let payload = json!({
            "code": "0",
            "msg": "",
            "data": [
                ["1712707200000", "71000", "71500", "70500", "71200", "100", "1", "1", "1"],
                ["1712620800000", "70000", "71200", "69800", "71000", "120", "1", "1", "1"]
            ]
        });

        let source = DataSourceConfig {
            data_id: "okx_btc_1d".into(),
            exchange: Exchange::Okx,
            symbol: Symbol::BtcUsdt,
            market_type: MarketType::Spot,
            kind: DataKind::KlineSeries,
            days: Some(2),
            interval: Some("1d".into()),
            ping_enabled: false,
            request_interval_ms: None,
            enabled: true,
        };

        let bars = parse_okx_candles(&payload, &source).unwrap();
        assert_eq!(bars.len(), 2);
        assert!(bars[0].open_time < bars[1].open_time);
        assert_eq!(bars[0].open, 70_000.0);
        assert_eq!(bars[1].close, 71_200.0);
    }

    #[test]
    fn okx_ticker_is_parsed_into_quote() {
        let payload = json!({
            "code": "0",
            "msg": "",
            "data": [{
                "bidPx": "71047.5",
                "askPx": "71047.6",
                "bidSz": "0.86246875",
                "askSz": "0.37879107",
                "ts": "1775718339217"
            }]
        });

        let quote = parse_okx_ticker(&payload).unwrap();
        assert_eq!(quote.best_bid, 71_047.5);
        assert_eq!(quote.best_ask, 71_047.6);
        assert_eq!(quote.ts, 1_775_718_339_217);
    }

    #[test]
    fn builtin_data_module_keeps_mock_for_non_okx_sources() {
        let core_ir = sample_core_ir_with_quote_binding();
        let mut counts = BTreeMap::new();

        let output = BuiltinDataModule::default()
            .collect(DataCollectionRequest {
                cycle_name: "fast",
                core_ir: &core_ir,
                data_fetch_counts: &mut counts,
                now_ms: 10,
                trace_id: "trace",
            })
            .unwrap();

        assert_eq!(output.normalized_data.len(), 1);
        assert_eq!(output.events.len(), 2);
        assert_eq!(output.events[0].event_type, RuntimeEventType::DataUpdated);
        assert_eq!(
            output.events[0].payload["provider_key"],
            "builtin.data.mock"
        );
        assert_eq!(output.events[1].event_type, RuntimeEventType::RuntimeError);
        assert_eq!(output.events[1].payload["source_health"], "Missing");
        assert_eq!(counts.get("binance_btc_quote").copied(), Some(1));
    }

    #[test]
    fn builtin_data_module_collects_mixed_sources_in_fast_cycle() {
        let core_ir = sample_core_ir_with_quote_and_kline_bindings();
        let mut counts = BTreeMap::new();

        let output = BuiltinDataModule::default()
            .collect(DataCollectionRequest {
                cycle_name: "fast",
                core_ir: &core_ir,
                data_fetch_counts: &mut counts,
                now_ms: 10,
                trace_id: "trace",
            })
            .unwrap();

        assert_eq!(output.normalized_data.len(), 2);
        assert_eq!(counts.get("binance_btc_quote").copied(), Some(1));
        assert_eq!(counts.get("okx_btc_kline").copied(), Some(1));
    }

    #[test]
    fn data_sources_from_core_ir_restores_request_controls_from_source_hints() {
        let mut core_ir = sample_core_ir_with_quote_binding();
        core_ir.data_bindings[0]
            .source_hints
            .insert("ping_enabled".into(), "true".into());
        core_ir.data_bindings[0]
            .source_hints
            .insert("request_interval_ms".into(), "1500".into());

        let sources = data_sources_from_core_ir(&core_ir);

        assert_eq!(sources.len(), 1);
        assert!(sources[0].ping_enabled);
        assert_eq!(sources[0].request_interval_ms, Some(1_500));
    }

    #[test]
    fn request_interval_uses_cached_snapshot_before_next_fetch_window() {
        let module = BuiltinDataModule::default();
        let source = DataSourceConfig {
            data_id: "binance_btc_quote".into(),
            exchange: Exchange::Binance,
            symbol: Symbol::BtcUsdt,
            market_type: MarketType::Spot,
            kind: DataKind::Quote,
            days: None,
            interval: None,
            ping_enabled: false,
            request_interval_ms: Some(5_000),
            enabled: true,
        };
        let mut counts = BTreeMap::new();

        let first = module
            .fetch_and_normalize(&source, 10_000, &mut counts)
            .unwrap();
        let second = module
            .fetch_and_normalize(&source, 12_000, &mut counts)
            .unwrap();

        assert_eq!(first.1.fallback, Some("mock"));
        assert_eq!(second.1.fallback, Some("request_interval"));
        assert_eq!(second.1.source_status, SourceStatus::Healthy);
        assert_eq!(counts.get("binance_btc_quote").copied(), Some(2));
    }

    #[test]
    fn pseudo_random_is_deterministic() {
        let a = pseudo_random(42, 7);
        let b = pseudo_random(42, 7);
        assert_eq!(a, b, "same inputs produce same output");
    }

    #[test]
    fn pseudo_random_range_is_normalized() {
        for i in 0..1000 {
            let val = pseudo_random(i, 12345);
            assert!(val >= -1.0 && val <= 1.0, "value {val} out of [-1,1]");
        }
    }

    #[test]
    fn pseudo_random_different_seed_different_output() {
        let a = pseudo_random(42, 7);
        let b = pseudo_random(42, 8);
        assert_ne!(a, b, "different seeds produce different output");
    }
}
