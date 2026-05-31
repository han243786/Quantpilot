use crate::{
    auth, io_error, list_runtime_ai_proposal_records, load_runtime_ai_proposal_record,
    runtime::{clean_optional_filter, RuntimeAiProposalListQuery},
    AppState, RuntimeAiProposalRecord,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

pub(super) async fn load_runtime_ai_proposal_for_user(
    state: &AppState,
    user_id: &auth::UserId,
    proposal_id: &str,
) -> Result<RuntimeAiProposalRecord, (StatusCode, String)> {
    let scoped = auth::scoped_key(user_id, proposal_id);
    if let Some(record) = state.ai_proposals.read().await.get(&scoped).cloned() {
        return Ok(record);
    }
    load_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), proposal_id).await
}

pub(crate) async fn list_runtime_ai_proposals(
    State(state): State<AppState>,
    Query(query): Query<RuntimeAiProposalListQuery>,
) -> Result<Json<Vec<RuntimeAiProposalRecord>>, (StatusCode, String)> {
    let mut records = list_runtime_ai_proposal_records(&state.ai_proposal_store_dir)
        .await
        .map_err(io_error)?;
    if let Some(source_kind) = query.source_kind {
        records.retain(|record| record.source_kind == source_kind);
    }
    if let Some(source_id) = clean_optional_filter(query.source_id) {
        records.retain(|record| record.source_id == source_id);
    }
    if let Some(status) = query.status {
        records.retain(|record| record.status == status);
    }
    records.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.ai_proposal_id.cmp(&left.ai_proposal_id))
    });
    Ok(Json(records))
}

pub(crate) async fn get_runtime_ai_proposal_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(ai_proposal_id): Path<String>,
) -> Result<Json<RuntimeAiProposalRecord>, (StatusCode, String)> {
    let scoped = auth::scoped_key(&user_id, &ai_proposal_id);
    if let Some(record) = state.ai_proposals.read().await.get(&scoped).cloned() {
        return Ok(Json(record));
    }
    load_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), &ai_proposal_id)
        .await
        .map(Json)
}
