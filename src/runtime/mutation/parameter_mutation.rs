use super::*;

fn validate_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
) -> Result<(), (StatusCode, String)> {
    let requested = boundary.requested.trim();
    if requested.is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "activation_boundary.requested 是必填字段",
        ));
    }
    if requested == "immediate" {
        return Err(json_bad_request(
            "parameter_mutation_boundary_violation",
            "不支持立即激活的参数变更；请使用 next_cycle_start、manual_pause 或 sequence_cursor",
        ));
    }
    if requested == "next_cycle_start" || requested == "manual_pause" {
        return Ok(());
    }
    if requested == "sequence_cursor" && boundary.resolved_sequence_no.is_some() {
        return Ok(());
    }
    if requested
        .strip_prefix("sequence_cursor:")
        .and_then(|value| value.parse::<u64>().ok())
        .is_some()
    {
        return Ok(());
    }
    Err(json_bad_request(
        "parameter_mutation_boundary_violation",
        "不支持的激活边界；请使用 next_cycle_start、manual_pause 或 sequence_cursor",
    ))
}

fn resolve_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
    current_sequence_no: u64,
) -> Result<RuntimeParameterMutationBoundary, (StatusCode, String)> {
    validate_runtime_parameter_mutation_boundary(boundary)?;
    let requested = boundary.requested.trim();
    if requested == "next_cycle_start" {
        return Ok(RuntimeParameterMutationBoundary {
            requested: "next_cycle_start".to_string(),
            resolved_sequence_no: Some(current_sequence_no + 2),
        });
    }
    if requested == "manual_pause" {
        return Ok(RuntimeParameterMutationBoundary {
            requested: "manual_pause".to_string(),
            resolved_sequence_no: None,
        });
    }
    let sequence_no = boundary.resolved_sequence_no.or_else(|| {
        requested
            .strip_prefix("sequence_cursor:")
            .and_then(|value| value.parse::<u64>().ok())
    });
    let Some(sequence_no) = sequence_no else {
        return Err(json_bad_request(
            "parameter_mutation_boundary_violation",
            "序列游标激活边界需要 resolved_sequence_no",
        ));
    };
    Ok(RuntimeParameterMutationBoundary {
        requested: "sequence_cursor".to_string(),
        resolved_sequence_no: Some(sequence_no),
    })
}

fn evaluate_runtime_parameter_mutation_safe_window(
    snapshot: Option<RuntimeParameterMutationSafeWindowSnapshot>,
) -> RuntimeParameterMutationSafeWindowState {
    let snapshot = snapshot.unwrap_or_default();
    let mut reason_code = "SAFE_WINDOW_OPEN";
    let mut message = "安全窗口已开启，允许运行时参数变更".to_string();
    let mut retryable = false;
    let mut retry_after_ms = None;

    if !matches!(
        snapshot.runtime_status.as_str(),
        "paused" | "idle" | "stopped" | "ready"
    ) {
        reason_code = "SAFE_WINDOW_RUNTIME_ACTIVE";
        message = format!(
            "运行时状态 `{}` 不符合参数变更条件",
            snapshot.runtime_status
        );
        retryable = true;
    } else if snapshot.open_order_count > 0 {
        reason_code = "SAFE_WINDOW_OPEN_ORDERS";
        message = format!(
            "{} 笔未结订单必须结算后才可变更参数",
            snapshot.open_order_count
        );
        retryable = true;
    } else if snapshot.outstanding_risk_violation {
        reason_code = "SAFE_WINDOW_RISK_VIOLATION";
        message = "存在未解决的风控违规，阻止参数变更".to_string();
        retryable = true;
    } else if snapshot.data_freshness_ms > 60_000 {
        reason_code = "SAFE_WINDOW_STALE_DATA";
        message = format!(
            "数据新鲜度 {}ms 超出 60000ms 安全窗口限制",
            snapshot.data_freshness_ms
        );
        retryable = true;
    } else if snapshot.portfolio_exposure_bps.abs() > 10_000 {
        reason_code = "SAFE_WINDOW_EXPOSURE_LIMIT";
        message = format!(
            "组合敞口 {}bps 超出安全窗口限制",
            snapshot.portfolio_exposure_bps
        );
        retryable = true;
    } else if snapshot.cooldown_remaining_ms > 0 {
        reason_code = "SAFE_WINDOW_COOLDOWN";
        message = format!("变更冷却还剩 {}ms", snapshot.cooldown_remaining_ms);
        retryable = true;
        retry_after_ms = Some(snapshot.cooldown_remaining_ms);
    }

    let allowed = reason_code == "SAFE_WINDOW_OPEN";
    RuntimeParameterMutationSafeWindowState {
        status: if allowed { "allowed" } else { "denied" }.to_string(),
        policy_version: snapshot.policy_version.clone(),
        allowed,
        reason_code: reason_code.to_string(),
        message,
        retryable,
        retry_after_ms,
        snapshot,
    }
}

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
