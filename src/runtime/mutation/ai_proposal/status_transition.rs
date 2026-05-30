use super::*;

pub(super) fn ai_proposal_approved_status() -> RuntimeAiProposalStatus {
    // v1.2.1: 使用独立 Approved 变体区分审批通过和静态检查通过
    RuntimeAiProposalStatus::Approved
}

/// v2.1.0: 验证 AI 提案状态转换是否合法
fn is_valid_ai_proposal_transition(
    current: RuntimeAiProposalStatus,
    next: RuntimeAiProposalStatus,
) -> bool {
    use RuntimeAiProposalStatus::*;
    matches!(
        (current, next),
        (Submitted, StaticCheckPassed | StaticCheckFailed)
            | (StaticCheckPassed, Approved | Denied | Expired)
    )
}

pub(super) async fn update_ai_proposal_status(
    state: &AppState,
    user_id: &auth::UserId,
    proposal_id: &str,
    status: RuntimeAiProposalStatus,
) {
    let mut proposals = state.ai_proposals.write().await;
    let scoped = auth::scoped_key(user_id, proposal_id);
    if let Some(record) = proposals.get_mut(&scoped) {
        if !is_valid_ai_proposal_transition(record.status, status) {
            safe_eprintln!(
                "[ai_proposal] 非法状态转换: {:?} → {:?} (proposal_id={})",
                record.status,
                status,
                proposal_id
            );
            return;
        }
        record.status = status;
        record.updated_at_ms = current_time_ms();
    }
}
