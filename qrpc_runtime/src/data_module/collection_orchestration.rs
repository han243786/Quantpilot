use anyhow::Result;
use qrpc_core::{RuntimeEvent, RuntimeEventType, SourceHealth};
use serde_json::json;
use std::collections::BTreeSet;

use super::{
    attach_data_quality_snapshot, build_data_quality_summary, data_sources_from_core_ir,
    market_data_preview, market_data_quality, BuiltinDataModule, DataCollectionOutput,
    DataCollectionRequest, DataModuleProvider,
};

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
