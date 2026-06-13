use super::*;
use qrpc_core_ir::v4::{
    bridge_core_ir_to_v4_machine_graph, unsupported_v4_first_wave_matrix, CapabilitySupportSource,
    ExecutionCapabilityKind, MachineActionSpec, MachineEventSelector, MachineMemoryField,
    MachineState, MachineTransition, RuntimeTradingMode, StateGroup, V4MachineContract,
    V4MachineGraphContract, VenueCapabilityMatrix, V4_COMPAT_CORE_IR_LOADED_EVENT,
    V4_COMPAT_DECISION_MACHINE_ID, V4_COMPAT_EXECUTION_MACHINE_ID,
    V4_COMPAT_OBSERVATION_MACHINE_ID, V4_COMPAT_OBSERVATION_READY_EVENT,
    V4_COMPAT_RISK_APPROVED_EVENT,
};
use qrpc_core_ir::{
    moving_average_compare_expr, AgentPolicy, AgentPolicyKind, ComparisonOp, CoreIndicatorKind,
    CoreMetadata, CoreSourceKind, CoreStrategyIr, CoreTimeInForce, DataBinding, DataBindingKind,
    ExecutionRule, ExecutionSizingKind, IndicatorNode, RiskPolicy, SeriesExpr, SignalKind,
    SignalRule,
};

mod fixture_builders;
mod payload_validation_tests;

use fixture_builders::*;

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

#[test]
fn v4_paper_simulated_runtime_runs_compat_bridge_graph_until_execution_ready() {
    let mut runtime = sample_runtime();

    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    assert_eq!(output.runtime_mode, RuntimeTradingMode::PaperSimulated);
    assert!(!output.provider_order_submission_attached);
    assert_eq!(
        runtime.machine_state_id(V4_COMPAT_OBSERVATION_MACHINE_ID),
        Some("ready")
    );
    assert_eq!(
        runtime.machine_state_id(V4_COMPAT_DECISION_MACHINE_ID),
        Some("ready")
    );
    assert_eq!(
        runtime.machine_state_id(V4_COMPAT_EXECUTION_MACHINE_ID),
        Some("ready")
    );
    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == V4_COMPAT_RISK_APPROVED_EVENT));
    assert!(output.events.iter().any(|event| {
        event.event_type == EVENT_TRANSITION_APPLIED
            && event.source == V4_COMPAT_EXECUTION_MACHINE_ID
    }));
    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == EVENT_RISK_PLANE_APPROVED));
    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_CAPABILITY_ACCEPTED));
    assert_eq!(output.memory_snapshot.risk_plane.approved_event_count, 1);
    assert_eq!(output.memory_snapshot.risk_plane.rejected_event_count, 0);
    assert!(output.memory_snapshot.risk_plane.real_order_path_unlocked);
    assert_eq!(output.memory_snapshot.execution.accepted_count, 1);
    assert_eq!(output.memory_snapshot.execution.rejected_count, 0);
    let execution_decision = output
        .memory_snapshot
        .execution
        .last_decision
        .as_ref()
        .unwrap();
    assert!(execution_decision.accepted);
    assert_eq!(
        execution_decision.entries[0].source,
        CapabilitySupportSource::RuntimeSimulated
    );
    assert_eq!(
        execution_decision.entries[0].status,
        V4ExecutionCapabilityRuntimeStatus::Accepted
    );
    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_ORDER_ACKNOWLEDGED));
    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_ORDER_FILLED));
    assert!(output
        .events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_FEE_CHARGED));
    assert_eq!(output.memory_snapshot.simulated_execution.order_count, 1);
    assert_eq!(output.memory_snapshot.simulated_execution.fill_count, 1);
    assert_eq!(
        output
            .memory_snapshot
            .simulated_execution
            .last_order
            .as_ref()
            .unwrap()
            .status,
        V4SimulatedOrderStatus::Filled
    );
    assert_eq!(
        output
            .memory_snapshot
            .simulated_execution
            .positions
            .first()
            .unwrap()
            .net_quantity,
        1.0
    );
    assert!(
        !output
            .memory_snapshot
            .venue_adapter_boundary
            .provider_order_submission_attached
    );
    assert!(
        output
            .memory_snapshot
            .venue_adapter_boundary
            .rejection_before_provider_submit
    );
}

#[test]
fn v4_runtime_simulated_execution_config_controls_fill_fee_and_asset_curve() {
    let mut runtime = V4PaperSimulatedRuntime::new_with_execution_capabilities(
        sample_compat_graph(),
        runtime_simulated_market_matrix(),
        vec![ExecutionCapabilityKind::Market],
    )
    .unwrap()
    .with_simulated_execution_config(V4SimulatedExecutionConfig {
        starting_cash: 1_000.0,
        quote_asset: "USDT".to_string(),
        default_venue_id: "paper-local".to_string(),
        default_symbol: "ETHUSDT".to_string(),
        default_quantity: 2.0,
        default_price: 100.0,
        default_fee_bps: 10.0,
        default_slippage_bps: 50.0,
        allow_partial_fill: true,
        max_fill_quantity: Some(1.0),
    })
    .unwrap();

    let output = runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    let simulated = output.memory_snapshot.simulated_execution;
    assert_eq!(simulated.order_count, 1);
    assert_eq!(simulated.fill_count, 1);
    assert_eq!(
        simulated.last_order.as_ref().unwrap().status,
        V4SimulatedOrderStatus::PartiallyFilled
    );
    assert_eq!(simulated.last_fill.as_ref().unwrap().quantity, 1.0);
    assert!((simulated.last_fill.as_ref().unwrap().price - 100.5).abs() < 1e-9);
    assert!((simulated.last_fill.as_ref().unwrap().fee - 0.1005).abs() < 1e-9);
    assert!((simulated.cash_balance - 899.3995).abs() < 1e-9);
    assert!((simulated.position_market_value - 100.5).abs() < 1e-9);
    assert!((simulated.portfolio_value - 999.8995).abs() < 1e-9);
    assert_eq!(simulated.asset_curve.len(), 1);
}

#[test]
fn v4_runtime_updates_simulated_portfolio_from_market_price() {
    let mut runtime = sample_runtime();
    runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    let events = runtime
        .update_simulated_market_price("paper-local", "BTCUSDT", 120.0, 2)
        .unwrap();
    let snapshot = runtime.simulated_execution_snapshot();

    assert!(events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_PORTFOLIO_CHANGED));
    assert_eq!(snapshot.positions[0].market_price, 120.0);
    assert_eq!(snapshot.position_market_value, 120.0);
    assert_eq!(snapshot.asset_curve.len(), 2);
}

#[test]
fn v4_runtime_triggers_stop_market_order_on_market_price_update() {
    let mut runtime = sample_runtime();
    runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    let request = V4SimulatedOrderRequest {
        order_id: Some("close-long-stop".to_string()),
        client_order_id: None,
        venue_id: "paper-local".to_string(),
        symbol: "BTCUSDT".to_string(),
        action: V4SimulatedPositionAction::CloseLong,
        order_type: V4SimulatedOrderType::StopMarket,
        quantity: 1.0,
        reference_price: 100.0,
        limit_price: None,
        trigger_price: Some(95.0),
        take_profit_price: None,
        stop_loss_price: None,
        trailing_offset_bps: None,
        expire_at_ms: None,
        time_in_force: Some(V4SimulatedTimeInForce::Gtc),
        post_only: false,
        reduce_only: true,
        close_only: true,
        allow_partial_fill: true,
        fee_bps: 10.0,
        slippage_bps: 0.0,
        max_fill_quantity: None,
    };
    let acknowledgement = runtime.simulated_execution.submit_order(request, 1, 2);
    assert!(acknowledgement
        .events
        .iter()
        .any(|(event_type, _)| *event_type == EVENT_EXECUTION_ORDER_ACKNOWLEDGED));
    assert_eq!(runtime.simulated_execution.snapshot().open_order_count, 1);

    let events = runtime
        .update_simulated_market_price("paper-local", "BTCUSDT", 94.0, 3)
        .unwrap();
    let snapshot = runtime.simulated_execution_snapshot();

    assert!(events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_CONDITIONAL_ORDER_TRIGGERED));
    assert!(events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_ORDER_FILLED));
    assert_eq!(snapshot.open_order_count, 0);
    assert_eq!(snapshot.fill_count, 2);
    assert!(snapshot.positions[0].net_quantity.abs() < 1e-9);
}

#[test]
fn v4_runtime_oco_bracket_fills_one_leg_and_cancels_the_other() {
    let mut runtime = sample_runtime();
    runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    let request = V4SimulatedOrderRequest {
        order_id: Some("bracket-1".to_string()),
        client_order_id: None,
        venue_id: "paper-local".to_string(),
        symbol: "BTCUSDT".to_string(),
        action: V4SimulatedPositionAction::CloseLong,
        order_type: V4SimulatedOrderType::OcoBracket,
        quantity: 1.0,
        reference_price: 100.0,
        limit_price: None,
        trigger_price: None,
        take_profit_price: Some(110.0),
        stop_loss_price: Some(95.0),
        trailing_offset_bps: None,
        expire_at_ms: None,
        time_in_force: Some(V4SimulatedTimeInForce::Gtc),
        post_only: false,
        reduce_only: true,
        close_only: true,
        allow_partial_fill: true,
        fee_bps: 10.0,
        slippage_bps: 0.0,
        max_fill_quantity: None,
    };
    runtime.simulated_execution.submit_order(request, 1, 2);

    let events = runtime
        .update_simulated_market_price("paper-local", "BTCUSDT", 111.0, 3)
        .unwrap();
    let snapshot = runtime.simulated_execution_snapshot();

    assert!(events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_ORDER_CANCELED));
    assert_eq!(snapshot.open_order_count, 0);
    assert_eq!(snapshot.fill_count, 2);
    assert!(snapshot.positions[0].net_quantity.abs() < 1e-9);
}

#[test]
fn v4_runtime_trailing_stop_updates_trigger_then_fills() {
    let mut runtime = sample_runtime();
    runtime
        .submit_event(V4RuntimeInputEvent {
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            source: "runtime".to_string(),
            payload: json!({ "strategy_id": "runtime.compat.sample" }),
            ts_ms: 1,
        })
        .unwrap();

    let request = V4SimulatedOrderRequest {
        order_id: Some("trail-1".to_string()),
        client_order_id: None,
        venue_id: "paper-local".to_string(),
        symbol: "BTCUSDT".to_string(),
        action: V4SimulatedPositionAction::CloseLong,
        order_type: V4SimulatedOrderType::TrailingStop,
        quantity: 1.0,
        reference_price: 100.0,
        limit_price: None,
        trigger_price: None,
        take_profit_price: None,
        stop_loss_price: None,
        trailing_offset_bps: Some(100.0),
        expire_at_ms: None,
        time_in_force: Some(V4SimulatedTimeInForce::Gtc),
        post_only: false,
        reduce_only: true,
        close_only: true,
        allow_partial_fill: true,
        fee_bps: 10.0,
        slippage_bps: 0.0,
        max_fill_quantity: None,
    };
    runtime.simulated_execution.submit_order(request, 1, 2);

    let first_update = runtime
        .update_simulated_market_price("paper-local", "BTCUSDT", 110.0, 3)
        .unwrap();
    assert!(first_update
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_ORDER_AMENDED));
    let last_order = runtime.simulated_execution_snapshot().last_order.unwrap();
    assert_eq!(last_order.trigger_price, Some(108.9));

    let fill_events = runtime
        .update_simulated_market_price("paper-local", "BTCUSDT", 108.0, 4)
        .unwrap();
    assert!(fill_events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_ORDER_FILLED));
    assert!(
        runtime.simulated_execution_snapshot().positions[0]
            .net_quantity
            .abs()
            < 1e-9
    );
}

#[test]
fn v4_runtime_gtd_order_expires_on_advance_time() {
    let mut runtime = sample_runtime();
    let request = V4SimulatedOrderRequest {
        order_id: Some("gtd-1".to_string()),
        client_order_id: None,
        venue_id: "paper-local".to_string(),
        symbol: "BTCUSDT".to_string(),
        action: V4SimulatedPositionAction::Buy,
        order_type: V4SimulatedOrderType::Limit,
        quantity: 1.0,
        reference_price: 100.0,
        limit_price: Some(90.0),
        trigger_price: None,
        take_profit_price: None,
        stop_loss_price: None,
        trailing_offset_bps: None,
        expire_at_ms: Some(10),
        time_in_force: Some(V4SimulatedTimeInForce::Gtd),
        post_only: false,
        reduce_only: false,
        close_only: false,
        allow_partial_fill: true,
        fee_bps: 10.0,
        slippage_bps: 0.0,
        max_fill_quantity: None,
    };
    runtime.simulated_execution.submit_order(request, 1, 2);

    let events = runtime.advance_time(10);

    assert!(events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_ORDER_EXPIRED));
    assert_eq!(runtime.simulated_execution_snapshot().open_order_count, 0);
}

#[test]
fn v4_runtime_amends_open_order_without_replacing_order_id() {
    let mut runtime = sample_runtime();
    let request = V4SimulatedOrderRequest {
        order_id: Some("amendable-1".to_string()),
        client_order_id: None,
        venue_id: "paper-local".to_string(),
        symbol: "BTCUSDT".to_string(),
        action: V4SimulatedPositionAction::Buy,
        order_type: V4SimulatedOrderType::Limit,
        quantity: 1.0,
        reference_price: 100.0,
        limit_price: Some(90.0),
        trigger_price: None,
        take_profit_price: None,
        stop_loss_price: None,
        trailing_offset_bps: None,
        expire_at_ms: None,
        time_in_force: Some(V4SimulatedTimeInForce::Gtc),
        post_only: false,
        reduce_only: false,
        close_only: false,
        allow_partial_fill: true,
        fee_bps: 10.0,
        slippage_bps: 0.0,
        max_fill_quantity: None,
    };
    runtime.simulated_execution.submit_order(request, 1, 2);

    let outcome = runtime.simulated_execution.amend_order(
        "amendable-1",
        Some(100.0),
        Some(91.0),
        None,
        Some(0.5),
        3,
    );
    let order = runtime.simulated_execution_snapshot().last_order.unwrap();

    assert!(outcome
        .events
        .iter()
        .any(|(event_type, _)| *event_type == EVENT_EXECUTION_ORDER_AMENDED));
    assert_eq!(order.order_id, "amendable-1");
    assert_eq!(order.limit_price, Some(91.0));
    assert_eq!(order.requested_quantity, 0.5);
    assert_eq!(order.amend_revision, 1);
}

#[test]
fn v4_backtest_tick_replay_is_deterministic_and_reports_micro_metrics() {
    let ticks = vec![
        V4BacktestTickInput {
            venue_id: "paper-local".to_string(),
            symbol: "BTCUSDT".to_string(),
            price: 70_250.0,
            size: 1.0,
            ts_ms: 1_700_000_060_000,
            sequence: 2,
            event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
        },
        V4BacktestTickInput {
            venue_id: "paper-local".to_string(),
            symbol: "BTCUSDT".to_string(),
            price: 70_000.0,
            size: 1.0,
            ts_ms: 1_700_000_000_000,
            sequence: 1,
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
        runtime.run_backtest_ticks(&ticks).unwrap()
    };

    let left = run_once();
    let right = run_once();

    assert_eq!(left.machine_trajectory, right.machine_trajectory);
    assert_eq!(left.replay_mode, "tick_replay");
    assert_eq!(left.input_tick_count, Some(2));
    assert_eq!(left.input_bar_count, 0);
    assert!(left.microstructure_metrics.is_some());
}

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
