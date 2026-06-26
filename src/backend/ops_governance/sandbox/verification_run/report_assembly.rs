use crate::*;

pub(super) fn build_report(
    request: &RequestSandboxVerificationRequest,
    now_ms: u64,
    sandbox_run_id: String,
    replay_window: ReplayWindow,
    baseline_metrics: SandboxMetrics,
    candidate_metrics: SandboxMetrics,
    diffs: SandboxMetricsDiff,
    verdict: SandboxVerdict,
    warnings: Vec<String>,
    fidelity: String,
) -> SandboxVerificationReport {
    SandboxVerificationReport {
        proposal_id: request.proposal_id.clone(),
        sandbox_run_id,
        replay_window,
        baseline_metrics,
        candidate_metrics,
        diffs,
        verdict,
        warnings,
        replay_fidelity: fidelity,
        productization_replay_diff: None,
        generated_at_ms: now_ms,
    }
}
