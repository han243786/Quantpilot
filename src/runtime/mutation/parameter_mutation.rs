use super::*;

#[path = "parameter_mutation/transition_lifecycle.rs"]
mod transition_lifecycle;

use transition_lifecycle::validate_runtime_parameter_mutation_boundary;
pub(crate) use transition_lifecycle::{
    activate_runtime_parameter_mutation, rollback_runtime_parameter_mutation,
};

fn runtime_parameter_mutation_record_id(
    request: &CreateRuntimeParameterMutationRequest,
    created_at_ms: u64,
    source_event_count: usize,
    proposed_parameter_version: &str,
) -> Result<String, (StatusCode, String)> {
    let digest = canonical_json_sha256_digest(&json!({
        "created_at_ms": created_at_ms,
        "source_event_count": source_event_count,
        "source_kind": request.source_kind,
        "source_id": &request.source_id,
        "target": &request.target,
        "proposed_parameter_version": proposed_parameter_version,
    }))
    .map_err(|error| internal_error(anyhow::anyhow!(error)))?;
    Ok(format!(
        "parameter_mutation_{}_{}",
        created_at_ms,
        &digest.value[..12]
    ))
}

pub(crate) async fn create_runtime_parameter_mutation(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<CreateRuntimeParameterMutationRequest>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)> {
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "parameter_mutation_boundary_violation",
            "参数变更提案需要当前能力哈希和权限边界",
            details,
        )
    })?;
    if request.source_kind != RuntimeEvidenceSourceKind::Run {
        return Err(json_bad_request(
            "bad_request",
            "参数变更提案目前需要 source_kind 为 `run`",
        ));
    }
    validate_runtime_parameter_mutation_target(&request.target)?;
    validate_runtime_parameter_mutation_boundary(&request.activation_boundary)?;
    let actor = request
        .actor
        .clone()
        .ok_or_else(|| json_bad_request("bad_request", "参数变更提案需要指定操作者"))?;
    let actor = normalize_actor_identity(Some(actor));
    let reason = request.reason.trim();
    if reason.is_empty() {
        return Err(json_bad_request("bad_request", "参数变更提案需要说明原因"));
    }

    let source = load_run_record_from_state(&state, &user_id, &request.source_id).await?;
    let old_parameter_version =
        canonical_runtime_parameter_version(&request.target, &request.old_value)?;
    let proposed_parameter_version =
        canonical_runtime_parameter_version(&request.target, &request.new_value)?;
    let is_noop = old_parameter_version == proposed_parameter_version;
    let now_ms = current_time_ms();
    let proposal_id = runtime_parameter_mutation_record_id(
        &request,
        now_ms,
        source.events.len(),
        &proposed_parameter_version,
    )?;
    let governance = runtime_parameter_mutation_governance(
        &source.governance,
        old_parameter_version.clone(),
        proposed_parameter_version.clone(),
    );
    let record = RuntimeParameterMutationRecord {
        proposal_id,
        source_kind: request.source_kind,
        source_id: request.source_id.clone(),
        graph_id: source.graph_id.clone(),
        target: request.target.clone(),
        old_value: request.old_value.clone(),
        new_value: request.new_value.clone(),
        old_parameter_version,
        proposed_parameter_version,
        status: if is_noop {
            RuntimeParameterMutationStatus::Rejected
        } else {
            RuntimeParameterMutationStatus::Proposed
        },
        rejection_reason: is_noop.then(|| "旧值和新值解析为相同的规范参数版本".to_string()),
        activation_boundary: request.activation_boundary.clone(),
        activation_state: None,
        safe_window_state: None,
        rollback_of: None,
        rollback_target_parameter_version: None,
        actor,
        reason: reason.to_string(),
        governance,
        lifecycle: Vec::new(),
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };
    let event = build_runtime_parameter_mutation_event(&record, record.status, now_ms);
    let proposal_event_governance =
        governance_with_parameter_version(&source.governance, &record.old_parameter_version);
    append_parameter_mutation_events_to_run(
        &state,
        &user_id,
        &request.source_id,
        vec![(event, proposal_event_governance)],
        None,
    )
    .await?;
    persist_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), &record)
        .await
        .map_err(io_error)?;
    state
        .evidence_metrics
        .record_mutation_proposal(record.status);
    state.parameter_mutations.write().await.insert(
        auth::scoped_key(&user_id, &record.proposal_id),
        record.clone(),
    );
    Ok(Json(record))
}

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
