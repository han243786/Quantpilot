use super::*;

#[test]
fn v4_runtime_returns_last_cache_and_recovers_soft_silent_machine() {
    let mut runtime = sample_runtime();
    runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    let silence_events = runtime.advance_time(60_001);
    assert!(silence_events.iter().any(|event| {
        event.event_type == EVENT_SILENCE_ENTERED
            && event.payload["machine_id"] == V4_COMPAT_OBSERVATION_MACHINE_ID
    }));
    assert_eq!(
        runtime.machine_status(V4_COMPAT_OBSERVATION_MACHINE_ID),
        Some(V4MachineRuntimeStatus::SoftSilent)
    );

    let pull_events = runtime
        .pull_machine(V4_COMPAT_OBSERVATION_MACHINE_ID, 60_010)
        .unwrap();
    assert!(pull_events
        .iter()
        .any(|event| event.event_type == EVENT_CACHE_RETURNED));
    assert!(pull_events
        .iter()
        .any(|event| event.event_type == EVENT_RECOVERY_STARTED));
    assert_eq!(
        runtime.machine_status(V4_COMPAT_OBSERVATION_MACHINE_ID),
        Some(V4MachineRuntimeStatus::Recovering)
    );

    let recovery_events = runtime
        .complete_recovery(V4_COMPAT_OBSERVATION_MACHINE_ID, 60_020)
        .unwrap();
    assert!(recovery_events
        .iter()
        .any(|event| event.event_type == EVENT_RECOVERY_COMPLETED));
    assert_eq!(
        runtime.machine_status(V4_COMPAT_OBSERVATION_MACHINE_ID),
        Some(V4MachineRuntimeStatus::Active)
    );
}

#[test]
fn v4_runtime_memory_snapshot_records_machine_memory_and_cache() {
    let mut runtime = sample_runtime();
    runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    let snapshot = runtime.memory_snapshot(2);
    let observation = snapshot
        .machines
        .iter()
        .find(|machine| machine.machine_id == V4_COMPAT_OBSERVATION_MACHINE_ID)
        .unwrap();

    assert_eq!(snapshot.runtime_mode, RuntimeTradingMode::PaperSimulated);
    assert!(!snapshot.provider_order_submission_attached);
    assert_eq!(observation.memory["data_binding_count"], Value::from(1_u64));
    assert!(observation.cached_output.is_some());
    assert!(snapshot.event_sequence >= 6);
}
