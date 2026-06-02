use crate::*;

use super::{
    compute_comparison_metrics, compute_metrics_diff, compute_sandbox_warnings,
    determine_sandbox_verdict, load_or_fetch_ai_proposal,
};

mod proposal_gate;
mod report_commit;

/// 可重用的沙箱验证核心逻辑（供 API handler 和异步自动触发调用）
pub(crate) async fn run_sandbox_verification(
    state: &AppState,
    request: &RequestSandboxVerificationRequest,
) -> Result<SandboxVerificationReport, (StatusCode, String)> {
    let ai_proposal = proposal_gate::load_eligible_proposal(state, request).await?;

    let now_ms = current_time_ms();
    let sandbox_run_id = format!("sbx-run-{}", now_ms);

    let replay_days: u64 = std::env::var("QUANTPILOT_SANDBOX_REPLAY_WINDOW_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let replay_window = ReplayWindow {
        from_ts: epoch_ms_to_iso8601(now_ms.saturating_sub(replay_days * 24 * 3600 * 1000)),
        to_ts: epoch_ms_to_iso8601(now_ms),
    };

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
