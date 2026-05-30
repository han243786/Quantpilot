use super::*;

pub(super) fn canonical_runtime_parameter_version(
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

pub(super) fn validate_runtime_parameter_mutation_target(
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

pub(super) fn runtime_mode_from_events(events: &[FrontendRuntimeEvent]) -> String {
    events
        .iter()
        .find_map(|event| {
            let mode = event.envelope.mode.trim();
            (!mode.is_empty()).then(|| mode.to_string())
        })
        .unwrap_or_else(|| "paper".to_string())
}

pub(super) fn status_contract_value(status: RuntimeParameterMutationStatus) -> &'static str {
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

pub(super) fn mutation_event_contract(
    status: RuntimeParameterMutationStatus,
) -> (&'static str, &'static str) {
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

pub(super) fn build_runtime_parameter_mutation_event(
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

pub(super) async fn append_parameter_mutation_events_to_run(
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

pub(super) fn runtime_parameter_mutation_governance(
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

pub(super) fn governance_with_parameter_version(
    governance: &RuntimeGovernanceSnapshot,
    parameter_version: &str,
) -> RuntimeGovernanceSnapshot {
    RuntimeGovernanceSnapshot {
        parameter_version: parameter_version.to_string(),
        ..governance.clone()
    }
}
