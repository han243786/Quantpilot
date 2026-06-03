use crate::*;

mod backtest_projection;
mod v4_replay_shape;

pub(super) async fn compute_comparison_metrics(
    state: &AppState,
    ai_proposal: &RuntimeAiProposalRecord,
) -> Result<(SandboxMetrics, SandboxMetrics, String), (StatusCode, String)> {
    backtest_projection::compute_comparison_metrics(state, ai_proposal).await
}
