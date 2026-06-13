use super::*;

#[test]
fn v4_runtime_rejects_order_when_request_uses_undeclared_capability() {
    let mut runtime = sample_runtime();
    let mut graph = sample_compat_graph();
    graph.metadata.insert(
        "default_symbol".to_string(),
        Value::String("BTCUSDT".to_string()),
    );
    runtime.graph = graph;

    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    assert_eq!(
        output
            .memory_snapshot
            .simulated_execution
            .rejected_order_count,
        0
    );

    let request = V4SimulatedOrderRequest {
        order_id: Some("bad-limit".to_string()),
        client_order_id: None,
        venue_id: "paper-local".to_string(),
        symbol: "BTCUSDT".to_string(),
        action: V4SimulatedPositionAction::Buy,
        order_type: V4SimulatedOrderType::Limit,
        quantity: 1.0,
        reference_price: 100.0,
        limit_price: Some(99.0),
        trigger_price: None,
        take_profit_price: None,
        stop_loss_price: None,
        trailing_offset_bps: None,
        expire_at_ms: None,
        time_in_force: None,
        post_only: false,
        reduce_only: false,
        close_only: false,
        allow_partial_fill: true,
        fee_bps: 10.0,
        slippage_bps: 0.0,
        max_fill_quantity: None,
    };
    let reason = runtime
        .validate_simulated_order_capabilities(&request)
        .unwrap_err();
    assert!(reason.contains("Limit"));
}

#[test]
fn v4_runtime_rejects_live_simulated_mode_until_supported() {
    let bridge_report = bridge_core_ir_to_v4_machine_graph(&sample_core_ir_for_v4_runtime());
    let error = V4PaperSimulatedRuntime::new_for_mode(
        bridge_report.graph.unwrap(),
        RuntimeTradingMode::LiveSimulated,
    )
    .unwrap_err();

    assert!(error.to_string().contains("只允许 PaperSimulated 模式"));
}

#[test]
fn v4_live_actual_boundary_allows_provider_actual_mode_but_keeps_submit_detached() {
    let runtime = V4Runtime::new_for_mode_with_execution_capabilities(
        sample_compat_graph(),
        RuntimeTradingMode::LiveActual,
        provider_native_market_matrix_for_live_actual(),
        vec![ExecutionCapabilityKind::Market],
    )
    .unwrap();
    let snapshot = runtime.memory_snapshot(1);

    assert_eq!(snapshot.runtime_mode, RuntimeTradingMode::LiveActual);
    assert!(
        snapshot
            .venue_adapter_boundary
            .provider_order_submission_allowed
    );
    assert!(
        !snapshot
            .venue_adapter_boundary
            .provider_order_submission_attached
    );
    assert_eq!(
        snapshot.venue_adapter_boundary.settlement_authority,
        RuntimeSettlementAuthority::ProviderActual
    );
}

#[test]
fn v4_live_actual_rejects_runtime_simulated_capability_source() {
    let mut runtime = V4Runtime::new_for_mode_with_execution_capabilities(
        sample_compat_graph(),
        RuntimeTradingMode::LiveActual,
        runtime_simulated_market_matrix_for_live_actual(),
        vec![ExecutionCapabilityKind::Market],
    )
    .unwrap();

    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_CAPABILITY_REJECTED));
    assert_eq!(output.memory_snapshot.execution.accepted_count, 0);
    assert_eq!(output.memory_snapshot.execution.rejected_count, 1);
    assert!(output
        .memory_snapshot
        .execution
        .last_decision
        .as_ref()
        .unwrap()
        .reason
        .contains("provider_native"));
}

#[test]
fn v4_live_actual_without_risk_plane_is_rejected() {
    let mut graph = sample_compat_graph();
    graph.risk_plane = None;
    let error = V4Runtime::new_for_mode(graph, RuntimeTradingMode::LiveActual).unwrap_err();

    assert!(error.to_string().contains("Risk Plane"));
}

#[test]
fn v4_live_actual_without_execution_capability_policy_is_rejected() {
    let error =
        V4Runtime::new_for_mode(sample_compat_graph(), RuntimeTradingMode::LiveActual).unwrap_err();

    assert!(error.to_string().contains("capability policy"));
}

#[test]
fn v4_runtime_rejects_unsupported_execution_capability_before_execution() {
    let mut runtime = V4PaperSimulatedRuntime::new_with_execution_capabilities(
        sample_compat_graph(),
        unsupported_v4_first_wave_matrix("paper-local"),
        vec![ExecutionCapabilityKind::Market],
    )
    .unwrap();

    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    assert_eq!(
        runtime.machine_state_id(V4_COMPAT_EXECUTION_MACHINE_ID),
        Some("idle")
    );
    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_CAPABILITY_REJECTED));
    assert_eq!(output.memory_snapshot.execution.accepted_count, 0);
    assert_eq!(output.memory_snapshot.execution.rejected_count, 1);
    let decision = output
        .memory_snapshot
        .execution
        .last_decision
        .as_ref()
        .unwrap();
    assert!(!decision.accepted);
    assert_eq!(
        decision.entries[0].status,
        V4ExecutionCapabilityRuntimeStatus::Unsupported
    );
}

#[test]
fn v4_runtime_rejects_provider_native_capability_in_paper_simulated() {
    let mut runtime = V4PaperSimulatedRuntime::new_with_execution_capabilities(
        sample_compat_graph(),
        provider_native_market_matrix_for_paper(),
        vec![ExecutionCapabilityKind::Market],
    )
    .unwrap();

    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    assert_eq!(
        runtime.machine_state_id(V4_COMPAT_EXECUTION_MACHINE_ID),
        Some("idle")
    );
    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_CAPABILITY_REJECTED));
    let decision = output
        .memory_snapshot
        .execution
        .last_decision
        .as_ref()
        .unwrap();
    assert!(!decision.accepted);
    assert_eq!(
        decision.entries[0].status,
        V4ExecutionCapabilityRuntimeStatus::ModeRejected
    );
    assert!(decision.reason.contains("requires runtime_simulated"));
}

#[test]
fn v4_runtime_rejects_missing_execution_capability_policy() {
    let mut runtime = V4PaperSimulatedRuntime::new(sample_compat_graph()).unwrap();

    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    assert_eq!(
        runtime.machine_state_id(V4_COMPAT_EXECUTION_MACHINE_ID),
        Some("idle")
    );
    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_CAPABILITY_REJECTED));
    let decision = output
        .memory_snapshot
        .execution
        .last_decision
        .as_ref()
        .unwrap();
    assert!(!decision.accepted);
    assert_eq!(
        decision.entries[0].status,
        V4ExecutionCapabilityRuntimeStatus::PolicyMissing
    );
}

#[test]
fn v4_runtime_rejects_forged_external_risk_plane_event_for_execution() {
    let mut graph = sample_compat_graph();
    let execution = graph
        .machines
        .iter_mut()
        .find(|machine| machine.machine_id == V4_COMPAT_EXECUTION_MACHINE_ID)
        .unwrap();
    execution.transitions[0].event.source = None;

    let mut runtime = V4PaperSimulatedRuntime::new(graph).unwrap();
    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_RISK_APPROVED_EVENT.to_string(),
            source: V4_COMPAT_DECISION_MACHINE_ID.to_string(),
            payload: json!({
                "execution_id": "exec_1",
                "risk_plane_approved": true
            }),
            ts_ms: 1,
        })
        .unwrap();

    assert_eq!(
        runtime.machine_state_id(V4_COMPAT_EXECUTION_MACHINE_ID),
        Some("idle")
    );
    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == EVENT_RISK_PLANE_REJECTED));
    assert_eq!(output.memory_snapshot.risk_plane.approved_event_count, 0);
    assert_eq!(output.memory_snapshot.risk_plane.rejected_event_count, 1);
    assert!(output
        .memory_snapshot
        .risk_plane
        .last_decision
        .as_ref()
        .unwrap()
        .reason
        .contains("必须由 Risk Plane 机器转换发出"));
}

#[test]
fn v4_runtime_rejects_execution_event_from_non_risk_plane_source() {
    let mut graph = sample_compat_graph();
    let execution = graph
        .machines
        .iter_mut()
        .find(|machine| machine.machine_id == V4_COMPAT_EXECUTION_MACHINE_ID)
        .unwrap();
    execution.transitions[0].event.source = None;

    let mut runtime = V4PaperSimulatedRuntime::new(graph).unwrap();
    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_RISK_APPROVED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({
                "execution_id": "exec_1",
                "risk_plane_approved": true
            }),
            ts_ms: 1,
        })
        .unwrap();

    assert_eq!(
        runtime.machine_state_id(V4_COMPAT_EXECUTION_MACHINE_ID),
        Some("idle")
    );
    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == EVENT_RISK_PLANE_REJECTED));
    assert!(output
        .memory_snapshot
        .risk_plane
        .last_decision
        .as_ref()
        .unwrap()
        .reason
        .contains("不是运行时 Risk Plane 机器"));
}
