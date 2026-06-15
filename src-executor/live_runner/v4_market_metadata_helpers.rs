use qrpc_core::Symbol;

use super::V4Runner;

pub(super) fn executor_v4_market_matrix(
    venue_id: impl Into<String>,
) -> qrpc_core_ir::v4::VenueCapabilityMatrix {
    let mut matrix = qrpc_core_ir::v4::unsupported_v4_first_wave_matrix(venue_id);
    for entry in &mut matrix.capabilities {
        if matches!(
            entry.capability,
            qrpc_core_ir::v4::ExecutionCapabilityKind::Market
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Gtc
                | qrpc_core_ir::v4::ExecutionCapabilityKind::ClientOrderId
        ) {
            entry.source = qrpc_core_ir::v4::CapabilitySupportSource::RuntimeSimulated;
            entry.supported_modes = vec![qrpc_core_ir::v4::RuntimeTradingMode::PaperSimulated];
        }
    }
    matrix
}

pub(super) fn resolve_v4_runner_venue_id(
    graph: &qrpc_core_ir::v4::V4MachineGraphContract,
) -> String {
    graph_metadata_string(graph, "default_venue_id")
        .or_else(|| graph_metadata_string(graph, "core_venue_kind"))
        .unwrap_or_else(|| V4Runner::DEFAULT_REALTIME_PAPER_VENUE_ID.to_string())
}

pub(super) fn resolve_v4_runner_default_symbol(
    graph: &qrpc_core_ir::v4::V4MachineGraphContract,
    subscribed_symbols: &[Symbol],
) -> String {
    graph_metadata_string(graph, "default_symbol")
        .or_else(|| {
            graph
                .metadata
                .get("symbols")
                .and_then(|value| value.as_array())
                .and_then(|symbols| symbols.first())
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
        .or_else(|| {
            subscribed_symbols
                .first()
                .map(|symbol| symbol.as_str().to_string())
        })
        .unwrap_or_else(|| "BTCUSDT".to_string())
}

fn graph_metadata_string(
    graph: &qrpc_core_ir::v4::V4MachineGraphContract,
    key: &str,
) -> Option<String> {
    graph
        .metadata
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
