use super::*;

pub(crate) async fn activate_runtime_parameter_mutation(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<ActivateRuntimeParameterMutationRequest>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)> {
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "parameter_mutation_boundary_violation",
            "runtime parameter mutation activation requires a current capability hash and permission boundary",
            details,
        )
    })?;

    let mut record =
        load_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), &proposal_id)
            .await?;
    if !matches!(
        record.status,
        RuntimeParameterMutationStatus::Proposed | RuntimeParameterMutationStatus::SafeWindowDenied
    ) {
        return Err(json_bad_request(
            "bad_request",
            "仅 proposed 或 safe_window_denied 状态的参数变更可以激活",
        ));
    }
    let source = load_run_record_from_state(&state, &user_id, &record.source_id).await?;
    let current_sequence_no = source
        .events
        .last()
        .map(|event| event.envelope.sequence_no)
        .unwrap_or(source.events.len() as u64);
    let requested_boundary = request
        .activation_boundary
        .clone()
        .unwrap_or_else(|| record.activation_boundary.clone());
    let resolved_boundary =
        resolve_runtime_parameter_mutation_boundary(&requested_boundary, current_sequence_no)?;
    let now_ms = current_time_ms();
    let actor = request
        .actor
        .clone()
        .map(|actor| normalize_actor_identity(Some(actor)))
        .unwrap_or_else(|| record.actor.clone());
    record.actor = actor;
    let safe_window_state =
        evaluate_runtime_parameter_mutation_safe_window(request.safe_window_context.clone());
    record.safe_window_state = Some(safe_window_state.clone());
    if !safe_window_state.allowed {
        record.status = RuntimeParameterMutationStatus::SafeWindowDenied;
        record.updated_at_ms = now_ms;
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
        let denied_governance =
            governance_with_parameter_version(&source.governance, &record.old_parameter_version);
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
    record.activation_boundary = resolved_boundary.clone();
    record.activation_state = Some(RuntimeParameterMutationActivationState {
        requested_boundary: requested_boundary.clone(),
        resolved_sequence_no: resolved_boundary.resolved_sequence_no,
        scheduled_at_ms: Some(now_ms),
        activated_at_ms: None,
        active_parameter_version: None,
        failure_reason: None,
        observation_deadline_ms: Some(now_ms.saturating_add(60_000)),
    });
    record.status = RuntimeParameterMutationStatus::ActivationScheduled;
    record.updated_at_ms = now_ms;

    let schedule_event = build_runtime_parameter_mutation_event(
        &record,
        RuntimeParameterMutationStatus::ActivationScheduled,
        now_ms,
    );
    let schedule_sequence_no = current_sequence_no + 1;
    record.lifecycle.push(mutation_lifecycle_entry(
        RuntimeParameterMutationStatus::ActivationScheduled,
        &schedule_event,
        schedule_sequence_no,
        "activation scheduled at an explicit boundary",
    ));
    state
        .evidence_metrics
        .record_mutation_activation_scheduled();

    let schedule_governance =
        governance_with_parameter_version(&source.governance, &record.old_parameter_version);
    let mut events = vec![(schedule_event, schedule_governance)];
    let mut active_parameter_version = None;

    if resolved_boundary.requested == "next_cycle_start" {
        let activated_at_ms = now_ms.saturating_add(1);
        if let Some(state) = record.activation_state.as_mut() {
            state.activated_at_ms = Some(activated_at_ms);
            state.active_parameter_version = Some(record.proposed_parameter_version.clone());
        }
        record.status = RuntimeParameterMutationStatus::Activated;
        record.updated_at_ms = activated_at_ms;
        let activation_event = build_runtime_parameter_mutation_event(
            &record,
            RuntimeParameterMutationStatus::Activated,
            activated_at_ms,
        );
        let activation_sequence_no = schedule_sequence_no + 1;
        record.lifecycle.push(mutation_lifecycle_entry(
            RuntimeParameterMutationStatus::Activated,
            &activation_event,
            activation_sequence_no,
            "activation boundary reached and parameter version became active",
        ));
        let activation_governance = governance_with_parameter_version(
            &source.governance,
            &record.proposed_parameter_version,
        );
        events.push((activation_event, activation_governance));
        active_parameter_version = Some(record.proposed_parameter_version.clone());
        state
            .evidence_metrics
            .record_mutation_activation_applied(activated_at_ms.saturating_sub(now_ms));
    } else if let Some(resolved_sequence_no) = resolved_boundary.resolved_sequence_no {
        if resolved_sequence_no <= schedule_sequence_no {
            let failed_at_ms = now_ms.saturating_add(1);
            if let Some(state) = record.activation_state.as_mut() {
                state.failure_reason =
                    Some("resolved boundary is not after the scheduling event".to_string());
            }
            record.status = RuntimeParameterMutationStatus::ActivationFailed;
            record.updated_at_ms = failed_at_ms;
            let failure_event = build_runtime_parameter_mutation_event(
                &record,
                RuntimeParameterMutationStatus::ActivationFailed,
                failed_at_ms,
            );
            record.lifecycle.push(mutation_lifecycle_entry(
                RuntimeParameterMutationStatus::ActivationFailed,
                &failure_event,
                schedule_sequence_no + 1,
                "activation boundary was already behind the schedule event",
            ));
            events.push((failure_event, source.governance.clone()));
            state.evidence_metrics.record_mutation_activation_failed();
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
    // Block 5 P1-6: 参数激活后自动生成签名快照
    auto_snapshot_on_activation(&state, &user_id, &record).await;
    Ok(Json(record))
}
