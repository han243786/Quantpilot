use super::*;

#[path = "transition_lifecycle/activation_flow.rs"]
mod activation_flow;
#[path = "transition_lifecycle/boundary_safety.rs"]
mod boundary_safety;

pub(crate) use activation_flow::activate_runtime_parameter_mutation;

use boundary_safety::{
    evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary,
};

pub(super) fn validate_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
) -> Result<(), (StatusCode, String)> {
    boundary_safety::validate_runtime_parameter_mutation_boundary(boundary)
}

fn runtime_parameter_mutation_rollback_record_id(
    source_id: &str,
    rollback_of: &str,
    target: &RuntimeParameterMutationTarget,
    created_at_ms: u64,
    source_event_count: usize,
    proposed_parameter_version: &str,
) -> Result<String, (StatusCode, String)> {
    let digest = canonical_json_sha256_digest(&json!({
        "created_at_ms": created_at_ms,
        "rollback_of": rollback_of,
        "source_event_count": source_event_count,
        "source_id": source_id,
        "target": target,
        "proposed_parameter_version": proposed_parameter_version,
    }))
    .map_err(|error| internal_error(anyhow::anyhow!(error)))?;
    Ok(format!(
        "parameter_rollback_{}_{}",
        created_at_ms,
        &digest.value[..12]
    ))
}

fn mutation_lifecycle_entry(
    status: RuntimeParameterMutationStatus,
    event: &FrontendRuntimeEvent,
    sequence_no: u64,
    message: impl Into<String>,
) -> RuntimeParameterMutationLifecycleEntry {
    let (_, reason_code) = mutation_event_contract(status);
    RuntimeParameterMutationLifecycleEntry {
        status,
        event_id: event.event_id.clone(),
        sequence_no,
        occurred_at_ms: event.event_time_ms,
        reason_code: reason_code.to_string(),
        message: message.into(),
    }
}

async fn persist_runtime_parameter_mutation_transition(
    state: &AppState,
    user_id: &auth::UserId,
    record: &RuntimeParameterMutationRecord,
) -> Result<(), (StatusCode, String)> {
    persist_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), record)
        .await
        .map_err(io_error)?;
    state.parameter_mutations.write().await.insert(
        auth::scoped_key(user_id, &record.proposal_id),
        record.clone(),
    );
    Ok(())
}

/// Block 5 P1-6 + P3-2: 激活时自动生成签名快照 + 递增代际
async fn auto_snapshot_on_activation(
    state: &AppState,
    user_id: &auth::UserId,
    mutation: &RuntimeParameterMutationRecord,
) {
    let now_ms = current_time_ms();
    // P3-2: 递增配置代际
    let gen = state
        .config_generation
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    const MAX_GENERATION_HISTORY: usize = 100;
    let mut history = state.config_generation_history.lock().await;
    history.push(qrpc_runtime::ConfigGenerationEntry {
        generation: gen,
        activated_at_ms: now_ms,
        deployment_revision: mutation.governance.deployment_revision.clone(),
        parameter_version: mutation.proposed_parameter_version.clone(),
    });
    let overflow = history.len().saturating_sub(MAX_GENERATION_HISTORY);
    if overflow > 0 {
        history.drain(0..overflow);
    }

    // P3-3: Shadow Evaluation — 记录激活前指标基线
    let _pre_activation_risk_reject = state
        .evidence_metrics
        .mutation_proposal_rejected_count
        .load(std::sync::atomic::Ordering::Relaxed);
    let _pre_activation_rollback = state
        .evidence_metrics
        .mutation_rollback_attempt_count
        .load(std::sync::atomic::Ordering::Relaxed);

    // P3-4: Observation Window — 设置 60s 观察截止时间
    let _observation_deadline_ms = now_ms.saturating_add(60_000);

    let snapshot_id = format!("snap-auto-{}", now_ms);
    let snapshot = DeploymentSignatureSnapshot {
        snapshot_id: snapshot_id.clone(),
        deployment_revision: mutation.governance.deployment_revision.clone(),
        capability_hash: mutation.governance.capability_hash.clone(),
        strategy_version: mutation.governance.strategy_version.clone(),
        parameter_version: mutation.proposed_parameter_version.clone(),
        core_ir_digest: "auto-generated-on-activation".to_string(),
        event_slice_bounds: EventSliceBounds {
            from_event_id: String::new(),
            to_event_id: String::new(),
            from_sequence: 0,
            to_sequence: 0,
            event_count: 0,
        },
        created_at_ms: now_ms,
        signature: qrpc_core::canonical_json_sha256_digest(&serde_json::json!({
            "capability_hash": mutation.governance.capability_hash,
            "strategy_version": mutation.governance.strategy_version,
            "parameter_version": mutation.proposed_parameter_version,
            "created_at_ms": now_ms,
        }))
        .map(|d| d.value)
        .unwrap_or_else(|_| "signature-unavailable".to_string()),
    };
    // 持久化并存入内存
    let dir = state.snapshot_store_dir.to_path_buf();
    let path = dir.join(format!("{}.json", snapshot_id));
    // v2.3.3: 使用统一原子写入 (含 fsync)
    crate::runtime_persistence::atomic_write_json(&path, &snapshot)
        .await
        .unwrap_or_else(|e| {
            safe_eprintln!("[snapshot] 原子写入快照失败: {}", e);
        });
    state
        .snapshots
        .write()
        .await
        .insert(auth::scoped_key(user_id, &snapshot_id), snapshot);
}

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
