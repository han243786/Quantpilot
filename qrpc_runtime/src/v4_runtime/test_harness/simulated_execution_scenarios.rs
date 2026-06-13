use super::*;

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
