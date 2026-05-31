use crate::{internal_error, runtime::runtime_simulated_v4_matrix};
use axum::http::StatusCode;
use tokio::task;

pub(super) async fn run_v4_backtest_runtime_execution(
    expanded_graph: qrpc_core_ir::v4::V4MachineGraphContract,
    symbols: &[String],
    event_type: &str,
    now_ms: u64,
    tick_replay: bool,
) -> Result<qrpc_core_ir::v4::V4BacktestArtifact, (StatusCode, String)> {
    let bars = qrpc_runtime::build_v4_deterministic_replay_bars(symbols, now_ms, event_type);
    let ticks = if tick_replay {
        bars.iter()
            .enumerate()
            .map(|(index, bar)| qrpc_runtime::V4BacktestTickInput {
                venue_id: bar.venue_id.clone(),
                symbol: bar.symbol.clone(),
                price: bar.close,
                size: 1.0,
                ts_ms: bar.ts_ms,
                sequence: index as u64,
                event_type: event_type.to_string(),
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    task::spawn_blocking(move || {
        let mut runtime = qrpc_runtime::V4PaperSimulatedRuntime::new_for_backtest(
            expanded_graph,
            runtime_simulated_v4_matrix("paper-local"),
            vec![qrpc_core_ir::v4::ExecutionCapabilityKind::Market],
        )
        .map_err(internal_error)?;
        if tick_replay {
            runtime
                .run_backtest_ticks(&ticks)
                .map_err(|error| internal_error(anyhow::anyhow!(error)))
        } else {
            runtime
                .run_backtest_bars(&bars)
                .map_err(|error| internal_error(anyhow::anyhow!(error)))
        }
    })
    .await
    .map_err(|error| internal_error(anyhow::anyhow!("v4 backtest task cancelled: {error}")))?
}
