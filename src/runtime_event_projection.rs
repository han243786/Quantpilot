use super::*;

#[derive(Clone, Copy)]
struct BacktestEventProjectionContext<'a> {
    session_index: usize,
    cycle_name: &'a str,
    session_started_at_ms: u64,
}

fn build_frontend_events(
    events: &[RuntimeEvent],
    runtime_targets: &CompileRuntimeTargets,
    projection_context: Option<BacktestEventProjectionContext<'_>>,
) -> Vec<FrontendRuntimeEvent> {
    events
        .iter()
        .map(|event| FrontendRuntimeEvent {
            event_id: event.event_id.clone(),
            event_type: runtime_event_name(&event.event_type).to_string(),
            source_id: event.source_id.clone(),
            node_id: resolve_event_node_id(
                event,
                &runtime_targets.source_to_node,
                runtime_targets
                    .runtime_node_id
                    .as_deref()
                    .unwrap_or_default(),
                runtime_targets
                    .execution_node_id
                    .as_deref()
                    .unwrap_or_default(),
            ),
            event_time_ms: event.ts_ms,
            severity: severity_for_event(&event.event_type).to_string(),
            summary: summarize_event(event),
            payload: annotate_frontend_event_payload(
                &event.payload,
                &event.trace_id,
                projection_context,
            ),
            envelope: RuntimeEventEnvelope::default(),
        })
        .collect()
}

fn annotate_frontend_event_payload(
    payload: &Value,
    trace_id: &str,
    projection_context: Option<BacktestEventProjectionContext<'_>>,
) -> Value {
    let mut object = match &payload {
        Value::Object(o) => o.clone(),
        _ => return payload.clone(),
    };

    object.insert("trace_id".to_string(), Value::String(trace_id.to_string()));
    if let Some(context) = projection_context {
        object.insert(
            "artifact_projection".to_string(),
            json!({
                "session_index": context.session_index,
                "cycle_name": context.cycle_name,
                "session_started_at_ms": context.session_started_at_ms,
            }),
        );
    }

    Value::Object(object)
}

pub(super) fn collect_frontend_events(
    session: &SessionOutput,
    runtime_targets: &CompileRuntimeTargets,
) -> Vec<FrontendRuntimeEvent> {
    let mut events =
        build_frontend_events(&session.slow_cycle.runtime_events, runtime_targets, None);
    events.extend(build_frontend_events(
        &session.fast_cycle.runtime_events,
        runtime_targets,
        None,
    ));
    events.sort_by_key(|item| item.event_time_ms);
    events
}

pub(super) fn attach_runtime_event_envelopes(
    events: &mut [FrontendRuntimeEvent],
    run_id: &str,
    mode: &str,
    governance: &RuntimeGovernanceSnapshot,
) {
    for (index, event) in events.iter_mut().enumerate() {
        attach_runtime_event_envelope(event, run_id, mode, governance, index as u64 + 1);
    }
}

pub(super) fn attach_runtime_event_envelope(
    event: &mut FrontendRuntimeEvent,
    run_id: &str,
    mode: &str,
    governance: &RuntimeGovernanceSnapshot,
    sequence_no: u64,
) {
    event.envelope = RuntimeEventEnvelope {
        event_id: event.event_id.clone(),
        event_type: event.event_type.clone(),
        stage: stage_for_frontend_event(&event.event_type),
        run_id: run_id.to_string(),
        sequence_no,
        occurred_at_ms: event.event_time_ms,
        ingested_at_ms: event.event_time_ms,
        trace_id: event
            .payload
            .get("trace_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        module_key: event.source_id.clone(),
        strategy_version: governance.strategy_version.clone(),
        parameter_version: governance.parameter_version.clone(),
        deployment_revision: governance.deployment_revision.clone(),
        capability_hash: governance.capability_hash.clone(),
        mode: mode.to_string(),
        severity: event.severity.clone(),
        retention_class: retention_class_for_frontend_event(&event.event_type),
        reason_code: event
            .payload
            .get("reason_code")
            .or_else(|| event.payload.get("status"))
            .and_then(Value::as_str)
            .map(str::to_string),
        payload_version: 1,
    };
}

pub(super) fn repair_runtime_event_envelopes(
    events: &mut [FrontendRuntimeEvent],
    run_id: &str,
    mode: &str,
    governance: &RuntimeGovernanceSnapshot,
) {
    attach_runtime_event_envelopes(events, run_id, mode, governance);
}

pub(super) fn validate_runtime_event_envelopes(
    events: &[FrontendRuntimeEvent],
    run_id: &str,
    governance: &RuntimeGovernanceSnapshot,
) -> Result<(), String> {
    let mut previous_sequence = 0;

    for (index, event) in events.iter().enumerate() {
        let envelope = &event.envelope;
        if event.event_id.trim().is_empty() {
            return Err(format!("索引 {index} 处的事件 event_id 为空"));
        }
        if event.event_type.trim().is_empty() {
            return Err(format!("事件 `{}` 的 event_type 为空", event.event_id));
        }
        if !is_known_frontend_event_type(&event.event_type) {
            return Err(format!(
                "事件 `{}` 的 event_type `{}` 未知",
                event.event_id, event.event_type
            ));
        }
        if envelope.event_id != event.event_id {
            return Err(format!(
                "事件 `{}` 的信封 event_id `{}` 不匹配",
                event.event_id, envelope.event_id
            ));
        }
        if envelope.event_type != event.event_type {
            return Err(format!(
                "事件 `{}` 的信封 event_type `{}` 不匹配",
                event.event_id, envelope.event_type
            ));
        }
        if envelope.run_id != run_id {
            return Err(format!(
                "事件 `{}` 的信封 run_id `{}` 不匹配",
                event.event_id, envelope.run_id
            ));
        }
        if envelope.sequence_no != previous_sequence + 1 {
            return Err(format!(
                "事件 `{}` 的信封 sequence_no 为 {}，期望 {}",
                event.event_id,
                envelope.sequence_no,
                previous_sequence + 1
            ));
        }
        if envelope.stage != stage_for_frontend_event(&event.event_type) {
            return Err(format!(
                "事件 `{}` 的信封 stage 与 event_type `{}` 不匹配",
                event.event_id, event.event_type
            ));
        }
        if envelope.retention_class != retention_class_for_frontend_event(&event.event_type) {
            return Err(format!(
                "事件 `{}` 的信封 retention_class 与 event_type `{}` 不匹配",
                event.event_id, event.event_type
            ));
        }
        if envelope.event_id.trim().is_empty()
            || envelope.run_id.trim().is_empty()
            || envelope.capability_hash.trim().is_empty()
            || envelope.capability_hash == "unknown"
            || envelope.deployment_revision.trim().is_empty()
            || envelope.deployment_revision == "unknown"
            || envelope.strategy_version.trim().is_empty()
            || envelope.parameter_version.trim().is_empty()
            || envelope.mode.trim().is_empty()
        {
            return Err(format!(
                "事件 `{}` 的信封治理身份不完整",
                event.event_id
            ));
        }
        if envelope.occurred_at_ms != event.event_time_ms {
            return Err(format!(
                "事件 `{}` 的信封 occurred_at_ms 与 event_time_ms 不匹配",
                event.event_id
            ));
        }
        if envelope.capability_hash != governance.capability_hash {
            return Err(format!(
                "事件 `{}` 的信封 capability_hash 与治理不匹配",
                event.event_id
            ));
        }
        if envelope.deployment_revision != governance.deployment_revision {
            return Err(format!(
                "事件 `{}` 的信封 deployment_revision 与治理不匹配",
                event.event_id
            ));
        }

        previous_sequence = envelope.sequence_no;
    }

    Ok(())
}

pub(super) fn prepend_capability_snapshot_event(
    events: &mut Vec<FrontendRuntimeEvent>,
    record_id: &str,
    runtime_mode: &str,
    occurred_at_ms: u64,
    governance: &RuntimeGovernanceSnapshot,
) {
    let event_id = format!("{record_id}_capability_snapshot");
    events.insert(
        0,
        FrontendRuntimeEvent {
            event_id: event_id.clone(),
            event_type: "CapabilitySnapshotTaken".to_string(),
            source_id: "runtime_governance".to_string(),
            node_id: "runtime".to_string(),
            event_time_ms: occurred_at_ms,
            severity: "Info".to_string(),
            summary: "Capability snapshot taken".to_string(),
            payload: json!({
                "capability_hash": governance.capability_hash,
                "schema_version": CAPABILITY_SCHEMA_VERSION,
                "permission_boundary_model_version": governance.permission_boundary.model_version,
                "chain_stages": RUNTIME_CHAIN_STAGES,
                "runtime_mode": runtime_mode,
                "trace_id": format!("{event_id}_trace"),
            }),
            envelope: RuntimeEventEnvelope::default(),
        },
    );
}

#[cfg(test)]
pub(super) fn security_violation_detected_event(
    record_id: &str,
    occurred_at_ms: u64,
    actor: Option<&ActorIdentity>,
    attempted_action: &str,
    denied_policy: &str,
    module_key: &str,
    reason_code: &str,
    governance: &RuntimeGovernanceSnapshot,
) -> FrontendRuntimeEvent {
    let event_id = format!("{record_id}_security_violation_{occurred_at_ms}");
    let mut event = FrontendRuntimeEvent {
        event_id: event_id.clone(),
        event_type: "SecurityViolationDetected".to_string(),
        source_id: module_key.to_string(),
        node_id: "runtime".to_string(),
        event_time_ms: occurred_at_ms,
        severity: "Error".to_string(),
        summary: format!("Security policy denied `{attempted_action}`"),
        payload: json!({
            "actor": actor,
            "attempted_action": attempted_action,
            "denied_policy": denied_policy,
            "module_key": module_key,
            "reason_code": reason_code,
            "trace_id": format!("{event_id}_trace"),
        }),
        envelope: RuntimeEventEnvelope::default(),
    };
    attach_runtime_event_envelopes(
        std::slice::from_mut(&mut event),
        record_id,
        "paper",
        governance,
    );
    event
}

pub(super) fn collect_frontend_events_for_backtest(
    backtest: &BacktestOutput,
    runtime_targets: &CompileRuntimeTargets,
) -> Vec<FrontendRuntimeEvent> {
    let mut events = backtest
        .sessions
        .iter()
        .enumerate()
        .flat_map(|(session_index, session)| {
            let session_started_at_ms = backtest
                .equity_curve
                .get(session_index)
                .map(|point| point.ts_ms)
                .unwrap_or(session.final_portfolio.updated_at_ms);
            let slow_context = BacktestEventProjectionContext {
                session_index,
                cycle_name: "slow",
                session_started_at_ms,
            };
            let fast_context = BacktestEventProjectionContext {
                session_index,
                cycle_name: "fast",
                session_started_at_ms,
            };
            let mut session_events = build_frontend_events(
                &session.slow_cycle.runtime_events,
                runtime_targets,
                Some(slow_context),
            );
            session_events.extend(build_frontend_events(
                &session.fast_cycle.runtime_events,
                runtime_targets,
                Some(fast_context),
            ));
            session_events
        })
        .collect::<Vec<_>>();
    events.sort_by_key(|item| item.event_time_ms);
    events
}

fn resolve_event_node_id(
    event: &RuntimeEvent,
    source_to_node: &BTreeMap<String, String>,
    runtime_node_id: &str,
    execution_node_id: &str,
) -> String {
    if let Some(node_id) = source_to_node.get(&event.source_id) {
        return node_id.clone();
    }

    match event.event_type {
        RuntimeEventType::ExecutionPlanned | RuntimeEventType::ExecutionFilled => {
            execution_node_id.to_string()
        }
        _ => runtime_node_id.to_string(),
    }
}

fn summarize_event(event: &RuntimeEvent) -> String {
    if let Some(summary) = event
        .payload
        .get("explanation_summary")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        return summary.to_string();
    }

    match event.event_type {
        RuntimeEventType::DataUpdated => format!("{} data updated", event.source_id),
        RuntimeEventType::IntentTriggered => {
            let side = event
                .payload
                .get("side")
                .and_then(Value::as_str)
                .unwrap_or("UNKNOWN");
            format!("{} triggered side={}", event.source_id, side)
        }
        RuntimeEventType::AgentDecisionProduced => {
            let side = event
                .payload
                .get("net_side")
                .and_then(Value::as_str)
                .unwrap_or("UNKNOWN");
            let strength = event
                .payload
                .get("net_strength")
                .and_then(Value::as_f64)
                .unwrap_or_default();
            format!("{} produced {} {:.4}", event.source_id, side, strength)
        }
        RuntimeEventType::RiskDecisionProduced => {
            let status = event
                .payload
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("UNKNOWN");
            format!("{} risk {}", event.source_id, status)
        }
        RuntimeEventType::ExecutionPlanned => {
            let orders = event
                .payload
                .get("orders")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            format!("execution planned with {} orders", orders)
        }
        RuntimeEventType::ExecutionFilled => {
            let qty = event
                .payload
                .get("qty")
                .and_then(Value::as_f64)
                .unwrap_or_default();
            let price = event
                .payload
                .get("price")
                .and_then(Value::as_f64)
                .unwrap_or_default();
            format!("filled qty {:.6} @ {:.2}", qty, price)
        }
        RuntimeEventType::PortfolioUpdated => {
            let equity = event
                .payload
                .get("equity_estimate")
                .and_then(Value::as_f64)
                .or_else(|| {
                    let cash = event.payload.get("cash_balance").and_then(Value::as_f64)?;
                    let exposure = event
                        .payload
                        .get("total_net_notional")
                        .and_then(Value::as_f64)
                        .unwrap_or_default();
                    Some(cash + exposure)
                })
                .unwrap_or_default();
            format!("portfolio updated equity {:.2}", equity)
        }
        RuntimeEventType::IntentEvaluated => format!("{} evaluated", event.source_id),
        RuntimeEventType::RuntimeWarning => format!("warning from {}", event.source_id),
        RuntimeEventType::RuntimeError => format!("error from {}", event.source_id),
    }
}

fn runtime_event_name(event_type: &RuntimeEventType) -> &'static str {
    match event_type {
        RuntimeEventType::DataUpdated => "DataUpdated",
        RuntimeEventType::IntentEvaluated => "IntentEvaluated",
        RuntimeEventType::IntentTriggered => "IntentTriggered",
        RuntimeEventType::AgentDecisionProduced => "AgentDecisionProduced",
        RuntimeEventType::RiskDecisionProduced => "RiskDecisionProduced",
        RuntimeEventType::ExecutionPlanned => "ExecutionPlanned",
        RuntimeEventType::ExecutionFilled => "ExecutionFilled",
        RuntimeEventType::PortfolioUpdated => "PortfolioUpdated",
        RuntimeEventType::RuntimeWarning => "RuntimeWarning",
        RuntimeEventType::RuntimeError => "RuntimeError",
    }
}

fn stage_for_frontend_event(event_type: &str) -> RuntimeEventStage {
    match event_type {
        "DataUpdated" => RuntimeEventStage::Data,
        "IntentEvaluated" | "IntentTriggered" => RuntimeEventStage::Intent,
        "AgentDecisionProduced" => RuntimeEventStage::Agent,
        "RiskDecisionProduced" => RuntimeEventStage::Risk,
        "ExecutionPlanned" => RuntimeEventStage::Execution,
        "ExecutionFilled" | "PortfolioUpdated" => RuntimeEventStage::Fill,
        "CapabilitySnapshotTaken"
        | "SecurityViolationDetected"
        | "AIProposalCreated"
        | "AIProposalDenied"
        | "AIProposalStaticCheckPassed"
        | "AIProposalStaticCheckFailed"
        | "ParameterMutationProposed"
        | "ParameterMutationRejected"
        | "ParameterMutationActivationScheduled"
        | "ParameterMutationActivated"
        | "ParameterMutationActivationFailed"
        | "ParameterMutationSafeWindowDenied"
        | "ParameterMutationRollbackScheduled"
        | "ParameterMutationRolledBack"
        | "ParameterMutationRollbackFailed" => RuntimeEventStage::System,
        _ => RuntimeEventStage::System,
    }
}

fn is_known_frontend_event_type(event_type: &str) -> bool {
    matches!(
        event_type,
        "DataUpdated"
            | "IntentEvaluated"
            | "IntentTriggered"
            | "AgentDecisionProduced"
            | "RiskDecisionProduced"
            | "ExecutionPlanned"
            | "ExecutionFilled"
            | "PortfolioUpdated"
            | "RuntimeWarning"
            | "RuntimeError"
            | "CapabilitySnapshotTaken"
            | "SecurityViolationDetected"
            | "AIProposalCreated"
            | "AIProposalDenied"
            | "AIProposalStaticCheckPassed"
            | "AIProposalStaticCheckFailed"
            | "ParameterMutationProposed"
            | "ParameterMutationRejected"
            | "ParameterMutationActivationScheduled"
            | "ParameterMutationActivated"
            | "ParameterMutationActivationFailed"
            | "ParameterMutationSafeWindowDenied"
            | "ParameterMutationRollbackScheduled"
            | "ParameterMutationRolledBack"
            | "ParameterMutationRollbackFailed"
    )
}

fn retention_class_for_frontend_event(event_type: &str) -> RuntimeEventRetentionClass {
    match event_type {
        "CapabilitySnapshotTaken"
        | "SecurityViolationDetected"
        | "AIProposalCreated"
        | "AIProposalDenied"
        | "AIProposalStaticCheckPassed"
        | "AIProposalStaticCheckFailed"
        | "ParameterMutationProposed"
        | "ParameterMutationRejected"
        | "ParameterMutationActivationScheduled"
        | "ParameterMutationActivated"
        | "ParameterMutationActivationFailed"
        | "ParameterMutationSafeWindowDenied"
        | "ParameterMutationRollbackScheduled"
        | "ParameterMutationRolledBack"
        | "ParameterMutationRollbackFailed" => RuntimeEventRetentionClass::Key,
        "ExecutionPlanned" | "ExecutionFilled" | "PortfolioUpdated" | "RiskDecisionProduced" => {
            RuntimeEventRetentionClass::Key
        }
        "DataUpdated" | "RuntimeWarning" | "RuntimeError" => RuntimeEventRetentionClass::Summary,
        _ => RuntimeEventRetentionClass::Debug,
    }
}

fn severity_for_event(event_type: &RuntimeEventType) -> &'static str {
    match event_type {
        RuntimeEventType::RuntimeError => "Error",
        RuntimeEventType::RuntimeWarning => "Warn",
        _ => "Info",
    }
}

pub(super) fn json_sse_event(name: &str, payload: impl Serialize) -> Event {
    let data = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    Event::default().event(name).data(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_stage_mapping_uses_typed_known_stages() {
        assert_eq!(
            stage_for_frontend_event("DataUpdated"),
            RuntimeEventStage::Data
        );
        assert_eq!(
            stage_for_frontend_event("RiskDecisionProduced"),
            RuntimeEventStage::Risk
        );
        assert_eq!(
            stage_for_frontend_event("PortfolioUpdated"),
            RuntimeEventStage::Fill
        );
        assert_eq!(
            stage_for_frontend_event("CapabilitySnapshotTaken"),
            RuntimeEventStage::System
        );
        assert_eq!(
            stage_for_frontend_event("SecurityViolationDetected"),
            RuntimeEventStage::System
        );
        assert_eq!(
            stage_for_frontend_event("ParameterMutationProposed"),
            RuntimeEventStage::System
        );
        assert_eq!(
            stage_for_frontend_event("ParameterMutationActivated"),
            RuntimeEventStage::System
        );
        assert_eq!(
            stage_for_frontend_event("UnexpectedEvent"),
            RuntimeEventStage::System
        );
    }

    #[test]
    fn event_retention_mapping_uses_typed_known_classes() {
        assert_eq!(
            retention_class_for_frontend_event("ExecutionFilled"),
            RuntimeEventRetentionClass::Key
        );
        assert_eq!(
            retention_class_for_frontend_event("CapabilitySnapshotTaken"),
            RuntimeEventRetentionClass::Key
        );
        assert_eq!(
            retention_class_for_frontend_event("SecurityViolationDetected"),
            RuntimeEventRetentionClass::Key
        );
        assert_eq!(
            retention_class_for_frontend_event("ParameterMutationRejected"),
            RuntimeEventRetentionClass::Key
        );
        assert_eq!(
            retention_class_for_frontend_event("ParameterMutationActivationScheduled"),
            RuntimeEventRetentionClass::Key
        );
        assert_eq!(
            retention_class_for_frontend_event("RuntimeWarning"),
            RuntimeEventRetentionClass::Summary
        );
        assert_eq!(
            retention_class_for_frontend_event("IntentEvaluated"),
            RuntimeEventRetentionClass::Debug
        );
    }

    #[test]
    fn event_envelope_serializes_typed_stage_and_retention_as_contract_strings() {
        let envelope = RuntimeEventEnvelope {
            stage: RuntimeEventStage::Risk,
            retention_class: RuntimeEventRetentionClass::Key,
            ..RuntimeEventEnvelope::default()
        };
        let value = serde_json::to_value(envelope).unwrap();

        assert_eq!(value["stage"], "risk");
        assert_eq!(value["retention_class"], "key");
    }

    fn sample_governance() -> RuntimeGovernanceSnapshot {
        RuntimeGovernanceSnapshot {
            capability_hash: "sha256:test-capability".to_string(),
            strategy_version: "strategy:v1".to_string(),
            parameter_version: "config:test".to_string(),
            deployment_revision: "sha256:test-deployment".to_string(),
            ..RuntimeGovernanceSnapshot::default()
        }
    }

    fn sample_event(event_id: &str, event_type: &str, event_time_ms: u64) -> FrontendRuntimeEvent {
        FrontendRuntimeEvent {
            event_id: event_id.to_string(),
            event_type: event_type.to_string(),
            source_id: "source.test".to_string(),
            node_id: "node.test".to_string(),
            event_time_ms,
            severity: "Info".to_string(),
            summary: "sample".to_string(),
            payload: json!({ "trace_id": format!("{event_id}_trace") }),
            envelope: RuntimeEventEnvelope::default(),
        }
    }

    #[test]
    fn runtime_event_envelope_validation_accepts_complete_events() {
        let governance = sample_governance();
        let mut events = vec![
            sample_event("event-1", "CapabilitySnapshotTaken", 100),
            sample_event("event-2", "ExecutionFilled", 101),
        ];
        attach_runtime_event_envelopes(&mut events, "run-test", "paper", &governance);

        validate_runtime_event_envelopes(&events, "run-test", &governance)
            .expect("complete envelopes should validate");
    }

    #[test]
    fn runtime_event_envelope_validation_rejects_duplicate_sequence() {
        let governance = sample_governance();
        let mut events = vec![
            sample_event("event-1", "CapabilitySnapshotTaken", 100),
            sample_event("event-2", "ExecutionFilled", 101),
        ];
        attach_runtime_event_envelopes(&mut events, "run-test", "paper", &governance);
        events[1].envelope.sequence_no = 1;

        let error = validate_runtime_event_envelopes(&events, "run-test", &governance)
            .expect_err("duplicate sequence should be rejected");
        assert!(error.contains("sequence_no"));
    }

    #[test]
    fn runtime_event_envelope_validation_rejects_missing_envelope_identity() {
        let governance = sample_governance();
        let mut events = vec![sample_event("event-1", "CapabilitySnapshotTaken", 100)];
        attach_runtime_event_envelopes(&mut events, "run-test", "paper", &governance);
        events[0].envelope.event_id.clear();

        let error = validate_runtime_event_envelopes(&events, "run-test", &governance)
            .expect_err("missing envelope id should be rejected");
        assert!(error.contains("event_id mismatch"));
    }

    #[test]
    fn runtime_event_envelope_validation_rejects_unknown_event_type() {
        let governance = sample_governance();
        let mut events = vec![sample_event("event-1", "UnexpectedEvent", 100)];
        attach_runtime_event_envelopes(&mut events, "run-test", "paper", &governance);

        let error = validate_runtime_event_envelopes(&events, "run-test", &governance)
            .expect_err("unknown event type should be rejected");
        assert!(error.contains("unknown event_type"));
    }

    #[test]
    fn runtime_event_envelope_validation_rejects_mismatched_stage_or_retention() {
        let governance = sample_governance();
        let mut events = vec![sample_event("event-1", "ExecutionFilled", 100)];
        attach_runtime_event_envelopes(&mut events, "run-test", "paper", &governance);
        events[0].envelope.stage = RuntimeEventStage::System;

        let error = validate_runtime_event_envelopes(&events, "run-test", &governance)
            .expect_err("stage mismatch should be rejected");
        assert!(error.contains("stage"));

        attach_runtime_event_envelopes(&mut events, "run-test", "paper", &governance);
        events[0].envelope.retention_class = RuntimeEventRetentionClass::Debug;
        let error = validate_runtime_event_envelopes(&events, "run-test", &governance)
            .expect_err("key retention mismatch should be rejected");
        assert!(error.contains("retention_class"));
    }

    #[test]
    fn security_violation_event_shape_has_key_system_envelope() {
        let governance = sample_governance();
        let actor = ActorIdentity {
            actor_id: "actor_test".to_string(),
            display_name: "Test Actor".to_string(),
        };
        let event = security_violation_detected_event(
            "run-test",
            123,
            Some(&actor),
            "runtime.start_live",
            "live_execution_allowed",
            "builtin.execution.paper",
            "LIVE_EXECUTION_DENIED",
            &governance,
        );

        assert_eq!(event.event_type, "SecurityViolationDetected");
        assert_eq!(event.severity, "Error");
        assert_eq!(event.payload["actor"]["actor_id"], "actor_test");
        assert_eq!(event.payload["attempted_action"], "runtime.start_live");
        assert_eq!(event.payload["denied_policy"], "live_execution_allowed");
        assert_eq!(event.payload["module_key"], "builtin.execution.paper");
        assert_eq!(event.payload["reason_code"], "LIVE_EXECUTION_DENIED");
        assert_eq!(event.envelope.stage, RuntimeEventStage::System);
        assert_eq!(
            event.envelope.retention_class,
            RuntimeEventRetentionClass::Key
        );
        assert_eq!(event.envelope.severity, "Error");
        assert_eq!(
            event.envelope.reason_code.as_deref(),
            Some("LIVE_EXECUTION_DENIED")
        );
        validate_runtime_event_envelopes(&[event], "run-test", &governance)
            .expect("security violation event envelope should validate");
    }
}
