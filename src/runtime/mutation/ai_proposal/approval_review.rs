use super::approval_persistence::{load_approval_from_disk, persist_approval};
use super::record_query::load_runtime_ai_proposal_for_user;
use super::sandbox_trigger::ensure_ai_proposal_can_be_approved;
use super::status_transition::{ai_proposal_approved_status, update_ai_proposal_status};
use super::RuntimeApprovalListQuery;
use crate::{
    auth, current_time_ms, io_error, json_bad_request, AppState, ApprovalActionRequest,
    RuntimeAiProposalStatus, RuntimeApprovalLifecycleEntry, RuntimeApprovalRecord,
    RuntimeApprovalReviewState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

pub(crate) async fn list_runtime_approvals(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Query(query): Query<RuntimeApprovalListQuery>,
) -> Result<Json<Vec<RuntimeApprovalRecord>>, (StatusCode, String)> {
    let prefix = auth::scoped_key(&user_id, "");
    let mut records: Vec<RuntimeApprovalRecord> = state
        .approval_records
        .read()
        .await
        .iter()
        .filter(|(key, _)| key.starts_with(&prefix))
        .map(|(_, value)| value.clone())
        .collect();
    if let Some(state_filter) = query.review_state.as_deref() {
        records.retain(|r| {
            format!("{:?}", r.review_state).to_lowercase() == state_filter.to_lowercase()
        });
    }
    records.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));
    Ok(Json(records))
}

pub(crate) async fn get_runtime_approval_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(approval_id): Path<String>,
) -> Result<Json<RuntimeApprovalRecord>, (StatusCode, String)> {
    let scoped = auth::scoped_key(&user_id, &approval_id);
    if let Some(record) = state.approval_records.read().await.get(&scoped).cloned() {
        return Ok(Json(record));
    }
    load_approval_from_disk(&state.approval_store_dir, &approval_id)
        .await
        .map(Json)
}

pub(crate) async fn approve_ai_proposal(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<ApprovalActionRequest>,
) -> Result<Json<RuntimeApprovalRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let proposal = load_runtime_ai_proposal_for_user(&state, &user_id, &proposal_id).await?;
    ensure_ai_proposal_can_be_approved(&state, &proposal).await?;
    // v1.1.2: 持有写锁完成整个读-改-写，消除 TOCTOU 竞态
    let mut approvals = state.approval_records.write().await;
    let approval = approvals
        .values()
        .find(|a| a.proposal_id == proposal_id)
        .cloned()
        .ok_or_else(|| {
            json_bad_request(
                "not_found",
                format!("提案 '{}' 的审批单不存在", proposal_id),
            )
        })?;

    if approval.review_state != RuntimeApprovalReviewState::Pending
        && approval.review_state != RuntimeApprovalReviewState::UnderReview
    {
        return Err(json_bad_request(
            "INVALID_APPROVAL_STATE",
            "审批单不在可审查状态",
        ));
    }

    let mut approval = approval;
    if !approval.reviewers_approved.contains(&request.actor_id)
        && !approval.reviewers_rejected.contains(&request.actor_id)
    {
        approval.reviewers_approved.push(request.actor_id.clone());
    }

    let required = approval.reviewers_required as usize;
    if approval.reviewers_approved.len() >= required {
        approval.review_state = RuntimeApprovalReviewState::Approved;
        approval.lifecycle.push(RuntimeApprovalLifecycleEntry {
            review_state: RuntimeApprovalReviewState::Approved,
            event_id: format!("event_approval_approved_{}", now_ms),
            sequence_no: approval.lifecycle.len() as u64 + 1,
            occurred_at_ms: now_ms,
            reason_code: "APPROVAL_APPROVED".to_string(),
            message: format!(
                "审批通过: {}/{} 审批人同意",
                approval.reviewers_approved.len(),
                required
            ),
            actor_id: Some(request.actor_id),
        });
        update_ai_proposal_status(
            &state,
            &user_id,
            &proposal_id,
            ai_proposal_approved_status(),
        )
        .await;
    } else {
        approval.review_state = RuntimeApprovalReviewState::UnderReview;
        approval.lifecycle.push(RuntimeApprovalLifecycleEntry {
            review_state: RuntimeApprovalReviewState::UnderReview,
            event_id: format!("event_approval_review_{}", now_ms),
            sequence_no: approval.lifecycle.len() as u64 + 1,
            occurred_at_ms: now_ms,
            reason_code: "APPROVAL_PARTIAL".to_string(),
            message: format!(
                "部分通过: {}/{} 审批人同意",
                approval.reviewers_approved.len(),
                required
            ),
            actor_id: Some(request.actor_id),
        });
    }

    persist_approval(&state.approval_store_dir, &approval)
        .await
        .map_err(io_error)?;
    approvals.insert(
        auth::scoped_key(&user_id, &approval.approval_id),
        approval.clone(),
    );

    Ok(Json(approval))
}

pub(crate) async fn reject_ai_proposal(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<ApprovalActionRequest>,
) -> Result<Json<RuntimeApprovalRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    // v1.1.2: 持有写锁完成整个读-改-写，消除 TOCTOU 竞态
    let mut approvals = state.approval_records.write().await;
    let mut approval = approvals
        .values()
        .find(|a| a.proposal_id == proposal_id)
        .cloned()
        .ok_or_else(|| {
            json_bad_request(
                "not_found",
                format!("提案 '{}' 的审批单不存在", proposal_id),
            )
        })?;

    if approval.review_state != RuntimeApprovalReviewState::Pending
        && approval.review_state != RuntimeApprovalReviewState::UnderReview
    {
        return Err(json_bad_request(
            "INVALID_APPROVAL_STATE",
            "审批单不在可审查状态",
        ));
    }

    approval.reviewers_rejected.push(request.actor_id.clone());
    approval.review_state = RuntimeApprovalReviewState::Rejected;
    approval.lifecycle.push(RuntimeApprovalLifecycleEntry {
        review_state: RuntimeApprovalReviewState::Rejected,
        event_id: format!("event_approval_rejected_{}", now_ms),
        sequence_no: approval.lifecycle.len() as u64 + 1,
        occurred_at_ms: now_ms,
        reason_code: "APPROVAL_REJECTED".to_string(),
        message: request.comment.unwrap_or_else(|| "审批拒绝".to_string()),
        actor_id: Some(request.actor_id),
    });
    update_ai_proposal_status(
        &state,
        &user_id,
        &proposal_id,
        RuntimeAiProposalStatus::Denied,
    )
    .await;

    persist_approval(&state.approval_store_dir, &approval)
        .await
        .map_err(io_error)?;
    approvals.insert(
        auth::scoped_key(&user_id, &approval.approval_id),
        approval.clone(),
    );

    Ok(Json(approval))
}

pub(crate) async fn claim_ai_proposal_review(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<ApprovalActionRequest>,
) -> Result<Json<RuntimeApprovalRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    // v1.1.2: 持有写锁完成整个读-改-写，消除 TOCTOU 竞态
    let mut approvals = state.approval_records.write().await;
    let mut approval = approvals
        .values()
        .find(|a| a.proposal_id == proposal_id)
        .cloned()
        .ok_or_else(|| {
            json_bad_request(
                "not_found",
                format!("提案 '{}' 的审批单不存在", proposal_id),
            )
        })?;

    if approval.review_state != RuntimeApprovalReviewState::Pending {
        return Err(json_bad_request(
            "invalid_approval_state",
            "仅待审批的提案可以被认领",
        ));
    }

    if !approval.reviewers_assigned.contains(&request.actor_id) {
        approval.reviewers_assigned.push(request.actor_id.clone());
    }
    approval.review_state = RuntimeApprovalReviewState::UnderReview;
    approval.lifecycle.push(RuntimeApprovalLifecycleEntry {
        review_state: RuntimeApprovalReviewState::UnderReview,
        event_id: format!("event_approval_claim_{}", now_ms),
        sequence_no: approval.lifecycle.len() as u64 + 1,
        occurred_at_ms: now_ms,
        reason_code: "APPROVAL_CLAIMED".to_string(),
        message: format!("审批人 {} 认领审批单", request.actor_id),
        actor_id: Some(request.actor_id),
    });
    persist_approval(&state.approval_store_dir, &approval)
        .await
        .map_err(io_error)?;
    approvals.insert(
        auth::scoped_key(&user_id, &approval.approval_id),
        approval.clone(),
    );

    Ok(Json(approval))
}
