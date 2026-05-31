use crate::{
    auth, io_error, list_runtime_parameter_mutation_records,
    load_runtime_parameter_mutation_record, paginate,
    runtime::{clean_optional_filter, RuntimeParameterMutationListQuery},
    AppState, PaginatedResponse, PaginationQuery, RuntimeParameterMutationRecord,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

pub(crate) async fn list_runtime_parameter_mutations(
    State(state): State<AppState>,
    Query(query): Query<RuntimeParameterMutationListQuery>,
) -> Result<Json<PaginatedResponse<RuntimeParameterMutationRecord>>, (StatusCode, String)> {
    let mut records = list_runtime_parameter_mutation_records(&state.mutation_store_dir)
        .await
        .map_err(io_error)?;
    if let Some(source_kind) = query.source_kind {
        records.retain(|record| record.source_kind == source_kind);
    }
    if let Some(source_id) = clean_optional_filter(query.source_id) {
        records.retain(|record| record.source_id == source_id);
    }
    records.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.proposal_id.cmp(&left.proposal_id))
    });
    let pq = PaginationQuery {
        limit: query.limit,
        offset: query.offset,
    };
    Ok(Json(paginate(records, pq)))
}

pub(crate) async fn get_runtime_parameter_mutation_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)> {
    let scoped = auth::scoped_key(&user_id, &proposal_id);
    if let Some(record) = state.parameter_mutations.read().await.get(&scoped).cloned() {
        return Ok(Json(record));
    }
    load_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), &proposal_id)
        .await
        .map(Json)
}
