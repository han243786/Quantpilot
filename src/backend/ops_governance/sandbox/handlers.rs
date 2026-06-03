use crate::*;

// ── 沙箱验证服务 ──
// Block 5 核心技术闸门：AI 提案必须经过独立沙箱回放验证方可提交审批

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

pub(super) async fn compute_comparison_metrics(
    state: &AppState,
    ai_proposal: &RuntimeAiProposalRecord,
) -> Result<(SandboxMetrics, SandboxMetrics, String), (StatusCode, String)> {
    let backtests = state.backtests.read().await;
    let mut graph_backtests: Vec<_> = backtests
        .values()
        .filter(|b| b.graph_id == ai_proposal.graph_id)
        .collect();
    graph_backtests.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));

    if graph_backtests.len() >= 2 {
        // 真实对比：最近两个回测
        let baseline = backtest_to_sandbox_metrics(graph_backtests[1]);
        let candidate = backtest_to_sandbox_metrics(graph_backtests[0]);
        Ok((baseline, candidate, "full".to_string()))
    } else if graph_backtests.len() == 1 {
        // 仅一个回测：基线与候选相同，标记为 partial
        let metrics = backtest_to_sandbox_metrics(graph_backtests[0]);
        Ok((metrics.clone(), metrics, "partial".to_string()))
    } else {
        // 无回测数据：无法验证
        Ok((
            SandboxMetrics::default(),
            SandboxMetrics::default(),
            "partial".to_string(),
        ))
    }
}

fn backtest_to_sandbox_metrics(backtest: &BacktestRecord) -> SandboxMetrics {
    let summary = &backtest.backtest.summary;
    let total_return = summary.total_return_ratio;
    let max_drawdown = summary.drawdown_analysis.max_drawdown_ratio.max(0.001);
    SandboxMetrics {
        total_return_ratio: total_return,
        max_drawdown_ratio: max_drawdown,
        sharpe_ratio: summary.risk_adjusted.sharpe_ratio,
        win_rate: summary.win_rate,
        avg_hold_hours: 48.0,
        turnover_ratio: 0.0, // 从 BacktestSummary 移除，由 trade ledger 计算
        profit_factor: summary.trade_analysis.profit_factor,
        calmar_ratio: summary.risk_adjusted.calmar_ratio,
    }
}

pub(super) async fn load_or_fetch_ai_proposal(
    state: &AppState,
    proposal_id: &str,
) -> Result<RuntimeAiProposalRecord, (StatusCode, String)> {
    if let Some(record) = state.ai_proposals.read().await.get(proposal_id).cloned() {
        return Ok(record);
    }
    load_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), proposal_id).await
}

pub(crate) async fn load_sandbox_report_from_disk(
    store_dir: &FsPath,
    proposal_id: &str,
) -> Result<SandboxVerificationReport, (StatusCode, String)> {
    if proposal_id.contains("..")
        || proposal_id.contains('/')
        || proposal_id.contains('\\')
        || proposal_id.is_empty()
        || proposal_id.len() > 128
    {
        return Err((StatusCode::BAD_REQUEST, "proposal_id 无效".to_string()));
    }
    let file_path = store_dir.join(format!("{}.json", proposal_id));
    let json = fs::read(&file_path).await.map_err(|_| {
        json_bad_request(
            "not_found",
            format!("提案 '{}' 的沙箱报告不存在", proposal_id),
        )
    })?;
    serde_json::from_slice(&json).map_err(|error| internal_error(anyhow::anyhow!("{}", error)))
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
