use anyhow::{anyhow, Context, Result};
use qrpc_core::CoreStrategyIr;
#[cfg(test)]
use qrpc_core::RuntimeEventType;
use qrpc_core::{
    DataKind, DataQualitySnapshot, DataSourceConfig, Exchange, KlineSeriesSnapshot,
    NormalizedKline, NormalizedMarketData, QuoteSnapshot, RawKline, RawQuote, RuntimeEvent,
    SourceStatus,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::circuit_breaker::CircuitBreaker;
#[cfg(test)]
use serde_json::json;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    process::Command,
    sync::Mutex,
    time::{Duration, Instant},
};
use tokio::{runtime::Handle, task};

mod collection_orchestration;
mod exchange_surface;
mod quality_diagnostics;
mod source_mapping;
use exchange_surface::{
    bar_interval_ms, binance_endpoint_for_source, binance_symbol, endpoint_for_source,
    okx_endpoint_for_source, parse_binance_klines, parse_okx_candles, parse_okx_ticker,
    ping_endpoint_for_source, provider_key_for_source,
};
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

#[derive(Debug, Clone, Default)]
struct PingProbe {
    latency_ms: Option<u64>,
    endpoint: Option<String>,
    error: Option<String>,
}

impl PingProbe {
    fn disabled() -> Self {
        Self::default()
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoricalBarsCache {
    fetched_at_ms: u64,
    bars: Vec<NormalizedKline>,
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
        if !source.ping_enabled {
            return PingProbe::disabled();
        }

        let endpoint = ping_endpoint_for_source(source);
        let started = Instant::now();
        match self.fetch_json(&endpoint) {
            Ok(_) => PingProbe {
                latency_ms: Some(started.elapsed().as_millis() as u64),
                endpoint: Some(endpoint),
                error: None,
            },
            Err(error) => PingProbe {
                latency_ms: None,
                endpoint: Some(endpoint),
                error: Some(format!("{error:#}")),
            },
        }
    }

    fn fetch_okx(
        &self,
        source: &DataSourceConfig,
        now_ms: u64,
    ) -> Result<(NormalizedMarketData, FetchDiagnostics)> {
        match source.kind {
            DataKind::KlineSeries => {
                let endpoint = okx_endpoint_for_source(source);
                let started = Instant::now();
                let payload = self
                    .fetch_json(&endpoint)
                    .with_context(|| format!("GET 请求 {endpoint} 失败"))?;
                let raw = parse_okx_candles(&payload, source)?;
                let elapsed = started.elapsed().as_millis() as u64;
                Ok((
                    NormalizedMarketData::KlineSeries(normalize_kline_series(
                        source,
                        raw,
                        now_ms,
                        elapsed,
                        SourceStatus::Healthy,
                    )),
                    FetchDiagnostics {
                        provider_key: "builtin.data.okx_v5_http",
                        source_status: SourceStatus::Healthy,
                        source_latency_ms: elapsed,
                        endpoint: Some(endpoint),
                        ping_latency_ms: None,
                        ping_endpoint: None,
                        ping_error: None,
                        fallback: None,
                        error: None,
                    },
                ))
            }
            DataKind::Quote => {
                let endpoint = okx_endpoint_for_source(source);
                let started = Instant::now();
                let payload = self
                    .fetch_json(&endpoint)
                    .with_context(|| format!("GET 请求 {endpoint} 失败"))?;
                let raw = parse_okx_ticker(&payload)?;
                let elapsed = started.elapsed().as_millis() as u64;
                Ok((
                    NormalizedMarketData::Quote(normalize_quote(
                        source,
                        raw,
                        elapsed,
                        SourceStatus::Healthy,
                    )),
                    FetchDiagnostics {
                        provider_key: "builtin.data.okx_v5_http",
                        source_status: SourceStatus::Healthy,
                        source_latency_ms: elapsed,
                        endpoint: Some(endpoint),
                        ping_latency_ms: None,
                        ping_endpoint: None,
                        ping_error: None,
                        fallback: None,
                        error: None,
                    },
                ))
            }
        }
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

    fn fetch_json(&self, endpoint: &str) -> Result<Value> {
        let client = self.client.clone();
        let endpoint = endpoint.to_string();
        let endpoint_for_reqwest = endpoint.clone();
        let primary_result = block_on_http(async move {
            #[allow(unused_assignments)]
            let mut last_status = 0u16;
            for attempt in 0..4 {
                let resp = client
                    .get(&endpoint_for_reqwest)
                    .send()
                    .await
                    .with_context(|| format!("请求 {endpoint_for_reqwest} 失败"))?;
                last_status = resp.status().as_u16();
                if last_status == 429 && attempt < 3 {
                    let delay = std::time::Duration::from_secs(1u64 << attempt);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                let status = resp.status().as_u16();
                let category = if status >= 500 {
                    "服务端临时错误"
                } else {
                    "客户端请求错误"
                };
                return resp
                    .error_for_status()
                    .with_context(|| {
                        format!(
                            "从 {endpoint_for_reqwest} 收到非成功响应 (HTTP {status}, {category})"
                        )
                    })?
                    .json::<Value>()
                    .await
                    .with_context(|| format!("从 {endpoint_for_reqwest} 收到无效 JSON"));
            }
            Err(anyhow::anyhow!(
                "请求 {endpoint_for_reqwest} 被限流 (429), 重试 3 次后仍失败"
            ))
        });

        match primary_result {
            Ok(payload) => Ok(payload),
            Err(primary_error) => {
                #[cfg(target_os = "windows")]
                {
                    fetch_json_via_powershell(endpoint)
                        .with_context(|| format!("{primary_error:#}"))
                }

                #[cfg(not(target_os = "windows"))]
                {
                    Err(primary_error)
                }
            }
        }
    }
}

fn block_on_http<F, T>(future: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    match Handle::try_current() {
        Ok(handle) => task::block_in_place(|| handle.block_on(future)),
        // v2.4.0 P1-B3: 使用 LazyLock 全局单例 Runtime, 避免每次创建新线程池
        Err(_) => FALLBACK_RT.block_on(future),
    }
}

static FALLBACK_RT: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("创建 Fallback tokio 运行时失败")
});

#[cfg(target_os = "windows")]
fn fetch_json_via_powershell(endpoint: String) -> Result<Value> {
    let escaped = endpoint.replace('\'', "''");
    let script = format!(
        "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
         $resp = Invoke-RestMethod -Uri '{escaped}' -TimeoutSec {HTTP_TIMEOUT_SECS}; \
         $resp | ConvertTo-Json -Depth 64 -Compress"
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .with_context(|| format!("调用 powershell 获取 {endpoint} 失败"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Err(anyhow!(
            "powershell HTTP 回退获取 {endpoint} 失败: {}",
            if stderr.is_empty() { stdout } else { stderr }
        ));
    }

    serde_json::from_slice::<Value>(&output.stdout)
        .with_context(|| format!("powershell 为 {endpoint} 返回了无效 JSON"))
}

fn normalize_kline_series(
    source: &DataSourceConfig,
    raw: Vec<RawKline>,
    now_ms: u64,
    source_latency_ms: u64,
    source_status: SourceStatus,
) -> KlineSeriesSnapshot {
    // v1.1.14: OHLC 关系验证，拒绝非法K线
    let bars = raw
        .into_iter()
        .filter_map(|bar| {
            let high = bar.high.max(bar.open).max(bar.close).max(bar.low);
            let low = bar.low.min(bar.open).min(bar.close).min(bar.high);
            // v1.2.1: 记录 OHLC 被 clamp 的情况
            if high != bar.high || low != bar.low {
                eprintln!("[data_module] 修正K线 OHLC 关系: open_time={}, O={:.2} H={:.2} L={:.2} C={:.2} (high clamped from {} to {}, low clamped from {} to {})",
                    bar.open_time, bar.open, bar.high, bar.low, bar.close, bar.high, high, bar.low, low);
            }
            if bar.high < bar.open || bar.high < bar.close || bar.low > bar.open || bar.low > bar.close || bar.low > bar.high || bar.volume < 0.0 {
                eprintln!("[data_module] 跳过非法K线: open_time={}, O={:.2} H={:.2} L={:.2} C={:.2} V={:.2}",
                    bar.open_time, bar.open, bar.high, bar.low, bar.close, bar.volume);
                None
            } else {
                Some(NormalizedKline {
            exchange: source.exchange.clone(),
            symbol: source.symbol.clone(),
            market_type: source.market_type.clone(),
                interval: source.interval.clone().unwrap_or_else(|| "1d".into()),
                open_time_ms: bar.open_time,
                close_time_ms: bar.close_time,
                open: bar.open,
                high,
                low,
                close: bar.close,
                volume: bar.volume.max(0.0),
            })
            }
        })
        .collect::<Vec<_>>();

    KlineSeriesSnapshot {
        data_id: source.data_id.clone(),
        exchange: source.exchange.clone(),
        symbol: source.symbol.clone(),
        market_type: source.market_type.clone(),
        interval: source.interval.clone().unwrap_or_else(|| "1d".into()),
        window_len: bars.len(),
        bars,
        ts_ms: now_ms,
        source_latency_ms,
        source_status,
        data_quality: DataQualitySnapshot::default(),
    }
}

fn normalize_quote(
    source: &DataSourceConfig,
    raw: RawQuote,
    source_latency_ms: u64,
    source_status: SourceStatus,
) -> QuoteSnapshot {
    QuoteSnapshot {
        data_id: source.data_id.clone(),
        exchange: source.exchange.clone(),
        symbol: source.symbol.clone(),
        market_type: source.market_type.clone(),
        best_bid: raw.best_bid,
        best_ask: raw.best_ask,
        bid_size: raw.bid_size,
        ask_size: raw.ask_size,
        mid_price: (raw.best_bid + raw.best_ask) / 2.0,
        ts_ms: raw.ts,
        source_latency_ms,
        source_status,
        data_quality: DataQualitySnapshot::default(),
    }
}

fn cache_ttl_ms(kind: DataKind) -> u64 {
    match kind {
        DataKind::Quote => QUOTE_CACHE_TTL_MS,
        DataKind::KlineSeries => KLINE_CACHE_TTL_MS,
    }
}

/// Configurable mock volatility — set via TestRunner before backtest
pub static MOCK_VOLATILITY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

const DEFAULT_MOCK_VOLATILITY: f64 = 0.015;

fn get_mock_volatility() -> f64 {
    let bits = MOCK_VOLATILITY.load(std::sync::atomic::Ordering::Relaxed);
    if bits == 0 {
        return DEFAULT_MOCK_VOLATILITY;
    }
    let vol = f64::from_bits(bits);
    // v2.1.x: 拒绝 NaN/Inf 注入; v2.4.0 P1-C3: 加 clamp 上限防止极端值
    if !vol.is_finite() {
        DEFAULT_MOCK_VOLATILITY
    } else {
        vol.clamp(1e-6, 1.0)
    }
}

fn mock_raw_klines(source: &DataSourceConfig, now_ms: u64) -> Result<Vec<RawKline>> {
    let days = source.days.unwrap_or(150);
    let mut bars = Vec::new();
    let interval_ms = 86_400_000_u64;
    // Deterministic pseudo-random seed from symbol debug name hash
    let symbol_bytes = format!("{:?}", source.symbol).into_bytes();
    let symbol_seed = symbol_bytes
        .iter()
        .fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));
    let seed = symbol_seed.wrapping_add(days as u64);

    for idx in 0..days {
        let day_index = idx as f64;
        // Multi-regime price: slow grind → breakout → consolidation → rally → pullback
        let trend_close = match source.exchange {
            Exchange::Binance => {
                if idx < 50 {
                    // Regime 1: slow accumulation 42k→43k
                    42_000.0 + day_index * 20.0
                } else if idx < 100 {
                    // Regime 2: breakout 43k→52k
                    43_000.0 + (idx - 50) as f64 * 180.0
                } else if idx < 140 {
                    // Regime 3: consolidation 52k→50k (slight pullback)
                    52_000.0 - (idx - 100) as f64 * 50.0
                } else {
                    // Regime 4: rally 50k→68k
                    50_000.0 + (idx - 140) as f64 * 300.0
                }
            }
            Exchange::Okx => 42_100.0 + day_index * 22.0,
        };

        // Add stochastic volatility (configurable via MOCK_VOLATILITY)
        let vol = get_mock_volatility();
        let noise = pseudo_random(idx as u64, seed) * trend_close * vol;
        let close = trend_close + noise;
        let daily_range = close * (0.002 + pseudo_random(idx as u64 + 1, seed).abs() * 0.008);
        let open = close - daily_range * pseudo_random(idx as u64 + 2, seed);
        let high = close.max(open) + daily_range * pseudo_random(idx as u64 + 3, seed).abs() * 0.5;
        let low = close.min(open) - daily_range * pseudo_random(idx as u64 + 4, seed).abs() * 0.5;
        let close_time = now_ms.saturating_sub(interval_ms * (days - idx) as u64);
        let open_time = close_time.saturating_sub(interval_ms);
        bars.push(RawKline {
            open_time,
            open,
            high,
            low,
            close,
            volume: 1000.0 + idx as f64 * 10.0 + pseudo_random(idx as u64 + 5, seed).abs() * 500.0,
            close_time,
        });
    }

    Ok(bars)
}

/// Deterministic pseudo-random in [-1.0, 1.0] based on index and seed
fn pseudo_random(idx: u64, seed: u64) -> f64 {
    let val = idx
        .wrapping_mul(6364136223846793005)
        .wrapping_add(seed.wrapping_mul(1442695040888963407))
        .wrapping_add(1);
    let mixed = (val ^ (val >> 33)).wrapping_mul(0xFF51AFD7ED558CCD);
    let mixed = (mixed ^ (mixed >> 33)).wrapping_mul(0xC4CEB9FE1A85EC53);
    let mixed = mixed ^ (mixed >> 33);
    // Map u64 to [-1.0, 1.0]
    (mixed as f64 / u64::MAX as f64) * 2.0 - 1.0
}

fn mock_raw_quote(source: &DataSourceConfig, now_ms: u64) -> Result<RawQuote> {
    let mid = match source.exchange {
        Exchange::Binance => 50_000.0,
        Exchange::Okx => 50_350.0,
    };
    Ok(RawQuote {
        best_bid: mid - 5.0,
        best_ask: mid + 5.0,
        bid_size: 10.0,
        ask_size: 10.0,
        ts: now_ms.saturating_sub(10),
    })
}

pub(crate) fn mock_kline_bars_for_backtest(
    source: &DataSourceConfig,
    now_ms: u64,
) -> Result<Vec<NormalizedKline>> {
    Ok(normalize_kline_series(
        source,
        mock_raw_klines(source, now_ms)?,
        now_ms,
        0,
        SourceStatus::Healthy,
    )
    .bars)
}

fn historical_cache_path(source: &DataSourceConfig) -> PathBuf {
    let interval = source.interval.as_deref().unwrap_or("1d");
    let days = source.days.unwrap_or(200);
    // 消毒符号名：仅保留字母数字和下划线，防止路径遍历
    let safe_symbol =
        sanitize_filename_component(&binance_symbol(source.symbol.clone()).to_ascii_lowercase());
    PathBuf::from("storage")
        .join("cache")
        .join("historical")
        .join(format!(
            "{:?}_{}_{}_{}_{}.json",
            source.exchange,
            safe_symbol,
            interval,
            days,
            match source.kind {
                DataKind::KlineSeries => "kline",
                DataKind::Quote => "quote",
            }
        ))
}

/// 移除路径中不安全的字符，防止目录遍历攻击
fn sanitize_filename_component(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(64)
        .collect()
}

fn load_historical_cache(source: &DataSourceConfig, now_ms: u64) -> Option<Vec<NormalizedKline>> {
    let path = historical_cache_path(source);
    let body = read_historical_cache_body(&path)?;
    let cache = serde_json::from_str::<HistoricalBarsCache>(&body).ok()?;
    let is_fresh = now_ms.saturating_sub(cache.fetched_at_ms) <= HISTORICAL_CACHE_TTL_MS;
    if is_fresh && !cache.bars.is_empty() {
        Some(cache.bars)
    } else {
        None
    }
}

fn load_stale_historical_cache(source: &DataSourceConfig) -> Option<Vec<NormalizedKline>> {
    let path = historical_cache_path(source);
    let body = read_historical_cache_body(&path)?;
    let cache = serde_json::from_str::<HistoricalBarsCache>(&body).ok()?;
    (!cache.bars.is_empty()).then_some(cache.bars)
}

fn read_historical_cache_body(path: &std::path::Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.len() > HISTORICAL_CACHE_MAX_BYTES {
        return None;
    }
    let body = fs::read_to_string(path).ok()?;
    (body.len() as u64 <= HISTORICAL_CACHE_MAX_BYTES).then_some(body)
}

fn persist_historical_cache(
    source: &DataSourceConfig,
    now_ms: u64,
    bars: &[NormalizedKline],
) -> Result<()> {
    let path = historical_cache_path(source);
    // v1.1.1: 写入前检查存储配额（cache 为 Temporary 生命周期，上限 200MB）
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建历史数据缓存目录失败: {}", parent.display()))?;
        let cache_dir = PathBuf::from("storage").join("cache").join("historical");
        let dir_size: u64 = std::fs::read_dir(&cache_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.metadata().ok())
                    .filter(|m| m.is_file())
                    .map(|m| m.len())
                    .sum()
            })
            .unwrap_or(0);
        const CACHE_MAX_BYTES: u64 = 200 * 1024 * 1024;
        if dir_size > CACHE_MAX_BYTES {
            return Err(anyhow!(
                "历史缓存目录已满: 当前 {} MB, 上限 200 MB，跳过缓存写入",
                dir_size / (1024 * 1024)
            ));
        }
    } else {
        fs::create_dir_all(path.parent().unwrap_or(&path))?;
    }
    let body = serde_json::to_string_pretty(&HistoricalBarsCache {
        fetched_at_ms: now_ms,
        bars: bars.to_vec(),
    })?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &body).with_context(|| format!("写入历史缓存 {} 失败", path.display()))?;
    // v2.3.3: fsync tmp 确保数据落盘
    if let Ok(f) = std::fs::File::open(&tmp) {
        let _ = f.sync_all();
    }
    fs::rename(&tmp, &path).with_context(|| format!("重命名历史缓存 {} 失败", path.display()))?;
    // v2.3.3: fsync 父目录确保 rename 落盘
    if let Some(parent) = path.parent() {
        if let Ok(f) = std::fs::File::open(parent) {
            let _ = f.sync_all();
        }
    }
    Ok(())
}

// v2.1.0: 复用 reqwest Client，避免每次请求重建连接池
fn shared_http_client() -> &'static Client {
    static CLIENT: std::sync::OnceLock<Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
            .user_agent("quantpilot/0.1")
            .build()
            .unwrap_or_else(|_| Client::new())
    })
}

fn fetch_historical_raw_klines(source: &DataSourceConfig) -> Result<Vec<RawKline>> {
    let endpoint = match source.exchange {
        Exchange::Okx => okx_endpoint_for_source(source),
        Exchange::Binance => binance_endpoint_for_source(source),
    };
    let client = shared_http_client().clone();
    let endpoint_for_reqwest = endpoint.clone();
    let payload = block_on_http(async move {
        client
            .get(&endpoint_for_reqwest)
            .send()
            .await
            .with_context(|| format!("请求 {endpoint_for_reqwest} 失败"))?
            .error_for_status()
            .with_context(|| format!("从 {endpoint_for_reqwest} 收到非成功响应"))?
            .json::<Value>()
            .await
            .with_context(|| format!("从 {endpoint_for_reqwest} 收到无效 JSON"))
    })
    .or_else(|primary_error| {
        #[cfg(target_os = "windows")]
        {
            fetch_json_via_powershell(endpoint.clone())
                .with_context(|| format!("{primary_error:#}"))
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(primary_error)
        }
    })?;

    match source.exchange {
        Exchange::Okx => parse_okx_candles(&payload, source),
        Exchange::Binance => parse_binance_klines(&payload),
    }
}

pub(crate) fn historical_kline_bars_for_backtest(
    source: &DataSourceConfig,
    now_ms: u64,
) -> Result<Vec<NormalizedKline>> {
    if let Some(cached) = load_historical_cache(source, now_ms) {
        return Ok(cached);
    }

    match fetch_historical_raw_klines(source) {
        Ok(raw) => {
            let bars = normalize_kline_series(source, raw, now_ms, 0, SourceStatus::Healthy).bars;
            persist_historical_cache(source, now_ms, &bars)?;
            Ok(bars)
        }
        Err(error) => {
            if let Some(cached) = load_stale_historical_cache(source) {
                return Ok(cached);
            }
            Err(error).with_context(|| {
                format!(
                    "加载 {} (交易所 {:?}) 的历史重放 K 线数据失败",
                    source.data_id, source.exchange
                )
            })
        }
    }
}

#[allow(dead_code)]
pub(crate) fn quote_snapshot_from_price(
    source: &DataSourceConfig,
    mid_price: f64,
    ts_ms: u64,
) -> QuoteSnapshot {
    normalize_quote(
        source,
        RawQuote {
            best_bid: mid_price - 5.0,
            best_ask: mid_price + 5.0,
            bid_size: 10.0,
            ask_size: 10.0,
            ts: ts_ms,
        },
        0,
        SourceStatus::Healthy,
    )
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
