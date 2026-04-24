use anyhow::{anyhow, Context, Result};
use qrpc_core::CoreStrategyIr;
use qrpc_core::{
    DataKind, DataQualitySnapshot, DataSourceConfig, Exchange, KlineSeriesSnapshot,
    NormalizedKline, NormalizedMarketData, QuoteSnapshot, RawKline, RawQuote, RuntimeEvent,
    RuntimeEventType, SourceHealth, SourceStatus, Symbol,
};
use qrpc_core_ir::DataBindingKind;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    process::Command,
    sync::Mutex,
    time::{Duration, Instant},
};
use tokio::{runtime::Handle, task};

const OKX_BASE_URL: &str = "https://www.okx.com";
const HTTP_TIMEOUT_SECS: u64 = 10;
const QUOTE_CACHE_TTL_MS: u64 = 5_000;
const KLINE_CACHE_TTL_MS: u64 = 60_000;
const HISTORICAL_CACHE_TTL_MS: u64 = 6 * 60 * 60 * 1000;
const BINANCE_BASE_URL: &str = "https://api.binance.com";

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
struct HistoricalBarsCache {
    fetched_at_ms: u64,
    bars: Vec<NormalizedKline>,
}

#[derive(Debug)]
pub struct BuiltinDataModule {
    client: Client,
    cache: Mutex<BTreeMap<String, CachedSnapshot>>,
}

impl Default for BuiltinDataModule {
    fn default() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
            .user_agent("quantpilot/0.1")
            .build()
            .expect("reqwest client should be constructible");
        Self {
            client,
            cache: Mutex::new(BTreeMap::new()),
        }
    }
}

impl DataModuleProvider for BuiltinDataModule {
    fn collect(&self, request: DataCollectionRequest<'_>) -> Result<DataCollectionOutput> {
        let mut normalized_data = Vec::new();
        let mut events = Vec::new();
        let mut fetched = BTreeSet::new();
        let data_sources = data_sources_from_core_ir(request.core_ir);

        for source in data_sources.iter().filter(|item| item.enabled) {
            if !fetched.insert(source.data_id.clone()) {
                continue;
            }

            let (data, diagnostics) =
                self.fetch_and_normalize(source, request.now_ms, request.data_fetch_counts)?;
            let data = attach_data_quality_snapshot(source, data, &diagnostics, request.now_ms);
            let preview = market_data_preview(&data);
            let quality = market_data_quality(&data);
            let quality_summary =
                build_data_quality_summary(source, &quality, &diagnostics, preview.latest_price);
            events.push(RuntimeEvent {
                event_id: format!("evt-data-{}-{}", source.data_id, request.now_ms),
                event_type: RuntimeEventType::DataUpdated,
                trace_id: request.trace_id.to_string(),
                source_id: source.data_id.clone(),
                ts_ms: request.now_ms,
                payload: json!({
                    "provider_key": diagnostics.provider_key,
                    "exchange": format!("{:?}", source.exchange),
                    "kind": format!("{:?}", source.kind),
                    "source_status": format!("{:?}", diagnostics.source_status),
                    "source_latency_ms": diagnostics.source_latency_ms,
                    "endpoint": diagnostics.endpoint,
                    "ping_enabled": source.ping_enabled,
                    "ping_latency_ms": diagnostics.ping_latency_ms,
                    "ping_endpoint": diagnostics.ping_endpoint,
                    "ping_error": diagnostics.ping_error,
                    "request_interval_ms": source.request_interval_ms,
                    "fallback": diagnostics.fallback,
                    "error": diagnostics.error,
                    "source_health": format!("{:?}", quality.source_health),
                    "freshness_ms": quality.freshness_ms,
                    "stale_after_ms": quality.stale_after_ms,
                    "gap_count": quality.gap_count,
                    "quality_flags": quality.quality_flags,
                    "explanation_summary": quality_summary,
                    "latest_price": preview.latest_price,
                    "latest_bar_time": preview.latest_bar_time,
                    "bid_price": preview.bid_price,
                    "ask_price": preview.ask_price,
                    "ts_ms": preview.ts_ms,
                }),
            });
            if quality.source_health != SourceHealth::Healthy || !quality.quality_flags.is_empty() {
                let quality_event_type = if matches!(
                    quality.source_health,
                    SourceHealth::Missing | SourceHealth::Error
                ) {
                    RuntimeEventType::RuntimeError
                } else {
                    RuntimeEventType::RuntimeWarning
                };
                events.push(RuntimeEvent {
                    event_id: format!("evt-data-quality-{}-{}", source.data_id, request.now_ms),
                    event_type: quality_event_type,
                    trace_id: request.trace_id.to_string(),
                    source_id: source.data_id.clone(),
                    ts_ms: request.now_ms,
                    payload: json!({
                        "provider_key": diagnostics.provider_key,
                        "exchange": format!("{:?}", source.exchange),
                        "kind": format!("{:?}", source.kind),
                        "source_status": format!("{:?}", diagnostics.source_status),
                        "source_latency_ms": diagnostics.source_latency_ms,
                        "ping_latency_ms": diagnostics.ping_latency_ms,
                        "ping_endpoint": diagnostics.ping_endpoint,
                        "ping_error": diagnostics.ping_error,
                        "request_interval_ms": source.request_interval_ms,
                        "fallback": diagnostics.fallback,
                        "error": diagnostics.error,
                        "source_health": format!("{:?}", quality.source_health),
                        "freshness_ms": quality.freshness_ms,
                        "stale_after_ms": quality.stale_after_ms,
                        "gap_count": quality.gap_count,
                        "quality_flags": quality.quality_flags,
                        "endpoint": diagnostics.endpoint,
                        "latest_price": preview.latest_price,
                        "latest_bar_time": preview.latest_bar_time,
                        "bid_price": preview.bid_price,
                        "ask_price": preview.ask_price,
                        "ts_ms": preview.ts_ms,
                        "explanation_summary": quality_summary,
                    }),
                });
            }
            normalized_data.push(data);
        }

        Ok(DataCollectionOutput {
            normalized_data,
            events,
        })
    }
}

pub(crate) fn data_sources_from_core_ir(core_ir: &CoreStrategyIr) -> Vec<DataSourceConfig> {
    core_ir
        .data_bindings
        .iter()
        .filter_map(|binding| {
            let exchange = binding
                .source_hints
                .get("exchange")
                .and_then(|value| parse_exchange_hint(value));
            let symbol = binding
                .source_hints
                .get("symbol")
                .and_then(|value| parse_symbol_hint(value));
            match (exchange, symbol) {
                (Some(exchange), Some(symbol)) => Some(DataSourceConfig {
                    data_id: binding.data_id.clone(),
                    exchange,
                    symbol,
                    market_type: qrpc_core::MarketType::Spot,
                    kind: match binding.kind {
                        DataBindingKind::KlineSeries => DataKind::KlineSeries,
                        DataBindingKind::Quote => DataKind::Quote,
                    },
                    days: matches!(binding.kind, DataBindingKind::KlineSeries).then_some(200),
                    interval: matches!(binding.kind, DataBindingKind::KlineSeries).then(|| {
                        binding
                            .source_hints
                            .get("timeframe")
                            .cloned()
                            .unwrap_or_else(|| "1d".to_string())
                    }),
                    ping_enabled: binding
                        .source_hints
                        .get("ping_enabled")
                        .and_then(|value| parse_bool_hint(value))
                        .unwrap_or(false),
                    request_interval_ms: binding
                        .source_hints
                        .get("request_interval_ms")
                        .and_then(|value| parse_u64_hint(value)),
                    enabled: true,
                }),
                _ => None,
            }
        })
        .collect()
}

fn parse_exchange_hint(value: &str) -> Option<Exchange> {
    match value.to_ascii_lowercase().as_str() {
        "binance" => Some(Exchange::Binance),
        "okx" => Some(Exchange::Okx),
        _ => None,
    }
}

fn parse_symbol_hint(value: &str) -> Option<Symbol> {
    let value = value.trim();
    (!value.is_empty()).then(|| Symbol::parse(value))
}

fn parse_bool_hint(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_u64_hint(value: &str) -> Option<u64> {
    value.trim().parse::<u64>().ok()
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MarketDataPreview {
    pub(crate) latest_price: Option<f64>,
    pub(crate) latest_bar_time: Option<u64>,
    pub(crate) bid_price: Option<f64>,
    pub(crate) ask_price: Option<f64>,
    pub(crate) ts_ms: Option<u64>,
}

fn data_timestamp_ms(data: &NormalizedMarketData) -> Option<u64> {
    match data {
        NormalizedMarketData::KlineSeries(series) => series
            .bars
            .last()
            .map(|bar| bar.close_time_ms)
            .or(Some(series.ts_ms)),
        NormalizedMarketData::Quote(quote) => Some(quote.ts_ms),
    }
}

fn stale_after_ms_for_source(source: &DataSourceConfig, data: &NormalizedMarketData) -> u64 {
    match data {
        NormalizedMarketData::Quote(_) => source.request_interval_ms.unwrap_or(QUOTE_CACHE_TTL_MS),
        NormalizedMarketData::KlineSeries(series) => {
            let interval_ms = bar_interval_ms(&series.interval);
            interval_ms.saturating_mul(2)
        }
    }
}

fn gap_count_for_data(data: &NormalizedMarketData) -> u32 {
    let NormalizedMarketData::KlineSeries(series) = data else {
        return 0;
    };
    if series.bars.len() < 2 {
        return 0;
    }

    let interval_ms = bar_interval_ms(&series.interval).max(1);
    let mut gap_count = 0u32;
    for pair in series.bars.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        let observed_delta = right.open_time_ms.saturating_sub(left.open_time_ms);
        if observed_delta > interval_ms {
            let missing = observed_delta / interval_ms;
            if missing > 1 {
                gap_count = gap_count.saturating_add((missing - 1) as u32);
            }
        }
    }
    gap_count
}

fn build_quality_flags(
    diagnostics: &FetchDiagnostics,
    freshness_ms: u64,
    stale_after_ms: u64,
    gap_count: u32,
) -> Vec<String> {
    let delayed_threshold_ms = if stale_after_ms > 0 {
        stale_after_ms / 2
    } else {
        0
    };
    let mut flags = Vec::new();

    if diagnostics.error.is_some() {
        flags.push("source_error".to_string());
    }
    if matches!(diagnostics.fallback, Some("mock")) {
        flags.push("missing_updates".to_string());
    }
    if matches!(diagnostics.fallback, Some("cache")) {
        flags.push("fallback_cache".to_string());
    }
    if diagnostics.source_status == SourceStatus::Stale || freshness_ms > stale_after_ms {
        flags.push("stale_data".to_string());
    }
    if delayed_threshold_ms > 0 && diagnostics.source_latency_ms > delayed_threshold_ms {
        flags.push("delayed_update".to_string());
    }
    if delayed_threshold_ms > 0
        && diagnostics
            .ping_latency_ms
            .map(|latency| latency > delayed_threshold_ms)
            .unwrap_or(false)
    {
        flags.push("ping_delayed".to_string());
    }
    if diagnostics.ping_error.is_some() {
        flags.push("ping_unavailable".to_string());
    }
    if gap_count > 0 {
        flags.push("gaps_detected".to_string());
    }

    flags.sort();
    flags.dedup();
    flags
}

fn source_health_from_flags(diagnostics: &FetchDiagnostics, flags: &[String]) -> SourceHealth {
    if matches!(diagnostics.fallback, Some("mock")) {
        return SourceHealth::Missing;
    }
    if diagnostics.source_status == SourceStatus::Error
        || flags.iter().any(|flag| flag == "source_error")
    {
        return SourceHealth::Error;
    }
    if diagnostics.source_status == SourceStatus::Stale
        || flags.iter().any(|flag| flag == "stale_data")
    {
        return SourceHealth::Stale;
    }
    if flags.iter().any(|flag| {
        matches!(
            flag.as_str(),
            "delayed_update" | "ping_delayed" | "gaps_detected"
        )
    }) {
        return SourceHealth::Delayed;
    }
    SourceHealth::Healthy
}

fn build_data_quality_snapshot(
    source: &DataSourceConfig,
    data: &NormalizedMarketData,
    diagnostics: &FetchDiagnostics,
    now_ms: u64,
) -> DataQualitySnapshot {
    let freshness_ms = data_timestamp_ms(data)
        .map(|timestamp| now_ms.saturating_sub(timestamp))
        .unwrap_or_default();
    let stale_after_ms = stale_after_ms_for_source(source, data);
    let gap_count = gap_count_for_data(data);
    let quality_flags = build_quality_flags(diagnostics, freshness_ms, stale_after_ms, gap_count);
    let source_health = source_health_from_flags(diagnostics, &quality_flags);

    DataQualitySnapshot {
        source_health,
        freshness_ms,
        stale_after_ms,
        gap_count,
        quality_flags,
    }
}

pub(crate) fn attach_data_quality_snapshot(
    source: &DataSourceConfig,
    mut data: NormalizedMarketData,
    diagnostics: &FetchDiagnostics,
    now_ms: u64,
) -> NormalizedMarketData {
    let quality = build_data_quality_snapshot(source, &data, diagnostics, now_ms);
    match &mut data {
        NormalizedMarketData::KlineSeries(series) => series.data_quality = quality,
        NormalizedMarketData::Quote(quote) => quote.data_quality = quality,
    }
    data
}

pub(crate) fn market_data_quality(data: &NormalizedMarketData) -> DataQualitySnapshot {
    match data {
        NormalizedMarketData::KlineSeries(series) => series.data_quality.clone(),
        NormalizedMarketData::Quote(quote) => quote.data_quality.clone(),
    }
}

pub(crate) fn build_data_quality_summary(
    source: &DataSourceConfig,
    quality: &DataQualitySnapshot,
    diagnostics: &FetchDiagnostics,
    latest_price: Option<f64>,
) -> String {
    let source_name = format!("{}::{:?}", source.data_id, source.kind);
    let health = format!("{:?}", quality.source_health).to_ascii_lowercase();
    let mut fragments = vec![format!("{source_name} quality {health}")];

    if let Some(price) = latest_price {
        fragments.push(format!("price {:.2}", price));
    }
    fragments.push(format!("freshness {}ms", quality.freshness_ms));
    fragments.push(format!("latency {}ms", diagnostics.source_latency_ms));

    if quality.gap_count > 0 {
        fragments.push(format!("gaps {}", quality.gap_count));
    }
    if !quality.quality_flags.is_empty() {
        fragments.push(format!("flags {}", quality.quality_flags.join(",")));
    }

    fragments.join(" | ")
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
            match self.fetch_okx(source, now_ms) {
                Ok((data, diagnostics)) => {
                    self.store_cache(source, &data, now_ms);
                    return Ok((data, diagnostics.with_ping(ping)));
                }
                Err(error) => {
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

        let data = self.mock_data(source, now_ms, SourceStatus::Healthy);
        self.store_cache(source, &data, now_ms);
        Ok((
            data,
            FetchDiagnostics {
                provider_key: "builtin.data.mock",
                source_status: SourceStatus::Healthy,
                source_latency_ms: 0,
                endpoint: None,
                ping_latency_ms: None,
                ping_endpoint: None,
                ping_error: None,
                fallback: Some("mock"),
                error: None,
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
        let cache = self.cache.lock().ok()?;
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
                    .with_context(|| format!("failed to GET {endpoint}"))?;
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
                    .with_context(|| format!("failed to GET {endpoint}"))?;
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
        let cache = self.cache.lock().ok()?;
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
            client
                .get(&endpoint_for_reqwest)
                .send()
                .await
                .with_context(|| format!("request failed for {endpoint_for_reqwest}"))?
                .error_for_status()
                .with_context(|| format!("non-success response from {endpoint_for_reqwest}"))?
                .json::<Value>()
                .await
                .with_context(|| format!("invalid JSON from {endpoint_for_reqwest}"))
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
        Err(_) => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to create tokio runtime for HTTP request")?
            .block_on(future),
    }
}

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
        .with_context(|| format!("failed to invoke powershell for {endpoint}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Err(anyhow!(
            "powershell HTTP fallback failed for {endpoint}: {}",
            if stderr.is_empty() { stdout } else { stderr }
        ));
    }

    serde_json::from_slice::<Value>(&output.stdout)
        .with_context(|| format!("powershell returned invalid JSON for {endpoint}"))
}

fn normalize_kline_series(
    source: &DataSourceConfig,
    raw: Vec<RawKline>,
    now_ms: u64,
    source_latency_ms: u64,
    source_status: SourceStatus,
) -> KlineSeriesSnapshot {
    let bars = raw
        .into_iter()
        .map(|bar| NormalizedKline {
            exchange: source.exchange.clone(),
            symbol: source.symbol.clone(),
            market_type: source.market_type.clone(),
            interval: source.interval.clone().unwrap_or_else(|| "1d".into()),
            open_time_ms: bar.open_time,
            close_time_ms: bar.close_time,
            open: bar.open,
            high: bar.high,
            low: bar.low,
            close: bar.close,
            volume: bar.volume,
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

fn provider_key_for_source(source: &DataSourceConfig) -> &'static str {
    match source.exchange {
        Exchange::Okx => "builtin.data.okx_v5_http",
        Exchange::Binance => "builtin.data.mock",
    }
}

fn endpoint_for_source(source: &DataSourceConfig) -> String {
    match source.exchange {
        Exchange::Okx => okx_endpoint_for_source(source),
        Exchange::Binance => binance_endpoint_for_source(source),
    }
}

fn ping_endpoint_for_source(source: &DataSourceConfig) -> String {
    match source.exchange {
        Exchange::Okx => format!("{OKX_BASE_URL}/api/v5/public/time"),
        Exchange::Binance => format!("{BINANCE_BASE_URL}/api/v3/ping"),
    }
}

fn binance_endpoint_for_source(source: &DataSourceConfig) -> String {
    let symbol = binance_symbol(source.symbol.clone());
    match source.kind {
        DataKind::KlineSeries => {
            let interval = source.interval.as_deref().unwrap_or("1d");
            let limit = source.days.unwrap_or(200).clamp(1, 1000);
            format!(
                "{BINANCE_BASE_URL}/api/v3/klines?symbol={symbol}&interval={interval}&limit={limit}"
            )
        }
        DataKind::Quote => {
            format!("{BINANCE_BASE_URL}/api/v3/ticker/bookTicker?symbol={symbol}")
        }
    }
}

fn okx_endpoint_for_source(source: &DataSourceConfig) -> String {
    let inst_id = okx_inst_id(source.symbol.clone(), source.market_type.clone());
    match source.kind {
        DataKind::KlineSeries => {
            let bar = okx_bar(source.interval.as_deref().unwrap_or("1d"));
            let limit = source.days.unwrap_or(200).min(300);
            format!(
                "{OKX_BASE_URL}/api/v5/market/history-candles?instId={inst_id}&bar={bar}&limit={limit}"
            )
        }
        DataKind::Quote => {
            format!("{OKX_BASE_URL}/api/v5/market/ticker?instId={inst_id}")
        }
    }
}

fn okx_inst_id(symbol: Symbol, market_type: qrpc_core::MarketType) -> String {
    match market_type {
        qrpc_core::MarketType::Spot => okx_spot_symbol(symbol.as_str()),
    }
}

fn binance_symbol(symbol: Symbol) -> String {
    symbol.as_str().to_string()
}

fn okx_spot_symbol(symbol: &str) -> String {
    const KNOWN_QUOTES: [&str; 4] = ["USDT", "USDC", "BTC", "ETH"];
    for quote in KNOWN_QUOTES {
        if let Some(base) = symbol.strip_suffix(quote) {
            if !base.is_empty() {
                return format!("{base}-{quote}");
            }
        }
    }
    symbol.to_string()
}

fn okx_bar(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "5m" => "5m",
        "1h" => "1H",
        "1d" => "1Dutc",
        _ => "1Dutc",
    }
}

fn bar_interval_ms(interval: &str) -> u64 {
    match interval {
        "1m" => 60_000,
        "5m" => 300_000,
        "1h" => 3_600_000,
        "1d" => 86_400_000,
        _ => 86_400_000,
    }
}

fn cache_ttl_ms(kind: DataKind) -> u64 {
    match kind {
        DataKind::Quote => QUOTE_CACHE_TTL_MS,
        DataKind::KlineSeries => KLINE_CACHE_TTL_MS,
    }
}

fn apply_snapshot_status(
    snapshot: &NormalizedMarketData,
    now_ms: u64,
    source_status: SourceStatus,
) -> NormalizedMarketData {
    match snapshot {
        NormalizedMarketData::KlineSeries(series) => {
            let mut next = series.clone();
            next.ts_ms = now_ms;
            next.source_status = source_status;
            next.data_quality.freshness_ms = data_timestamp_ms(snapshot)
                .map(|timestamp| now_ms.saturating_sub(timestamp))
                .unwrap_or_default();
            NormalizedMarketData::KlineSeries(next)
        }
        NormalizedMarketData::Quote(quote) => {
            let mut next = quote.clone();
            next.source_status = source_status;
            next.source_latency_ms = now_ms.saturating_sub(next.ts_ms);
            next.data_quality.freshness_ms = now_ms.saturating_sub(next.ts_ms);
            NormalizedMarketData::Quote(next)
        }
    }
}

pub(crate) fn market_data_preview(data: &NormalizedMarketData) -> MarketDataPreview {
    match data {
        NormalizedMarketData::KlineSeries(series) => {
            let latest = series.bars.last();
            MarketDataPreview {
                latest_price: latest.map(|bar| bar.close),
                latest_bar_time: latest.map(|bar| bar.close_time_ms),
                bid_price: None,
                ask_price: None,
                ts_ms: Some(series.ts_ms),
            }
        }
        NormalizedMarketData::Quote(quote) => MarketDataPreview {
            latest_price: Some(quote.mid_price),
            latest_bar_time: None,
            bid_price: Some(quote.best_bid),
            ask_price: Some(quote.best_ask),
            ts_ms: Some(quote.ts_ms),
        },
    }
}

fn parse_okx_candles(payload: &Value, source: &DataSourceConfig) -> Result<Vec<RawKline>> {
    ensure_okx_success(payload)?;
    let rows = payload
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("OKX candles response missing data array"))?;
    let interval = source.interval.as_deref().unwrap_or("1d");
    let interval_ms = bar_interval_ms(interval);

    let mut bars = rows
        .iter()
        .filter_map(Value::as_array)
        .filter(|row| row.len() >= 9)
        .filter_map(|row| {
            let confirm = row.get(8)?.as_str().unwrap_or("1");
            if confirm != "1" {
                return None;
            }
            Some(RawKline {
                open_time: parse_u64_field(row, 0).ok()?,
                open: parse_f64_field(row, 1).ok()?,
                high: parse_f64_field(row, 2).ok()?,
                low: parse_f64_field(row, 3).ok()?,
                close: parse_f64_field(row, 4).ok()?,
                volume: parse_f64_field(row, 5).ok()?,
                close_time: parse_u64_field(row, 0).ok()?.saturating_add(interval_ms),
            })
        })
        .collect::<Vec<_>>();

    if bars.is_empty() {
        return Err(anyhow!(
            "OKX candles response did not contain confirmed bars"
        ));
    }

    bars.sort_by_key(|bar| bar.open_time);
    Ok(bars)
}

fn parse_okx_ticker(payload: &Value) -> Result<RawQuote> {
    ensure_okx_success(payload)?;
    let row = payload
        .get("data")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("OKX ticker response missing data item"))?;

    Ok(RawQuote {
        best_bid: parse_f64_value(
            row.get("bidPx")
                .ok_or_else(|| anyhow!("OKX ticker missing bidPx"))?,
        )?,
        best_ask: parse_f64_value(
            row.get("askPx")
                .ok_or_else(|| anyhow!("OKX ticker missing askPx"))?,
        )?,
        bid_size: parse_f64_value(
            row.get("bidSz")
                .ok_or_else(|| anyhow!("OKX ticker missing bidSz"))?,
        )?,
        ask_size: parse_f64_value(
            row.get("askSz")
                .ok_or_else(|| anyhow!("OKX ticker missing askSz"))?,
        )?,
        ts: parse_u64_value(
            row.get("ts")
                .ok_or_else(|| anyhow!("OKX ticker missing ts"))?,
        )?,
    })
}

fn parse_binance_klines(payload: &Value) -> Result<Vec<RawKline>> {
    let rows = payload
        .as_array()
        .ok_or_else(|| anyhow!("Binance klines response must be an array"))?;
    let mut bars = rows
        .iter()
        .filter_map(Value::as_array)
        .filter(|row| row.len() >= 7)
        .map(|row| {
            Ok(RawKline {
                open_time: row
                    .first()
                    .and_then(Value::as_u64)
                    .ok_or_else(|| anyhow!("Binance kline missing open time"))?,
                open: parse_f64_value(
                    row.get(1)
                        .ok_or_else(|| anyhow!("Binance kline missing open"))?,
                )?,
                high: parse_f64_value(
                    row.get(2)
                        .ok_or_else(|| anyhow!("Binance kline missing high"))?,
                )?,
                low: parse_f64_value(
                    row.get(3)
                        .ok_or_else(|| anyhow!("Binance kline missing low"))?,
                )?,
                close: parse_f64_value(
                    row.get(4)
                        .ok_or_else(|| anyhow!("Binance kline missing close"))?,
                )?,
                volume: parse_f64_value(
                    row.get(5)
                        .ok_or_else(|| anyhow!("Binance kline missing volume"))?,
                )?,
                close_time: row
                    .get(6)
                    .and_then(Value::as_u64)
                    .ok_or_else(|| anyhow!("Binance kline missing close time"))?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if bars.is_empty() {
        return Err(anyhow!("Binance klines response was empty"));
    }
    bars.sort_by_key(|bar| bar.open_time);
    Ok(bars)
}

fn ensure_okx_success(payload: &Value) -> Result<()> {
    let code = payload
        .get("code")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("OKX response missing code"))?;
    if code == "0" {
        return Ok(());
    }
    let msg = payload.get("msg").and_then(Value::as_str).unwrap_or("");
    Err(anyhow!("OKX API returned code {code}: {msg}"))
}

fn parse_u64_field(row: &[Value], index: usize) -> Result<u64> {
    parse_u64_value(
        row.get(index)
            .ok_or_else(|| anyhow!("OKX array row missing index {index}"))?,
    )
}

fn parse_f64_field(row: &[Value], index: usize) -> Result<f64> {
    parse_f64_value(
        row.get(index)
            .ok_or_else(|| anyhow!("OKX array row missing index {index}"))?,
    )
}

fn parse_u64_value(value: &Value) -> Result<u64> {
    value
        .as_str()
        .ok_or_else(|| anyhow!("expected string u64 value"))?
        .parse::<u64>()
        .with_context(|| format!("invalid u64 value: {value}"))
}

fn parse_f64_value(value: &Value) -> Result<f64> {
    value
        .as_str()
        .ok_or_else(|| anyhow!("expected string f64 value"))?
        .parse::<f64>()
        .with_context(|| format!("invalid f64 value: {value}"))
}

fn mock_raw_klines(source: &DataSourceConfig, now_ms: u64) -> Result<Vec<RawKline>> {
    let days = source.days.unwrap_or(150);
    let mut bars = Vec::new();
    let interval_ms = 86_400_000_u64;

    for idx in 0..days {
        let day_index = idx as f64;
        let close = match source.exchange {
            Exchange::Binance => {
                if idx < 100 {
                    42_000.0 + day_index * 20.0
                } else {
                    45_000.0 + (idx - 100) as f64 * 250.0
                }
            }
            Exchange::Okx => 42_100.0 + day_index * 22.0,
        };
        let open = close * 0.99;
        let high = close * 1.01;
        let low = close * 0.98;
        let close_time = now_ms.saturating_sub(interval_ms * (days - idx) as u64);
        let open_time = close_time.saturating_sub(interval_ms);
        bars.push(RawKline {
            open_time,
            open,
            high,
            low,
            close,
            volume: 1000.0 + idx as f64 * 10.0,
            close_time,
        });
    }

    Ok(bars)
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
    PathBuf::from("storage")
        .join("cache")
        .join("historical")
        .join(format!(
            "{:?}_{}_{}_{}_{}.json",
            source.exchange,
            binance_symbol(source.symbol.clone()).to_ascii_lowercase(),
            interval,
            days,
            match source.kind {
                DataKind::KlineSeries => "kline",
                DataKind::Quote => "quote",
            }
        ))
}

fn load_historical_cache(source: &DataSourceConfig, now_ms: u64) -> Option<Vec<NormalizedKline>> {
    let path = historical_cache_path(source);
    let body = fs::read_to_string(path).ok()?;
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
    let body = fs::read_to_string(path).ok()?;
    let cache = serde_json::from_str::<HistoricalBarsCache>(&body).ok()?;
    (!cache.bars.is_empty()).then_some(cache.bars)
}

fn persist_historical_cache(
    source: &DataSourceConfig,
    now_ms: u64,
    bars: &[NormalizedKline],
) -> Result<()> {
    let path = historical_cache_path(source);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("failed to create historical cache dir {}", parent.display())
        })?;
    }
    let body = serde_json::to_string_pretty(&HistoricalBarsCache {
        fetched_at_ms: now_ms,
        bars: bars.to_vec(),
    })?;
    fs::write(&path, body)
        .with_context(|| format!("failed to write historical cache {}", path.display()))?;
    Ok(())
}

fn fetch_historical_raw_klines(source: &DataSourceConfig) -> Result<Vec<RawKline>> {
    let endpoint = match source.exchange {
        Exchange::Okx => okx_endpoint_for_source(source),
        Exchange::Binance => binance_endpoint_for_source(source),
    };
    let client = Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .user_agent("quantpilot/0.1")
        .build()
        .expect("reqwest client should be constructible");
    let endpoint_for_reqwest = endpoint.clone();
    let payload = block_on_http(async move {
        client
            .get(&endpoint_for_reqwest)
            .send()
            .await
            .with_context(|| format!("request failed for {endpoint_for_reqwest}"))?
            .error_for_status()
            .with_context(|| format!("non-success response from {endpoint_for_reqwest}"))?
            .json::<Value>()
            .await
            .with_context(|| format!("invalid JSON from {endpoint_for_reqwest}"))
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
                    "failed to load historical replay bars for {} on {:?}",
                    source.data_id, source.exchange
                )
            })
        }
    }
}

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
}
