use super::*;

pub(crate) async fn rollback_runtime_parameter_mutation(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<RollbackRuntimeParameterMutationRequest>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)> {
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "parameter_mutation_boundary_violation",
            "runtime parameter mutation rollback requires a current capability hash and permission boundary",
            details,
        )
    })?;

    let original =
        load_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), &proposal_id)
            .await?;
    if original.status != RuntimeParameterMutationStatus::Activated {
        return Err(json_bad_request(
            "bad_request",
            "仅已激活的参数变更可以回滚",
        ));
    }
    state.evidence_metrics.record_mutation_rollback_attempt();

    let source = load_run_record_from_state(&state, &user_id, &original.source_id).await?;
    let target_parameter_version = request
        .target_parameter_version
        .clone()
        .unwrap_or_else(|| original.old_parameter_version.clone());

    let ledger = list_runtime_parameter_mutation_records(&state.mutation_store_dir)
        .await
        .map_err(io_error)?;
    let mut rollback_value = None;
    for item in ledger.iter() {
        if item.source_id != original.source_id || item.target != original.target {
            continue;
        }
        if item.old_parameter_version == target_parameter_version {
            rollback_value = Some(item.old_value.clone());
            break;
        }
        if item.proposed_parameter_version == target_parameter_version {
            rollback_value = Some(item.new_value.clone());
            break;
        }
    }
    let Some(new_value) = rollback_value else {
        return Err(json_bad_request(
            "parameter_mutation_rollback_unknown_version",
            "回滚目标参数版本必须在变更台账中",
        ));
    };
    if target_parameter_version == source.governance.parameter_version {
        return Err(json_bad_request(
            "parameter_mutation_rollback_noop",
            "回滚目标参数版本已是当前活跃版本",
        ));
    }

    let current_sequence_no = source
        .events
        .last()
        .map(|event| event.envelope.sequence_no)
        .unwrap_or(source.events.len() as u64);
    let requested_boundary = request
        .activation_boundary
        .clone()
        .unwrap_or_else(RuntimeParameterMutationBoundary::default);
    let resolved_boundary =
        resolve_runtime_parameter_mutation_boundary(&requested_boundary, current_sequence_no)?;
    let now_ms = current_time_ms();
    let actor = request
        .actor
        .clone()
        .map(|actor| normalize_actor_identity(Some(actor)))
        .unwrap_or_else(|| original.actor.clone());
    let reason = request
        .reason
        .clone()
        .unwrap_or_else(|| format!("Rollback {}", original.proposal_id));
    let proposal_id = runtime_parameter_mutation_rollback_record_id(
        &original.source_id,
        &original.proposal_id,
        &original.target,
        now_ms,
        source.events.len(),
        &target_parameter_version,
    )?;
    let governance = runtime_parameter_mutation_governance(
        &source.governance,
        source.governance.parameter_version.clone(),
        target_parameter_version.clone(),
    );
    let mut record = RuntimeParameterMutationRecord {
        proposal_id,
        source_kind: original.source_kind,
        source_id: original.source_id.clone(),
        graph_id: original.graph_id.clone(),
        target: original.target.clone(),
        old_value: original.new_value.clone(),
        new_value,
        old_parameter_version: source.governance.parameter_version.clone(),
        proposed_parameter_version: target_parameter_version.clone(),
        status: RuntimeParameterMutationStatus::RollbackScheduled,
        rejection_reason: None,
        activation_boundary: resolved_boundary.clone(),
        activation_state: None,
        safe_window_state: Some(evaluate_runtime_parameter_mutation_safe_window(
            request.safe_window_context.clone(),
        )),
        rollback_of: Some(original.proposal_id.clone()),
        rollback_target_parameter_version: Some(target_parameter_version.clone()),
        actor,
        reason,
        governance,
        lifecycle: Vec::new(),
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };

    if let Some(safe_window_state) = record.safe_window_state.clone() {
        if !safe_window_state.allowed {
            record.status = RuntimeParameterMutationStatus::SafeWindowDenied;
            let denied_event = build_runtime_parameter_mutation_event(
                &record,
                RuntimeParameterMutationStatus::SafeWindowDenied,
                now_ms,
            );
            let denied_sequence_no = current_sequence_no + 1;
            record.lifecycle.push(mutation_lifecycle_entry(
                RuntimeParameterMutationStatus::SafeWindowDenied,
                &denied_event,
                denied_sequence_no,
                safe_window_state.message.clone(),
            ));
            let denied_governance = governance_with_parameter_version(
                &source.governance,
                &record.old_parameter_version,
            );
            append_parameter_mutation_events_to_run(
                &state,
                &user_id,
                &record.source_id,
                vec![(denied_event, denied_governance)],
                None,
            )
            .await?;
            state.evidence_metrics.record_mutation_safe_window_denied();
            persist_runtime_parameter_mutation_transition(&state, &user_id, &record).await?;
            return Err(json_bad_request(
                "parameter_mutation_safe_window_denied",
                safe_window_state.message,
            ));
        }
    }

    record.activation_state = Some(RuntimeParameterMutationActivationState {
        requested_boundary: requested_boundary.clone(),
        resolved_sequence_no: resolved_boundary.resolved_sequence_no,
        scheduled_at_ms: Some(now_ms),
        activated_at_ms: None,
        active_parameter_version: None,
        failure_reason: None,
        observation_deadline_ms: None,
    });
    let schedule_event = build_runtime_parameter_mutation_event(
        &record,
        RuntimeParameterMutationStatus::RollbackScheduled,
        now_ms,
    );
    let schedule_sequence_no = current_sequence_no + 1;
    record.lifecycle.push(mutation_lifecycle_entry(
        RuntimeParameterMutationStatus::RollbackScheduled,
        &schedule_event,
        schedule_sequence_no,
        "rollback scheduled at an explicit boundary",
    ));
    state.evidence_metrics.record_mutation_rollback_scheduled();

    let schedule_governance =
        governance_with_parameter_version(&source.governance, &record.old_parameter_version);
    let mut events = vec![(schedule_event, schedule_governance)];
    let mut active_parameter_version = None;

    if resolved_boundary.requested == "next_cycle_start" {
        let rolled_back_at_ms = now_ms.saturating_add(1);
        if let Some(state) = record.activation_state.as_mut() {
            state.activated_at_ms = Some(rolled_back_at_ms);
            state.active_parameter_version = Some(record.proposed_parameter_version.clone());
        }
        record.status = RuntimeParameterMutationStatus::RolledBack;
        record.updated_at_ms = rolled_back_at_ms;
        let rollback_event = build_runtime_parameter_mutation_event(
            &record,
            RuntimeParameterMutationStatus::RolledBack,
            rolled_back_at_ms,
        );
        let rollback_sequence_no = schedule_sequence_no + 1;
        record.lifecycle.push(mutation_lifecycle_entry(
            RuntimeParameterMutationStatus::RolledBack,
            &rollback_event,
            rollback_sequence_no,
            "rollback boundary reached and prior parameter version became active",
        ));
        let rollback_governance = governance_with_parameter_version(
            &source.governance,
            &record.proposed_parameter_version,
        );
        events.push((rollback_event, rollback_governance));
        active_parameter_version = Some(record.proposed_parameter_version.clone());
        state.evidence_metrics.record_mutation_rollback_applied();
    } else if let Some(resolved_sequence_no) = resolved_boundary.resolved_sequence_no {
        if resolved_sequence_no <= schedule_sequence_no {
            let failed_at_ms = now_ms.saturating_add(1);
            if let Some(state) = record.activation_state.as_mut() {
                state.failure_reason = Some(
                    "resolved rollback boundary is not after the scheduling event".to_string(),
                );
            }
            record.status = RuntimeParameterMutationStatus::RollbackFailed;
            record.updated_at_ms = failed_at_ms;
            let failure_event = build_runtime_parameter_mutation_event(
                &record,
                RuntimeParameterMutationStatus::RollbackFailed,
                failed_at_ms,
            );
            record.lifecycle.push(mutation_lifecycle_entry(
                RuntimeParameterMutationStatus::RollbackFailed,
                &failure_event,
                schedule_sequence_no + 1,
                "rollback boundary was already behind the schedule event",
            ));
            events.push((failure_event, source.governance.clone()));
            state.evidence_metrics.record_mutation_rollback_failed();
        }
    }

    append_parameter_mutation_events_to_run(
        &state,
        &user_id,
        &record.source_id,
        events,
        active_parameter_version,
    )
    .await?;
    persist_runtime_parameter_mutation_transition(&state, &user_id, &record).await?;
    Ok(Json(record))
}
