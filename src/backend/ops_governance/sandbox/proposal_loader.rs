use crate::*;

pub(super) async fn load_or_fetch_ai_proposal(
    state: &AppState,
    proposal_id: &str,
) -> Result<RuntimeAiProposalRecord, (StatusCode, String)> {
    if let Some(record) = state.ai_proposals.read().await.get(proposal_id).cloned() {
        return Ok(record);
    }
    load_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), proposal_id).await
}
