use crate::*;

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
