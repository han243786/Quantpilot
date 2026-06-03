use crate::*;

/// 从同图回测数据中选取基线和候选进行真实对比
/// 若存在多个回测，取最近两个对比；若仅一个，基线与候选使用同一数据并标记 partial
#[allow(dead_code)]
fn compare_v4_backtest_artifact_replay_shape(
    baseline: &qrpc_core_ir::v4::V4BacktestArtifact,
    candidate: &qrpc_core_ir::v4::V4BacktestArtifact,
) -> SandboxVerdict {
    let baseline_fill_rate = baseline
        .microstructure_metrics
        .as_ref()
        .map(|metrics| metrics.fill_rate)
        .unwrap_or(0.0);
    let candidate_fill_rate = candidate
        .microstructure_metrics
        .as_ref()
        .map(|metrics| metrics.fill_rate)
        .unwrap_or(0.0);
    let same_symbols = baseline.symbols == candidate.symbols;
    let trajectory_covered =
        candidate.machine_trajectory.len() >= baseline.machine_trajectory.len().saturating_div(2);
    let risk_rejections_not_worse =
        count_v4_risk_rejections(candidate) <= count_v4_risk_rejections(baseline);

    if same_symbols
        && trajectory_covered
        && risk_rejections_not_worse
        && candidate_fill_rate + f64::EPSILON >= baseline_fill_rate
    {
        SandboxVerdict::CandidateComparable
    } else {
        SandboxVerdict::CandidateUnderperforms
    }
}

#[allow(dead_code)]
fn count_v4_risk_rejections(artifact: &qrpc_core_ir::v4::V4BacktestArtifact) -> usize {
    artifact
        .risk_plane_decisions
        .iter()
        .filter(|decision| !decision.approved)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v4_artifact_replay_shape_marks_lower_fill_rate_as_underperforming() {
        let artifact = |fill_rate| qrpc_core_ir::v4::V4BacktestArtifact {
            schema_version: qrpc_core_ir::v4::V4_BACKTEST_ARTIFACT_VERSION.to_string(),
            graph_id: "graph-v4".to_string(),
            started_at_ms: 1,
            ended_at_ms: 2,
            replay_mode: "tick_replay".to_string(),
            input_bar_count: 0,
            input_tick_count: Some(2),
            symbols: vec!["BTCUSDT".to_string()],
            machine_trajectory: vec![qrpc_core_ir::v4::V4BacktestMachineTrajectoryPoint {
                ts_ms: 1,
                event_sequence: 1,
                machine_id: "compat.execution".to_string(),
                template: qrpc_core_ir::v4::MachineTemplateKind::Execution,
                state_id: "ready".to_string(),
                status: "active".to_string(),
                symbol: Some("BTCUSDT".to_string()),
            }],
            risk_plane_decisions: Vec::new(),
            execution_capability_sources: Vec::new(),
            microstructure_metrics: Some(qrpc_core_ir::v4::V4BacktestMicrostructureMetrics {
                submitted_order_count: 1,
                filled_order_count: if fill_rate > 0.0 { 1 } else { 0 },
                fill_rate,
                average_slippage_bps: 0.0,
                queue_position_estimate: 0.0,
                vwap_deviation_bps: 0.0,
            }),
            final_snapshot: None,
        };

        let verdict = compare_v4_backtest_artifact_replay_shape(&artifact(1.0), &artifact(0.0));

        assert_eq!(verdict, SandboxVerdict::CandidateUnderperforms);
    }
}
