use crate::*;

use super::{
    compute_comparison_metrics, compute_metrics_diff, compute_sandbox_warnings,
    determine_sandbox_verdict, load_or_fetch_ai_proposal,
};

mod proposal_gate;
mod replay_window;
mod report_commit;

/// 可重用的沙箱验证核心逻辑（供 API handler 和异步自动触发调用）
pub(crate) async fn run_sandbox_verification(
    state: &AppState,
    request: &RequestSandboxVerificationRequest,
) -> Result<SandboxVerificationReport, (StatusCode, String)> {
    let ai_proposal = proposal_gate::load_eligible_proposal(state, request).await?;

    let (now_ms, sandbox_run_id, replay_window) = replay_window::build_replay_window();

    let (baseline_metrics, candidate_metrics, fidelity) =
        compute_comparison_metrics(state, &ai_proposal).await?;

    let diffs = compute_metrics_diff(&baseline_metrics, &candidate_metrics);
    let verdict = determine_sandbox_verdict(&diffs);
    let warnings = compute_sandbox_warnings(&diffs, fidelity.as_str());

    let report = SandboxVerificationReport {
        proposal_id: request.proposal_id.clone(),
        sandbox_run_id,
        replay_window,
        baseline_metrics,
        candidate_metrics,
        diffs,
        verdict,
        warnings,
        replay_fidelity: fidelity,
        generated_at_ms: now_ms,
    };

    report_commit::commit_report(state, request, &report).await?;

    Ok(report)
}
