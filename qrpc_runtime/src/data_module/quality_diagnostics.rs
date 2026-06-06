use qrpc_core::{
    DataQualitySnapshot, DataSourceConfig, NormalizedMarketData, SourceHealth, SourceStatus,
};

use super::{bar_interval_ms, FetchDiagnostics, QUOTE_CACHE_TTL_MS};

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

fn gap_count_for_data(data: &NormalizedMarketData) -> u64 {
    let NormalizedMarketData::KlineSeries(series) = data else {
        return 0;
    };
    if series.bars.len() < 2 {
        return 0;
    }

    let interval_ms = bar_interval_ms(&series.interval).max(1);
    let mut gap_count = 0u64;
    for pair in series.bars.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        let observed_delta = right.open_time_ms.saturating_sub(left.open_time_ms);
        if observed_delta > interval_ms {
            let missing = observed_delta / interval_ms;
            if missing > 1 {
                gap_count = gap_count.saturating_add((missing - 1) as u64);
            }
        }
    }
    gap_count
}

fn build_quality_flags(
    diagnostics: &FetchDiagnostics,
    freshness_ms: u64,
    stale_after_ms: u64,
    gap_count: u64,
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

pub(super) fn apply_snapshot_status(
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
