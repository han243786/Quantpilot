use crate::{FrontendRuntimeEvent, RuntimeEventEnvelope};
use serde_json::{json, Value};

pub(super) fn build_v4_backtest_output(
    artifact: &qrpc_core_ir::v4::V4BacktestArtifact,
    equity_curve: Vec<qrpc_core::BacktestEquityPoint>,
) -> qrpc_core::BacktestOutput {
    let initial_equity = equity_curve
        .first()
        .map(|point| point.equity)
        .unwrap_or_default();
    let final_equity = equity_curve
        .last()
        .map(|point| point.equity)
        .unwrap_or(initial_equity);
    let net_profit = final_equity - initial_equity;
    let total_return_ratio = if initial_equity.abs() > f64::EPSILON {
        net_profit / initial_equity
    } else {
        0.0
    };
    let win_rate = v4_win_rate_from_equity_curve(&equity_curve);
    qrpc_core::BacktestOutput {
        mode: "v4_backtest".to_string(),
        started_at_ms: artifact.started_at_ms,
        ended_at_ms: artifact.ended_at_ms,
        elapsed_ms: Some(artifact.ended_at_ms.saturating_sub(artifact.started_at_ms)),
        sessions: Vec::new(),
        equity_curve,
        benchmark_equity_curve: Vec::new(),
        period_returns: Vec::new(),
        summary: qrpc_core::BacktestSummary {
            step_count: artifact
                .input_tick_count
                .unwrap_or(artifact.input_bar_count),
            trade_count: artifact.execution_capability_sources.len(),
            total_return_ratio,
            final_equity,
            net_profit,
            win_rate,
            annualized_return: 0.0,
            annualized_volatility: 0.0,
            risk_adjusted: Default::default(),
            trade_analysis: Default::default(),
            drawdown_analysis: Default::default(),
            benchmark_comparison: None,
            skewness: 0.0,
            kurtosis: 0.0,
        },
        final_portfolio: v4_portfolio_from_artifact(artifact),
        v4_artifact: Some(artifact.clone()),
        debug_values: None,
    }
}

fn v4_win_rate_from_equity_curve(equity_curve: &[qrpc_core::BacktestEquityPoint]) -> f64 {
    let mut wins = 0_u64;
    let mut losses = 0_u64;
    for window in equity_curve.windows(2) {
        let prev = window[0].equity;
        let curr = window[1].equity;
        if !prev.is_finite() || !curr.is_finite() {
            continue;
        }
        if curr > prev {
            wins += 1;
        } else if curr < prev {
            losses += 1;
        }
    }
    let decisions = wins + losses;
    if decisions > 0 {
        wins as f64 / decisions as f64
    } else {
        0.0
    }
}

pub(super) fn v4_equity_curve_from_artifact(
    artifact: &qrpc_core_ir::v4::V4BacktestArtifact,
) -> Vec<qrpc_core::BacktestEquityPoint> {
    artifact
        .final_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.pointer("/simulated_execution/asset_curve"))
        .and_then(Value::as_array)
        .map(|points| {
            points
                .iter()
                .filter_map(|point| {
                    Some(qrpc_core::BacktestEquityPoint {
                        ts_ms: point.get("ts_ms")?.as_u64()?,
                        equity: point.get("portfolio_value")?.as_f64()?,
                        cash_balance: point.get("cash_balance")?.as_f64()?,
                        net_notional: point.get("position_market_value")?.as_f64()?,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn v4_portfolio_from_artifact(
    artifact: &qrpc_core_ir::v4::V4BacktestArtifact,
) -> qrpc_core::PortfolioState {
    let simulated = artifact
        .final_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.get("simulated_execution"));
    let cash = simulated
        .and_then(|value| value.get("cash_balance"))
        .and_then(Value::as_f64)
        .unwrap_or_default();
    let net_notional = simulated
        .and_then(|value| value.get("position_market_value"))
        .and_then(Value::as_f64)
        .unwrap_or_default();
    let mut portfolio = qrpc_core::PortfolioState::new(cash, artifact.ended_at_ms);
    portfolio.total_net_notional = net_notional;
    portfolio.total_gross_notional = net_notional.abs();
    portfolio
}

pub(super) fn frontend_events_from_v4_backtest_artifact(
    backtest_id: &str,
    artifact: &qrpc_core_ir::v4::V4BacktestArtifact,
    equity_curve: &[qrpc_core::BacktestEquityPoint],
) -> Vec<FrontendRuntimeEvent> {
    let mut events = Vec::new();
    for (index, point) in equity_curve.iter().enumerate() {
        events.push(v4_frontend_event(
            backtest_id,
            format!("{}_v4_portfolio_{}", backtest_id, index),
            "PortfolioUpdated",
            "v4.runtime",
            "v4.execution",
            point.ts_ms,
            "Info",
            format!("v4 portfolio updated equity {:.2}", point.equity),
            json!({
                "cash_balance": point.cash_balance,
                "available_cash_balance": point.cash_balance,
                "frozen_cash_balance": 0.0,
                "total_gross_notional": point.net_notional.abs(),
                "total_net_notional": point.net_notional,
                "total_leverage": 0.0,
                "positions": 0,
                "open_orders": [],
                "equity_estimate": point.equity,
                "trace_id": format!("{}_v4_portfolio_trace_{}", backtest_id, index),
                "artifact_projection": {
                    "session_index": index,
                    "cycle_name": "v4",
                    "session_started_at_ms": point.ts_ms
                }
            }),
        ));
    }
    for (index, decision) in artifact.risk_plane_decisions.iter().enumerate() {
        events.push(v4_frontend_event(
            backtest_id,
            format!("{}_v4_risk_{}", backtest_id, index),
            "RiskDecisionProduced",
            "v4.risk_plane",
            decision.target_machine_id.clone(),
            decision.ts_ms,
            if decision.approved { "Info" } else { "Warn" },
            decision.reason.clone(),
            json!({
                "status": if decision.approved { "APPROVED" } else { "REJECTED" },
                "reason_code": if decision.approved { "V4_RISK_APPROVED" } else { "V4_RISK_REJECTED" },
                "decision": decision,
                "trace_id": format!("{}_v4_risk_trace_{}", backtest_id, index),
                "artifact_projection": {
                    "session_index": index,
                    "cycle_name": "v4",
                    "session_started_at_ms": decision.ts_ms
                }
            }),
        ));
    }
    for (index, source) in artifact.execution_capability_sources.iter().enumerate() {
        events.push(v4_frontend_event(
            backtest_id,
            format!("{}_v4_execution_capability_{}", backtest_id, index),
            "ExecutionPlanned",
            "v4.execution_capability",
            source.target_machine_id.clone(),
            source.ts_ms,
            if source.accepted { "Info" } else { "Warn" },
            source.reason.clone(),
            json!({
                "orders": if source.accepted { 1 } else { 0 },
                "capability_source": source,
                "trace_id": format!("{}_v4_execution_trace_{}", backtest_id, index),
                "artifact_projection": {
                    "session_index": index,
                    "cycle_name": "v4",
                    "session_started_at_ms": source.ts_ms
                }
            }),
        ));
    }
    for (index, point) in artifact.machine_trajectory.iter().enumerate() {
        events.push(v4_frontend_event(
            backtest_id,
            format!("{}_v4_machine_{}", backtest_id, index),
            "V4MachineTrajectoryObserved",
            "v4.machine_graph",
            point.machine_id.clone(),
            point.ts_ms,
            "Info",
            format!("{} reached {}", point.machine_id, point.state_id),
            json!({
                "trajectory": point,
                "trace_id": format!("{}_v4_machine_trace_{}", backtest_id, index),
                "artifact_projection": {
                    "session_index": index,
                    "cycle_name": "v4",
                    "session_started_at_ms": point.ts_ms
                }
            }),
        ));
    }
    events.sort_by(|left, right| {
        left.event_time_ms
            .cmp(&right.event_time_ms)
            .then_with(|| left.event_id.cmp(&right.event_id))
    });
    events
}

#[allow(clippy::too_many_arguments)]
fn v4_frontend_event(
    _backtest_id: &str,
    event_id: String,
    event_type: &str,
    source_id: impl Into<String>,
    node_id: impl Into<String>,
    event_time_ms: u64,
    severity: &str,
    summary: impl Into<String>,
    payload: Value,
) -> FrontendRuntimeEvent {
    FrontendRuntimeEvent {
        event_id,
        event_type: event_type.to_string(),
        source_id: source_id.into(),
        node_id: node_id.into(),
        event_time_ms,
        severity: severity.to_string(),
        summary: summary.into(),
        payload,
        envelope: RuntimeEventEnvelope::default(),
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::backtest_execution_start::v4_projection::{
        v4_equity_curve_from_artifact, v4_win_rate_from_equity_curve,
    };

    fn point(ts_ms: u64, equity: f64) -> qrpc_core::BacktestEquityPoint {
        qrpc_core::BacktestEquityPoint {
            ts_ms,
            equity,
            cash_balance: equity,
            net_notional: 0.0,
        }
    }

    #[test]
    fn v4_win_rate_counts_up_steps_over_directional_steps() {
        let equity_curve = vec![
            point(0, 100.0),
            point(1, 110.0),
            point(2, 110.0),
            point(3, 90.0),
            point(4, 120.0),
        ];

        assert_eq!(v4_win_rate_from_equity_curve(&equity_curve), 2.0 / 3.0);
    }

    #[test]
    fn v4_equity_curve_empty_artifact_does_not_fabricate_zero_point() {
        let artifact = qrpc_core_ir::v4::V4BacktestArtifact {
            schema_version: "quantpilot/v4-backtest-artifact/v1".to_string(),
            graph_id: "graph_empty".to_string(),
            started_at_ms: 1,
            ended_at_ms: 2,
            replay_mode: "deterministic_bar_replay".to_string(),
            input_bar_count: 0,
            input_tick_count: None,
            symbols: Vec::new(),
            machine_trajectory: Vec::new(),
            risk_plane_decisions: Vec::new(),
            execution_capability_sources: Vec::new(),
            microstructure_metrics: None,
            final_snapshot: None,
        };

        assert!(v4_equity_curve_from_artifact(&artifact).is_empty());
    }
}
