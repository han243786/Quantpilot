use crate::*;

use super::load_or_fetch_ai_proposal;

pub(super) async fn load_eligible_proposal(
    state: &AppState,
    request: &RequestSandboxVerificationRequest,
) -> Result<RuntimeAiProposalRecord, (StatusCode, String)> {
    let ai_proposal = load_or_fetch_ai_proposal(state, &request.proposal_id).await?;

    if ai_proposal.status != RuntimeAiProposalStatus::StaticCheckPassed {
        return Err(json_bad_request(
            "SANDBOX_VERIFICATION_DENIED",
            "沙箱验证要求 AI 提案已通过静态检查",
        ));
    }

    Ok(ai_proposal)
}
