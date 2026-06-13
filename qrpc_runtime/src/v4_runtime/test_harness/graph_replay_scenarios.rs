use super::*;

#[test]
fn market_price_tick_helper_records_market_snapshot() {
    let mut runtime = V4PaperSimulatedRuntime::new_with_execution_capabilities(
        sample_compat_graph(),
        runtime_simulated_market_matrix(),
        vec![ExecutionCapabilityKind::Market],
    )
    .unwrap();

    let output = runtime
        .submit_market_price_tick("paper-local", "BTCUSDT", 70_000.0, 123, "price_tick")
        .unwrap();

    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == "price_tick"));
    assert!(output.events.iter().any(|event| {
        event.event_type == "price_tick" && event.source == V4_DEFAULT_MARKET_DATA_SOURCE
    }));
    assert_eq!(
        output.memory_snapshot.simulated_execution.portfolio_value,
        100_000.0
    );
}

#[test]
fn v4_nested_machine_parent_transition_wins_over_child_transition() {
    let mut runtime = V4PaperSimulatedRuntime::new(nested_observation_graph(true)).unwrap();
    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({
                "strategy_id": "runtime.compat.sample",
                "price": 70_000.0
            }),
            ts_ms: 123,
        })
        .unwrap();

    let parent = output
        .memory_snapshot
        .machines
        .iter()
        .find(|machine| machine.machine_id == V4_COMPAT_OBSERVATION_MACHINE_ID)
        .unwrap();
    assert_eq!(parent.state_id, "idle");
    assert_eq!(parent.children[0].machine_id, "data.market.child");
    assert_eq!(parent.children[0].state_id, "idle");
}

#[test]
fn v4_nested_machine_memory_is_isolated_and_visible_in_snapshot() {
    let mut runtime = V4PaperSimulatedRuntime::new(nested_observation_graph(false)).unwrap();
    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({
                "strategy_id": "runtime.compat.sample",
                "price": 70_000.0
            }),
            ts_ms: 123,
        })
        .unwrap();

    let parent = output
        .memory_snapshot
        .machines
        .iter()
        .find(|machine| machine.machine_id == V4_COMPAT_OBSERVATION_MACHINE_ID)
        .unwrap();
    let child = &parent.children[0];
    assert!(!parent.memory.contains_key("price"));
    assert_eq!(child.state_id, "ready", "{:?}", output.events);
    assert_eq!(child.memory.get("price"), Some(&json!(70_000.0)));
}

#[test]
fn v4_backtest_bar_replay_is_deterministic() {
    let bars = vec![
        V4BacktestBarInput {
            venue_id: "paper-local".to_string(),
            symbol: "BTCUSDT".to_string(),
            close: 70_000.0,
            ts_ms: 1_700_000_000_000,
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
        },
        V4BacktestBarInput {
            venue_id: "paper-local".to_string(),
            symbol: "BTCUSDT".to_string(),
            close: 70_250.0,
            ts_ms: 1_700_000_060_000,
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
        },
    ];
    let run_once = || {
        let mut runtime = V4PaperSimulatedRuntime::new_for_backtest(
            sample_compat_graph(),
            runtime_simulated_market_matrix(),
            vec![ExecutionCapabilityKind::Market],
        )
        .unwrap();
        runtime.run_backtest_bars(&bars).unwrap()
    };

    let left = run_once();
    let right = run_once();

    assert_eq!(left.machine_trajectory, right.machine_trajectory);
    assert_eq!(left.final_snapshot, right.final_snapshot);
    assert_eq!(left.input_bar_count, 2);
    assert_eq!(left.symbols, vec!["BTCUSDT".to_string()]);
}

#[test]
fn v4_multi_symbol_expansion_creates_independent_machine_instances() {
    let graph = sample_compat_graph();
    let expanded =
        expand_v4_graph_for_symbols(&graph, &["BTCUSDT".to_string(), "ETHUSDT".to_string()])
            .unwrap();

    assert_eq!(expanded.machines.len(), graph.machines.len() * 2);
    assert!(expanded
        .machines
        .iter()
        .any(|machine| machine.machine_id.starts_with("btcusdt::")));
    assert!(expanded
        .machines
        .iter()
        .any(|machine| machine.machine_id.starts_with("ethusdt::")));
    assert_eq!(
        expanded
            .risk_plane
            .as_ref()
            .map(|plane| plane.machine_ids.len()),
        graph
            .risk_plane
            .as_ref()
            .map(|plane| plane.machine_ids.len() * 2)
    );
    expanded.validate_static_contract().unwrap();
}
