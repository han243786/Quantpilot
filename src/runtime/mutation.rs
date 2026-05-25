fn canonical_runtime_parameter_version(
    target: &RuntimeParameterMutationTarget,
    value: &Value,
) -> Result<String, (StatusCode, String)> {
    let digest = canonical_json_sha256_digest(&json!({
        "target": target,
        "value": value,
    }))
    .map_err(|error| internal_error(anyhow::anyhow!(error)))?;
    Ok(format!("sha256:{}", digest.value))
}

fn validate_runtime_parameter_mutation_target(
    target: &RuntimeParameterMutationTarget,
) -> Result<(), (StatusCode, String)> {
    if target.node_id.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "target.node_id 是运行时参数变更提案的必填字段",
        ));
    }
    if target.module_key.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "target.module_key 是运行时参数变更提案的必填字段",
        ));
    }
    if target.parameter_path.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "target.parameter_path 是运行时参数变更提案的必填字段",
        ));
    }
    if !SUPPORTED_FRONTEND_MODULE_KEYS.contains(&target.module_key.as_str()) {
        return Err(json_bad_request(
            "capability_gated",
            format!(
                "模块 `{}` 未启用以支持运行时参数变更提案",
                target.module_key
            ),
        ));
    }
    Ok(())
}

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
        message = format!(
            "变更冷却还剩 {}ms",
            snapshot.cooldown_remaining_ms
        );
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

fn runtime_mode_from_events(events: &[FrontendRuntimeEvent]) -> String {
    events
        .iter()
        .find_map(|event| {
            let mode = event.envelope.mode.trim();
            (!mode.is_empty()).then(|| mode.to_string())
        })
        .unwrap_or_else(|| "paper".to_string())
}

fn status_contract_value(status: RuntimeParameterMutationStatus) -> &'static str {
    match status {
        RuntimeParameterMutationStatus::Proposed => "proposed",
        RuntimeParameterMutationStatus::Rejected => "rejected",
        RuntimeParameterMutationStatus::ActivationScheduled => "activation_scheduled",
        RuntimeParameterMutationStatus::Activated => "activated",
        RuntimeParameterMutationStatus::ActivationFailed => "activation_failed",
        RuntimeParameterMutationStatus::SafeWindowDenied => "safe_window_denied",
        RuntimeParameterMutationStatus::RollbackScheduled => "rollback_scheduled",
        RuntimeParameterMutationStatus::RolledBack => "rolled_back",
        RuntimeParameterMutationStatus::RollbackFailed => "rollback_failed",
    }
}

fn mutation_event_contract(status: RuntimeParameterMutationStatus) -> (&'static str, &'static str) {
    match status {
        RuntimeParameterMutationStatus::Proposed => {
            ("ParameterMutationProposed", "PARAMETER_MUTATION_PROPOSED")
        }
        RuntimeParameterMutationStatus::Rejected => {
            ("ParameterMutationRejected", "PARAMETER_MUTATION_REJECTED")
        }
        RuntimeParameterMutationStatus::ActivationScheduled => (
            "ParameterMutationActivationScheduled",
            "PARAMETER_MUTATION_ACTIVATION_SCHEDULED",
        ),
        RuntimeParameterMutationStatus::Activated => {
            ("ParameterMutationActivated", "PARAMETER_MUTATION_ACTIVATED")
        }
        RuntimeParameterMutationStatus::ActivationFailed => (
            "ParameterMutationActivationFailed",
            "PARAMETER_MUTATION_ACTIVATION_FAILED",
        ),
        RuntimeParameterMutationStatus::SafeWindowDenied => (
            "ParameterMutationSafeWindowDenied",
            "PARAMETER_MUTATION_SAFE_WINDOW_DENIED",
        ),
        RuntimeParameterMutationStatus::RollbackScheduled => (
            "ParameterMutationRollbackScheduled",
            "PARAMETER_MUTATION_ROLLBACK_SCHEDULED",
        ),
        RuntimeParameterMutationStatus::RolledBack => (
            "ParameterMutationRolledBack",
            "PARAMETER_MUTATION_ROLLED_BACK",
        ),
        RuntimeParameterMutationStatus::RollbackFailed => (
            "ParameterMutationRollbackFailed",
            "PARAMETER_MUTATION_ROLLBACK_FAILED",
        ),
    }
}

fn build_runtime_parameter_mutation_event(
    record: &RuntimeParameterMutationRecord,
    status: RuntimeParameterMutationStatus,
    event_time_ms: u64,
) -> FrontendRuntimeEvent {
    let (event_type, reason_code) = mutation_event_contract(status);
    FrontendRuntimeEvent {
        event_id: format!(
            "event_{}_{}_{}",
            record.proposal_id,
            status_contract_value(status),
            event_time_ms
        ),
        event_type: event_type.to_string(),
        source_id: record.target.module_key.clone(),
        node_id: record.target.node_id.clone(),
        event_time_ms,
        severity: match status {
            RuntimeParameterMutationStatus::Rejected
            | RuntimeParameterMutationStatus::ActivationFailed
            | RuntimeParameterMutationStatus::SafeWindowDenied
            | RuntimeParameterMutationStatus::RollbackFailed => "Warn".to_string(),
            _ => "Info".to_string(),
        },
        summary: match status {
            RuntimeParameterMutationStatus::Proposed => format!(
                "Parameter mutation proposed for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::Rejected => format!(
                "Parameter mutation rejected for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::ActivationScheduled => format!(
                "Parameter mutation activation scheduled for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::Activated => format!(
                "Parameter mutation activated for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::ActivationFailed => format!(
                "Parameter mutation activation failed for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::SafeWindowDenied => format!(
                "Parameter mutation safe window denied for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::RollbackScheduled => format!(
                "Parameter mutation rollback scheduled for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::RolledBack => format!(
                "Parameter mutation rolled back for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeParameterMutationStatus::RollbackFailed => format!(
                "Parameter mutation rollback failed for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
        },
        payload: json!({
            "proposal_id": &record.proposal_id,
            "status": status,
            "reason_code": reason_code,
            "source_kind": record.source_kind,
            "source_id": &record.source_id,
            "target": &record.target,
            "old_parameter_version": &record.old_parameter_version,
            "proposed_parameter_version": &record.proposed_parameter_version,
            "activation_boundary": &record.activation_boundary,
            "actor": &record.actor,
            "reason": &record.reason,
            "rejection_reason": &record.rejection_reason,
            "governance": &record.governance,
            "activation_state": &record.activation_state,
            "safe_window_state": &record.safe_window_state,
            "rollback_of": &record.rollback_of,
            "rollback_target_parameter_version": &record.rollback_target_parameter_version,
        }),
        envelope: RuntimeEventEnvelope::default(),
    }
}

async fn append_parameter_mutation_events_to_run(
    state: &AppState,
    user_id: &auth::UserId,
    source_id: &str,
    mut events: Vec<(FrontendRuntimeEvent, RuntimeGovernanceSnapshot)>,
    active_parameter_version: Option<String>,
) -> Result<(), (StatusCode, String)> {
    let mut record = load_run_record_from_state(state, user_id, source_id).await?;
    let mode = runtime_mode_from_events(&record.events);
    let mut next_sequence = record
        .events
        .last()
        .map(|event| event.envelope.sequence_no)
        .unwrap_or(record.events.len() as u64);
    for (event, governance) in events.iter_mut() {
        next_sequence += 1;
        attach_runtime_event_envelope(event, source_id, &mode, governance, next_sequence);
        record.events.push(event.clone());
    }
    if let Some(parameter_version) = active_parameter_version {
        record.governance.parameter_version = parameter_version;
    }
    validate_runtime_event_envelopes(&record.events, source_id, &record.governance)
        .map_err(|message| internal_error(anyhow::anyhow!(message)))?;

    state
        .runs
        .write()
        .await
        .insert(auth::scoped_key(user_id, source_id), record.clone());

    let persisted_path = state.run_store_dir.join(format!("{source_id}.json"));
    if fs::try_exists(&persisted_path).await.map_err(io_error)? {
        persist_run_record(state.run_store_dir.as_ref(), &record)
            .await
            .map_err(io_error)?;
    }

    Ok(())
}

fn runtime_parameter_mutation_governance(
    source_governance: &RuntimeGovernanceSnapshot,
    old_parameter_version: String,
    proposed_parameter_version: String,
) -> RuntimeParameterMutationGovernance {
    RuntimeParameterMutationGovernance {
        capability_hash: source_governance.capability_hash.clone(),
        deployment_revision: source_governance.deployment_revision.clone(),
        strategy_version: source_governance.strategy_version.clone(),
        previous_parameter_version: old_parameter_version,
        proposed_parameter_version,
        permission_boundary_model_version: source_governance
            .permission_boundary
            .model_version
            .clone(),
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

async fn create_runtime_parameter_mutation(
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
    let actor = request.actor.clone().ok_or_else(|| {
        json_bad_request(
            "bad_request",
            "参数变更提案需要指定操作者",
        )
    })?;
    let actor = normalize_actor_identity(Some(actor));
    let reason = request.reason.trim();
    if reason.is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "参数变更提案需要说明原因",
        ));
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
        rejection_reason: is_noop.then(|| {
            "旧值和新值解析为相同的规范参数版本".to_string()
        }),
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
    state
        .parameter_mutations
        .write()
        .await
        .insert(auth::scoped_key(&user_id, &record.proposal_id), record.clone());
    Ok(Json(record))
}

async fn list_runtime_parameter_mutations(
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
    let pq = PaginationQuery { limit: query.limit, offset: query.offset };
    Ok(Json(paginate(records, pq)))
}

async fn get_runtime_parameter_mutation_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)> {
    let scoped = auth::scoped_key(&user_id, &proposal_id);
    if let Some(record) = state
        .parameter_mutations
        .read()
        .await
        .get(&scoped)
        .cloned()
    {
        return Ok(Json(record));
    }
    load_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), &proposal_id)
        .await
        .map(Json)
}

fn validate_hash_identity(
    value: &str,
    target: &'static str,
    label: &'static str,
) -> Result<(), (StatusCode, String)> {
    let trimmed = value.trim();
    let valid = trimmed.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .chars()
                .all(|ch| ch.is_ascii_digit() || matches!(ch, 'a'..='f'))
    });
    if valid {
        Ok(())
    } else {
        Err(json_bad_request(
            "bad_request",
            format!("{target} 必须为 sha256:<64位小写十六进制> 格式 ({label})"),
        ))
    }
}

fn validate_ai_model_identity(model: &RuntimeAiModelIdentity) -> Result<(), (StatusCode, String)> {
    if model.provider.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "AI 提案候选必须指定 model.provider",
        ));
    }
    if model.model.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "AI 提案候选必须指定 model.model",
        ));
    }
    if model.model_version.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "AI 提案候选必须指定 model.model_version",
        ));
    }
    Ok(())
}

fn ai_proposal_static_check_result(
    request: &CreateRuntimeAiProposalRequest,
    old_parameter_version: &str,
    proposed_parameter_version: &str,
    source_event_count: usize,
    checked_at_ms: u64,
) -> RuntimeAiProposalStaticCheckResult {
    let mut details = Vec::new();
    if source_event_count == 0 {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "missing_source_evidence".to_string(),
            target: "source_id".to_string(),
            message: "AI proposals require at least one source evidence event".to_string(),
        });
    }
    if old_parameter_version == proposed_parameter_version {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "noop_parameter_version".to_string(),
            target: "new_value".to_string(),
            message: "旧值和新值解析为相同的规范参数版本".to_string(),
        });
    }
    if request.reason.trim().is_empty() {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "missing_reason".to_string(),
            target: "reason".to_string(),
            message: "AI 提案候选需要说明原因".to_string(),
        });
    }

    if is_v4_ai_proposal_target(&request.target)
        && request.source_kind != RuntimeEvidenceSourceKind::Backtest
    {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "v4_proposal_requires_backtest_artifact".to_string(),
            target: "source_kind".to_string(),
            message: "v4 AI proposals must be anchored to a v4 backtest artifact and machine trajectory.".to_string(),
        });
    }

    if details.is_empty() {
        RuntimeAiProposalStaticCheckResult {
            status: RuntimeAiProposalStatus::StaticCheckPassed,
            reason_code: "AI_PROPOSAL_STATIC_CHECK_PASSED".to_string(),
            message: "AI 提案候选通过静态校验".to_string(),
            checked_at_ms,
            details,
        }
    } else {
        RuntimeAiProposalStaticCheckResult {
            status: RuntimeAiProposalStatus::StaticCheckFailed,
            reason_code: "AI_PROPOSAL_STATIC_CHECK_FAILED".to_string(),
            message: "AI proposal candidate failed static validation".to_string(),
            checked_at_ms,
            details,
        }
    }
}

fn is_v4_ai_proposal_target(target: &RuntimeParameterMutationTarget) -> bool {
    target.module_key.starts_with("v4.") || target.parameter_path.starts_with("v4.")
}

#[allow(dead_code)]
fn analyze_v4_backtest_artifact_for_ai(
    artifact: &qrpc_core_ir::v4::V4BacktestArtifact,
) -> Value {
    let mut state_counts = std::collections::BTreeMap::<String, u64>::new();
    let mut machine_counts = std::collections::BTreeMap::<String, u64>::new();
    for point in &artifact.machine_trajectory {
        *state_counts
            .entry(format!("{}:{}", point.machine_id, point.state_id))
            .or_default() += 1;
        *machine_counts.entry(point.machine_id.clone()).or_default() += 1;
    }
    let risk_decision_count = artifact.risk_plane_decisions.len() as u64;
    let risk_rejected_count = artifact
        .risk_plane_decisions
        .iter()
        .filter(|decision| !decision.approved)
        .count() as u64;
    let risk_reject_ratio = if risk_decision_count == 0 {
        0.0
    } else {
        risk_rejected_count as f64 / risk_decision_count as f64
    };
    let fill_rate = artifact
        .microstructure_metrics
        .as_ref()
        .map(|metrics| metrics.fill_rate)
        .unwrap_or(0.0);

    json!({
        "analysis_version": "quantpilot/v4-ai-trajectory-analysis/v1",
        "graph_id": artifact.graph_id,
        "replay_mode": artifact.replay_mode,
        "machine_count": machine_counts.len(),
        "trajectory_point_count": artifact.machine_trajectory.len(),
        "state_visit_counts": state_counts,
        "machine_visit_counts": machine_counts,
        "risk_decision_count": risk_decision_count,
        "risk_rejected_count": risk_rejected_count,
        "risk_reject_ratio": risk_reject_ratio,
        "execution_fill_rate": fill_rate,
    })
}

fn runtime_ai_proposal_governance(
    source_governance: &RuntimeGovernanceSnapshot,
    old_parameter_version: String,
    proposed_parameter_version: String,
) -> RuntimeAiProposalGovernance {
    RuntimeAiProposalGovernance {
        capability_hash: source_governance.capability_hash.clone(),
        deployment_revision: source_governance.deployment_revision.clone(),
        strategy_version: source_governance.strategy_version.clone(),
        previous_parameter_version: old_parameter_version,
        proposed_parameter_version,
        permission_boundary_model_version: source_governance
            .permission_boundary
            .model_version
            .clone(),
        ai_write_policy: source_governance
            .permission_boundary
            .ai_write_policy
            .clone(),
    }
}

fn runtime_ai_proposal_record_id(
    request: &CreateRuntimeAiProposalRequest,
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
        "model": &request.model,
        "prompt_hash": &request.prompt_hash,
        "evidence_hash": &request.evidence_hash,
        "proposed_parameter_version": proposed_parameter_version,
    }))
    .map_err(|error| internal_error(anyhow::anyhow!(error)))?;
    Ok(format!(
        "ai_proposal_{}_{}",
        created_at_ms,
        &digest.value[..12]
    ))
}

fn ai_proposal_event_contract(status: RuntimeAiProposalStatus) -> (&'static str, &'static str) {
    match status {
        RuntimeAiProposalStatus::Submitted | RuntimeAiProposalStatus::Draft => {
            ("AIProposalCreated", "AI_PROPOSAL_CREATED")
        }
        RuntimeAiProposalStatus::Denied => ("AIProposalDenied", "AI_PROPOSAL_DENIED"),
        RuntimeAiProposalStatus::StaticCheckPassed => (
            "AIProposalStaticCheckPassed",
            "AI_PROPOSAL_STATIC_CHECK_PASSED",
        ),
        RuntimeAiProposalStatus::StaticCheckFailed => (
            "AIProposalStaticCheckFailed",
            "AI_PROPOSAL_STATIC_CHECK_FAILED",
        ),
        RuntimeAiProposalStatus::Expired => ("AIProposalDenied", "AI_PROPOSAL_EXPIRED"),
        RuntimeAiProposalStatus::Approved => ("AIProposalApproved", "AI_PROPOSAL_APPROVED"),
    }
}

fn build_runtime_ai_proposal_event(
    record: &RuntimeAiProposalRecord,
    status: RuntimeAiProposalStatus,
    event_time_ms: u64,
) -> FrontendRuntimeEvent {
    let (event_type, reason_code) = ai_proposal_event_contract(status);
    FrontendRuntimeEvent {
        event_id: format!(
            "event_{}_{}_{}",
            record.ai_proposal_id, reason_code, event_time_ms
        ),
        event_type: event_type.to_string(),
        source_id: record.target.module_key.clone(),
        node_id: record.target.node_id.clone(),
        event_time_ms,
        severity: match status {
            RuntimeAiProposalStatus::Denied | RuntimeAiProposalStatus::StaticCheckFailed => {
                "Warn".to_string()
            }
            _ => "Info".to_string(),
        },
        summary: match status {
            RuntimeAiProposalStatus::Submitted | RuntimeAiProposalStatus::Draft => format!(
                "AI proposal candidate created for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::Denied => format!(
                "AI proposal candidate denied for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::StaticCheckPassed => format!(
                "AI proposal static check passed for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::StaticCheckFailed => format!(
                "AI proposal static check failed for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::Expired => format!(
                "AI proposal expired for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
            RuntimeAiProposalStatus::Approved => format!(
                "AI proposal approved for {}.{}",
                record.target.node_id, record.target.parameter_path
            ),
        },
        payload: json!({
            "ai_proposal_id": &record.ai_proposal_id,
            "status": status,
            "reason_code": reason_code,
            "source_kind": record.source_kind,
            "source_id": &record.source_id,
            "graph_id": &record.graph_id,
            "source_evidence": &record.source_evidence,
            "target": &record.target,
            "old_parameter_version": &record.old_parameter_version,
            "proposed_parameter_version": &record.proposed_parameter_version,
            "denial_reason": &record.denial_reason,
            "static_check": &record.static_check,
            "model": &record.model,
            "prompt_hash": &record.prompt_hash,
            "evidence_hash": &record.evidence_hash,
            "actor": &record.actor,
            "reason": &record.reason,
            "governance": &record.governance,
        }),
        envelope: RuntimeEventEnvelope::default(),
    }
}

fn ai_proposal_lifecycle_entry(
    status: RuntimeAiProposalStatus,
    event: &FrontendRuntimeEvent,
    sequence_no: u64,
    message: impl Into<String>,
) -> RuntimeAiProposalLifecycleEntry {
    let (_, reason_code) = ai_proposal_event_contract(status);
    RuntimeAiProposalLifecycleEntry {
        status,
        event_id: event.event_id.clone(),
        sequence_no,
        occurred_at_ms: event.event_time_ms,
        reason_code: reason_code.to_string(),
        message: message.into(),
    }
}

async fn persist_runtime_ai_proposal_transition(
    state: &AppState,
    user_id: &auth::UserId,
    record: &RuntimeAiProposalRecord,
) -> Result<(), (StatusCode, String)> {
    persist_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), record)
        .await
        .map_err(io_error)?;
    state
        .ai_proposals
        .write()
        .await
        .insert(auth::scoped_key(user_id, &record.ai_proposal_id), record.clone());
    Ok(())
}

async fn create_runtime_ai_proposal(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<CreateRuntimeAiProposalRequest>,
) -> Result<Json<RuntimeAiProposalRecord>, (StatusCode, String)> {
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "ai_proposal_denied",
            "AI proposal candidates require a current capability hash and permission boundary",
            details,
        )
    })?;
    let capability_context = request
        .capability_context
        .as_ref()
        .expect("validated above");
    if capability_context
        .permission_boundary
        .ai_write_policy
        .trim()
        != "proposal_only"
    {
        return Err(json_bad_request(
            "ai_proposal_denied",
            "AI 写入策略必须为 proposal_only 才能创建提案",
        ));
    }
    if request.source_kind != RuntimeEvidenceSourceKind::Run {
        return Err(json_bad_request(
            "bad_request",
            "AI 提案候选目前需要 source_kind 为 `run`",
        ));
    }
    validate_runtime_parameter_mutation_target(&request.target)?;
    if request.old_value.is_null() {
        return Err(json_bad_request(
            "bad_request",
            "AI 提案候选需要旧值",
        ));
    }
    if request.new_value.is_null() {
        return Err(json_bad_request(
            "bad_request",
            "AI 提案候选需要新值",
        ));
    }
    validate_ai_model_identity(&request.model)?;
    validate_hash_identity(
        &request.prompt_hash,
        "prompt_hash",
        "AI proposal candidates",
    )?;
    validate_hash_identity(
        &request.evidence_hash,
        "evidence_hash",
        "AI proposal candidates",
    )?;
    let actor = request.actor.clone().ok_or_else(|| {
        json_bad_request(
            "bad_request",
            "AI 提案候选需要指定操作者",
        )
    })?;
    let actor = normalize_actor_identity(Some(actor));

    let source = load_run_record_from_state(&state, &user_id, &request.source_id).await?;
    let old_parameter_version =
        canonical_runtime_parameter_version(&request.target, &request.old_value)?;
    let proposed_parameter_version =
        canonical_runtime_parameter_version(&request.target, &request.new_value)?;
    let now_ms = current_time_ms();
    let static_check = ai_proposal_static_check_result(
        &request,
        &old_parameter_version,
        &proposed_parameter_version,
        source.events.len(),
        now_ms,
    );
    let status = static_check.status;
    let ai_proposal_id = runtime_ai_proposal_record_id(
        &request,
        now_ms,
        source.events.len(),
        &proposed_parameter_version,
    )?;
    let governance = runtime_ai_proposal_governance(
        &source.governance,
        old_parameter_version.clone(),
        proposed_parameter_version.clone(),
    );
    let source_evidence = RuntimeAiProposalSourceEvidence {
        source_kind: request.source_kind,
        source_id: request.source_id.clone(),
        graph_id: source.graph_id.clone(),
        event_count: source.events.len(),
        evidence_hash: request.evidence_hash.clone(),
    };
    let mut record = RuntimeAiProposalRecord {
        ai_proposal_id,
        source_kind: request.source_kind,
        source_id: request.source_id.clone(),
        graph_id: source.graph_id.clone(),
        source_evidence,
        target: request.target.clone(),
        old_value: request.old_value.clone(),
        new_value: request.new_value.clone(),
        old_parameter_version,
        proposed_parameter_version,
        status,
        denial_reason: None,
        static_check,
        model: request.model.clone(),
        prompt_hash: request.prompt_hash.clone(),
        evidence_hash: request.evidence_hash.clone(),
        actor,
        reason: request.reason.trim().to_string(),
        governance,
        lifecycle: Vec::new(),
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };

    let created_event =
        build_runtime_ai_proposal_event(&record, RuntimeAiProposalStatus::Submitted, now_ms);
    let static_event_time_ms = now_ms.saturating_add(1);
    let static_event = build_runtime_ai_proposal_event(&record, status, static_event_time_ms);
    let current_sequence_no = source
        .events
        .last()
        .map(|event| event.envelope.sequence_no)
        .unwrap_or(source.events.len() as u64);
    record.lifecycle.push(ai_proposal_lifecycle_entry(
        RuntimeAiProposalStatus::Submitted,
        &created_event,
        current_sequence_no + 1,
        "AI proposal candidate submitted",
    ));
    record.lifecycle.push(ai_proposal_lifecycle_entry(
        status,
        &static_event,
        current_sequence_no + 2,
        record.static_check.message.clone(),
    ));
    let event_governance =
        governance_with_parameter_version(&source.governance, &record.old_parameter_version);
    append_parameter_mutation_events_to_run(
        &state,
        &user_id,
        &request.source_id,
        vec![
            (created_event, event_governance.clone()),
            (static_event, event_governance),
        ],
        None,
    )
    .await?;
    // Block 5 P1-4: 静态校验通过后自动创建审批单并触发沙箱验证
    if status == RuntimeAiProposalStatus::StaticCheckPassed {
        let proposal_id = record.ai_proposal_id.clone();
        // v2.1.0: 原子计数器后缀防毫秒级 approval_id 冲突
        static APPROVAL_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let seq = APPROVAL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let approval_id = format!("apr-{}-{}", now_ms, seq);
        let approval = RuntimeApprovalRecord {
            approval_id: approval_id.clone(),
            proposal_id: proposal_id.clone(),
            approval_level: RuntimeApprovalLevel::L1SingleReviewer,
            review_state: RuntimeApprovalReviewState::Pending,
            chain_stage_impact: vec!["intent".to_string(), "agent".to_string()],
            sandbox_report_url: None,
            rollback_plan: RuntimeRollbackPlan {
                method: "generation_rollback".to_string(),
                target_generation: 0,
                estimated_recovery_ms: 5000,
            },
            created_at_ms: now_ms,
            expires_at_ms: now_ms.saturating_add(24 * 3600 * 1000), // L1: 24h
            reviewers_required: 1,
            reviewers_assigned: vec!["pm-strategy-btc".to_string()],
            reviewers_approved: Vec::new(),
            reviewers_rejected: Vec::new(),
            lifecycle: vec![RuntimeApprovalLifecycleEntry {
                review_state: RuntimeApprovalReviewState::Pending,
                event_id: format!("event_apr_pending_{}", now_ms),
                sequence_no: 1,
                occurred_at_ms: now_ms,
                reason_code: "APPROVAL_CREATED".to_string(),
                message: "审批单自动创建".to_string(),
                actor_id: None,
            }],
        };
        persist_approval(&state.approval_store_dir, &approval)
            .await
            .map_err(io_error)?;
        state
            .approval_records
            .write()
            .await
            .insert(auth::scoped_key(&user_id, &approval_id), approval);

        // GP §9.4: 与 approve 路径保持 approval_records -> ai_proposals 锁顺序。
        persist_runtime_ai_proposal_transition(&state, &user_id, &record).await?;

        // v1.1.2: 异步触发沙箱验证，JoinHandle 存入 state 防止 panic 静默丢失
        // v2.4.0 P1-B2: 添加 catch_unwind + 3次退避重试
        let state_clone = state.clone();
        let pid = proposal_id.clone();
        let handle = tokio::spawn(async move {
            let sandbox_request = RequestSandboxVerificationRequest {
                backtest_id: None,
                proposal_id: pid.clone(),
            };
            let mut success = false;
            for attempt in 0u32..3 {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(sandbox_verification::run_sandbox_verification(
                        &state_clone, &sandbox_request,
                    ))
                }));
                match result {
                    Ok(Ok(_report)) => {
                        success = true;
                        break;
                    }
                    Ok(Err(e)) => {
                        safe_eprintln!("[sandbox] 验证尝试 {}/3 失败: {}", attempt + 1, e.1);
                    }
                    Err(panic_err) => {
                        let msg = panic_err.downcast_ref::<String>()
                            .map(|s| s.as_str())
                            .or_else(|| panic_err.downcast_ref::<&str>().copied())
                            .unwrap_or("未知 panic");
                        safe_eprintln!("[sandbox] 验证尝试 {}/3 panic: {}", attempt + 1, msg);
                    }
                }
                if attempt < 2 {
                    tokio::time::sleep(std::time::Duration::from_millis(500 * (attempt + 1) as u64)).await;
                }
            }
            if success {
                // 更新审批单的沙箱报告 URL
                let approval_to_persist = {
                    let mut approvals = state_clone.approval_records.write().await;
                    let mut updated = None;
                    for approval in approvals.values_mut() {
                        if approval.proposal_id == pid {
                            approval.sandbox_report_url = Some(format!(
                                "/api/v1/ai/proposals/{}/sandbox-report",
                                pid
                            ));
                            updated = Some(approval.clone());
                            break;
                        }
                    }
                    updated
                };
                if let Some(approval) = approval_to_persist {
                    let _ = persist_approval(&state_clone.approval_store_dir, &approval).await;
                }
            } else {
                safe_eprintln!("[sandbox] 沙箱验证 3 次尝试全部失败, proposal={}", pid);
            }
        });
        // v1.1.2: 监视 JoinHandle 防止 panic 静默丢失
        tokio::spawn(async move {
            if let Err(e) = handle.await {
                safe_eprintln!("[sandbox] 沙箱验证任务异常: {}", e);
            }
        });
    }
    if status != RuntimeAiProposalStatus::StaticCheckPassed {
        persist_runtime_ai_proposal_transition(&state, &user_id, &record).await?;
    }

    Ok(Json(record))
}

async fn list_runtime_ai_proposals(
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

async fn get_runtime_ai_proposal_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(ai_proposal_id): Path<String>,
) -> Result<Json<RuntimeAiProposalRecord>, (StatusCode, String)> {
    let scoped = auth::scoped_key(&user_id, &ai_proposal_id);
    if let Some(record) = state
        .ai_proposals
        .read()
        .await
        .get(&scoped)
        .cloned()
    {
        return Ok(Json(record));
    }
    load_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), &ai_proposal_id)
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

fn governance_with_parameter_version(
    governance: &RuntimeGovernanceSnapshot,
    parameter_version: &str,
) -> RuntimeGovernanceSnapshot {
    RuntimeGovernanceSnapshot {
        parameter_version: parameter_version.to_string(),
        ..governance.clone()
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
    state
        .parameter_mutations
        .write()
        .await
        .insert(auth::scoped_key(user_id, &record.proposal_id), record.clone());
    Ok(())
}

async fn activate_runtime_parameter_mutation(
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
    crate::runtime_persistence::atomic_write_json(&path, &snapshot).await.unwrap_or_else(|e| {
        safe_eprintln!("[snapshot] 原子写入快照失败: {}", e);
    });
    state
        .snapshots
        .write()
        .await
        .insert(auth::scoped_key(user_id, &snapshot_id), snapshot);
}

async fn rollback_runtime_parameter_mutation(
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

async fn list_runtime_approvals(
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

async fn get_runtime_approval_detail(
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

async fn approve_ai_proposal(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(request): Json<ApprovalActionRequest>,
) -> Result<Json<RuntimeApprovalRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    // v1.1.2: 持有写锁完成整个读-改-写，消除 TOCTOU 竞态
    let mut approvals = state.approval_records.write().await;
    let approval = approvals
        .values()
        .find(|a| a.proposal_id == proposal_id)
        .cloned()
        .ok_or_else(|| json_bad_request(
            "not_found",
            format!("提案 '{}' 的审批单不存在", proposal_id),
        ))?;

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
        approval
            .lifecycle
            .push(RuntimeApprovalLifecycleEntry {
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
        update_ai_proposal_status(&state, &user_id, &proposal_id, ai_proposal_approved_status()).await;
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
    approvals
        .insert(auth::scoped_key(&user_id, &approval.approval_id), approval.clone());

    Ok(Json(approval))
}

async fn reject_ai_proposal(
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
        .ok_or_else(|| json_bad_request(
            "not_found",
            format!("提案 '{}' 的审批单不存在", proposal_id),
        ))?;

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
    update_ai_proposal_status(&state, &user_id, &proposal_id, RuntimeAiProposalStatus::Denied).await;

    persist_approval(&state.approval_store_dir, &approval)
        .await
        .map_err(io_error)?;
    approvals
        .insert(auth::scoped_key(&user_id, &approval.approval_id), approval.clone());

    Ok(Json(approval))
}

async fn claim_ai_proposal_review(
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
        .ok_or_else(|| json_bad_request(
            "not_found",
            format!("提案 '{}' 的审批单不存在", proposal_id),
        ))?;

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
    approvals
        .insert(auth::scoped_key(&user_id, &approval.approval_id), approval.clone());

    Ok(Json(approval))
}

fn ai_proposal_approved_status() -> RuntimeAiProposalStatus {
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

async fn update_ai_proposal_status(
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
                record.status, status, proposal_id
            );
            return;
        }
        record.status = status;
        record.updated_at_ms = current_time_ms();
    }
}

// ── Block 5: 审批辅助函数 ──

async fn persist_approval(
    store_dir: &FsPath,
    approval: &RuntimeApprovalRecord,
) -> std::io::Result<()> {
    fs::create_dir_all(store_dir).await?;
    let file_path = store_dir.join(format!("{}.json", approval.approval_id));
    // v2.3.3: 使用统一原子写入 (含 fsync)
    crate::runtime_persistence::atomic_write_json(&file_path, approval).await
}

async fn load_approval_from_disk(
    store_dir: &FsPath,
    approval_id: &str,
) -> Result<RuntimeApprovalRecord, (StatusCode, String)> {
    let file_path = store_dir.join(format!("{}.json", approval_id));
    let json = fs::read(&file_path).await.map_err(|_| {
        json_bad_request(
            "not_found",
            format!("审批单 '{}' 不存在", approval_id),
        )
    })?;
    serde_json::from_slice(&json).map_err(|error| {
        internal_error(anyhow::anyhow!("{}", error))
    })
}

// ── Block 5: 运营报表 ──

#[derive(Debug, Deserialize)]
struct OpsDailyQuery {
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuditWeeklyQuery {
    week_start: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResearchMonthlyQuery {
    month: Option<String>,
}

#[cfg(test)]
mod v4_ai_proposal_tests {
    use super::*;

    fn v4_target() -> RuntimeParameterMutationTarget {
        RuntimeParameterMutationTarget {
            node_id: "compat.execution".to_string(),
            module_key: "v4.machine.param".to_string(),
            parameter_path: "v4.machine.timeout_ms".to_string(),
        }
    }

    #[test]
    fn v4_ai_proposal_static_check_requires_backtest_source() {
        let request = CreateRuntimeAiProposalRequest {
            source_kind: RuntimeEvidenceSourceKind::Run,
            source_id: "run-1".to_string(),
            target: v4_target(),
            old_value: json!(1),
            new_value: json!(2),
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: "sha256:prompt".to_string(),
            evidence_hash: "sha256:evidence".to_string(),
            actor: None,
            reason: "Tune v4 machine timeout from trajectory evidence".to_string(),
            capability_context: None,
        };

        let result = ai_proposal_static_check_result(&request, "old", "new", 1, 1);

        assert_eq!(result.status, RuntimeAiProposalStatus::StaticCheckFailed);
        assert!(result
            .details
            .iter()
            .any(|detail| detail.code == "v4_proposal_requires_backtest_artifact"));
    }

    #[test]
    fn v4_artifact_analysis_summarizes_trajectory_and_fill_rate() {
        let artifact = qrpc_core_ir::v4::V4BacktestArtifact {
            schema_version: qrpc_core_ir::v4::V4_BACKTEST_ARTIFACT_VERSION.to_string(),
            graph_id: "graph-v4".to_string(),
            started_at_ms: 1,
            ended_at_ms: 2,
            replay_mode: "tick_replay".to_string(),
            input_bar_count: 0,
            input_tick_count: Some(2),
            symbols: vec!["BTCUSDT".to_string()],
            machine_trajectory: vec![qrpc_core_ir::v4::V4BacktestMachineTrajectoryPoint {
                ts_ms: 1,
                event_sequence: 1,
                machine_id: "compat.execution".to_string(),
                template: qrpc_core_ir::v4::MachineTemplateKind::Execution,
                state_id: "ready".to_string(),
                status: "active".to_string(),
                symbol: Some("BTCUSDT".to_string()),
            }],
            risk_plane_decisions: Vec::new(),
            execution_capability_sources: Vec::new(),
            microstructure_metrics: Some(qrpc_core_ir::v4::V4BacktestMicrostructureMetrics {
                submitted_order_count: 1,
                filled_order_count: 1,
                fill_rate: 1.0,
                average_slippage_bps: 2.0,
                queue_position_estimate: 0.0,
                vwap_deviation_bps: 2.0,
            }),
            final_snapshot: None,
        };

        let analysis = analyze_v4_backtest_artifact_for_ai(&artifact);

        assert_eq!(analysis["analysis_version"], "quantpilot/v4-ai-trajectory-analysis/v1");
        assert_eq!(analysis["machine_count"], 1);
        assert_eq!(analysis["execution_fill_rate"], 1.0);
    }
}
