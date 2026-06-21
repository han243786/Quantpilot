use super::*;

#[test]
fn v4_runtime_rejects_event_payload_missing_required_catalog_field() {
    let mut runtime = sample_runtime();

    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({}),
            ts_ms: 1,
        })
        .unwrap();

    assert_eq!(
        runtime.machine_state_id(V4_COMPAT_OBSERVATION_MACHINE_ID),
        Some("idle")
    );
    let rejection = output
        .events
        .iter()
        .find(|event| event.event_type == V4_RUNTIME_EVENT_REJECTED_EVENT)
        .expect("missing v4 runtime rejection event");
    assert!(rejection
        .payload
        .get("reason")
        .and_then(Value::as_str)
        .unwrap()
        .contains("strategy_id"));
}

#[test]
fn v4_runtime_rejects_event_payload_with_wrong_catalog_type() {
    let mut runtime = sample_runtime();

    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": 42 }),
            ts_ms: 1,
        })
        .unwrap();

    assert_eq!(
        runtime.machine_state_id(V4_COMPAT_OBSERVATION_MACHINE_ID),
        Some("idle")
    );
    let rejection = output
        .events
        .iter()
        .find(|event| event.event_type == V4_RUNTIME_EVENT_REJECTED_EVENT)
        .expect("missing v4 runtime rejection event");
    assert!(rejection
        .payload
        .get("reason")
        .and_then(Value::as_str)
        .unwrap()
        .contains("type mismatch"));
}

#[test]
fn v4_runtime_rejects_unknown_external_input_payload_field() {
    let mut runtime = sample_runtime();

    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({
                "strategy_id": "runtime.compat.sample",
                "unexpected": true
            }),
            ts_ms: 1,
        })
        .unwrap();

    assert_eq!(
        runtime.machine_state_id(V4_COMPAT_OBSERVATION_MACHINE_ID),
        Some("idle")
    );
    let rejection = output
        .events
        .iter()
        .find(|event| event.event_type == V4_RUNTIME_EVENT_REJECTED_EVENT)
        .expect("missing v4 runtime rejection event");
    assert!(rejection
        .payload
        .get("reason")
        .and_then(Value::as_str)
        .unwrap()
        .contains("unknown field `unexpected`"));
}

#[test]
fn v4_runtime_rejects_structured_guard_descriptor_without_execution() {
    let mut graph = sample_compat_graph();
    let observation = graph
        .machines
        .iter_mut()
        .find(|machine| machine.machine_id == V4_COMPAT_OBSERVATION_MACHINE_ID)
        .unwrap();
    observation.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
        guard_id: "loaded_event_guard".to_string(),
        reads: vec![
            MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "strategy_id".to_string(),
            },
            MachineGuardReadRef {
                source: MachineGuardReadSource::ReadonlyRuntimeFact,
                path: "clock.tick_ms".to_string(),
            },
        ],
        parameter_paths: vec!["guard.min_signal_age_ms".to_string()],
        explanation: Some("structured descriptor is not executable yet".to_string()),
    });
    let mut runtime = V4PaperSimulatedRuntime::new(graph).unwrap();

    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    assert_eq!(
        runtime.machine_state_id(V4_COMPAT_OBSERVATION_MACHINE_ID),
        Some("idle")
    );
    let rejection = output
        .events
        .iter()
        .find(|event| event.event_type == V4_RUNTIME_EVENT_REJECTED_EVENT)
        .expect("missing v4 runtime rejection event");
    let reason = rejection
        .payload
        .get("reason")
        .and_then(Value::as_str)
        .unwrap();
    assert!(reason.contains("structured guard `loaded_event_guard`"));
    assert!(reason.contains("2 reads"));
    assert!(reason.contains("guard execution is not enabled"));
}

#[test]
fn v4_runtime_input_event_denies_unknown_top_level_fields() {
    let result = serde_json::from_value::<V4RuntimeInputEvent>(json!({
        "event_type": V4_COMPAT_CORE_IR_LOADED_EVENT,
        "source": "runtime",
        "payload": { "strategy_id": "runtime.compat.sample" },
        "ts_ms": 1,
        "unexpected": true
    }));

    assert!(result.is_err());
}

#[test]
fn v4_runtime_rejects_non_finite_default_execution_costs() {
    let mut config = V4SimulatedExecutionConfig::default();
    config.default_fee_bps = f64::INFINITY;
    let error = V4PaperSimulatedRuntime::new(sample_compat_graph())
        .unwrap()
        .with_simulated_execution_config(config)
        .expect_err("non-finite default fee should be rejected");
    assert!(error.to_string().contains("default_fee_bps"));

    let mut config = V4SimulatedExecutionConfig::default();
    config.default_slippage_bps = f64::NAN;
    let error = V4PaperSimulatedRuntime::new(sample_compat_graph())
        .unwrap()
        .with_simulated_execution_config(config)
        .expect_err("non-finite default slippage should be rejected");
    assert!(error.to_string().contains("default_slippage_bps"));
}
