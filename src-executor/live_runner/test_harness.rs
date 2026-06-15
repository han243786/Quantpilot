use super::*;
use crate::executor_state::ActiveStrategy;
use crate::ws_client::WsEvent;
use qrpc_core::{CoreStrategyIr, RuntimeEvent, Symbol};
use qrpc_core_ir::{
    v4::{RuntimeTradingMode, V4MachineGraphContract, V4StaticContractBundle},
    CoreMetadata, CoreSourceKind, CoreTimeInForce, ExecutionRule, ExecutionSizingKind,
};
use qrpc_runtime::{
    EVENT_EXECUTION_FEE_CHARGED, EVENT_EXECUTION_ORDER_ACKNOWLEDGED, EVENT_EXECUTION_ORDER_FILLED,
    V4_DEFAULT_MARKET_DATA_SOURCE,
};

const SAMPLE_REALTIME_V4_QS: &str = r#"
v4_strategy strategy.v4.w0_1 {
  venue paper-simulated
  mode paper_simulated
  require capability market

  machine data.market observation priority 8000 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on market.tick from idle to ready emit bar_closed write last_signal_at
  }

  machine risk.guard decision priority 9500 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on bar_closed from idle to ready emit risk.approved write last_signal_at
  }

  machine execution.router execution priority 4000 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on risk.approved from idle to ready write last_signal_at
  }

  edge data.market -> risk.guard on bar_closed
  edge risk.guard -> execution.router on risk.approved
  risk_plane risk.guard priority 9000
}
"#;

fn sample_realtime_graph() -> V4MachineGraphContract {
    let bundle = V4StaticContractBundle {
        venue_matrices: vec![executor_v4_market_matrix("paper-simulated")],
        ..V4StaticContractBundle::default()
    };
    let report = quantscript::audit_v4_quant_script_static(SAMPLE_REALTIME_V4_QS, &bundle);
    let handoff = quantscript::build_v4_qs_runtime_handoff(&report);
    assert!(
        handoff.accepted_for_runtime_handoff,
        "expected realtime sample graph to pass handoff: {:?}",
        handoff.diagnostics
    );
    report.parsed_graph.expect("sample v4 graph should parse")
}

fn empty_core_ir(strategy_id: &str) -> CoreStrategyIr {
    CoreStrategyIr::new(
        CoreMetadata {
            strategy_id: strategy_id.to_string(),
            name: strategy_id.to_string(),
            source_kind: CoreSourceKind::RuntimeProtocol,
        },
        ExecutionRule {
            execution_id: format!("exec_{strategy_id}"),
            venue_kind: "paper".into(),
            sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
            slippage_bps: 0.0,
            taker_fee_bps: 0.0,
            total_cost_buffer_bps: 0.0,
            time_in_force: CoreTimeInForce::Gtc,
            params: BTreeMap::new(),
        },
    )
}

#[test]
fn detect_trigger_intent_signal() {
    let event = RuntimeEvent {
        event_id: "evt-1".into(),
        event_type: RuntimeEventType::IntentTriggered,
        trace_id: "t1".into(),
        source_id: "ind_1".into(),
        ts_ms: 1000,
        payload: serde_json::json!({"strength": 0.85, "indicator_id": "ma_cross"}),
    };
    let t = LiveRunner::detect_trigger("s1", &event).unwrap();
    assert_eq!(t.strategy_id, "s1");
    assert_eq!(t.trigger_type, "intent_triggered");
    assert_eq!(t.node_id, "ma_cross");
    assert_eq!(t.strength, 0.85);
}

#[test]
fn detect_trigger_agent_decision() {
    let event = RuntimeEvent {
        event_id: "evt-2".into(),
        event_type: RuntimeEventType::AgentDecisionProduced,
        trace_id: "t2".into(),
        source_id: "agent_1".into(),
        ts_ms: 2000,
        payload: serde_json::json!({"net_strength": 0.6, "agent_id": "a1"}),
    };
    let t = LiveRunner::detect_trigger("s1", &event).unwrap();
    assert_eq!(t.trigger_type, "agent_decided");
    assert_eq!(t.node_id, "a1");
}

#[test]
fn detect_trigger_unknown_returns_none() {
    let event = RuntimeEvent {
        event_id: "evt-3".into(),
        event_type: RuntimeEventType::DataUpdated,
        trace_id: "t3".into(),
        source_id: "d1".into(),
        ts_ms: 3000,
        payload: serde_json::json!({}),
    };
    assert!(LiveRunner::detect_trigger("s1", &event).is_none());
}

#[test]
fn v4_runner_realtime_paper_simulated_tick_closes_local_execution_loop() {
    let strategy_id = "w0_1_realtime_paper_simulated";
    let strategy = ActiveStrategy {
        strategy_id: strategy_id.to_string(),
        name: "W0-1 realtime paper simulated".to_string(),
        runtime_kind: RuntimeKind::V4,
        core_ir: empty_core_ir(strategy_id),
        v4_graph: Some(sample_realtime_graph()),
        graph_json: serde_json::Value::Null,
        params: BTreeMap::new(),
        status: crate::executor_state::StrategyStatus::Loaded,
        subscribed_symbols: vec![Symbol::Other("BTCUSDT".to_string())],
        execution_mode: ExecutionMode::PaperSimulated,
        strategy_config_preflight: None,
    };
    let (trigger_broadcast, _) = broadcast::channel(16);
    let mut pool = RunnerPool::new(trigger_broadcast);
    pool.register(&strategy).unwrap();
    match pool.runners.get(strategy_id).unwrap() {
        RunnerInstance::V4(runner) => assert_eq!(runner.venue_id, "paper-simulated"),
        RunnerInstance::V3(_) => panic!("expected v4 runner"),
    }
    let mut evidence_rx = pool.v4_evidence_broadcast.subscribe();

    pool.broadcast_ws_event(WsEvent::Ticker {
        symbol: "BTCUSDT".to_string(),
        price: 70_000.0,
        ts_ms: 123,
    });

    let evidence = evidence_rx
        .try_recv()
        .expect("v4 runner should broadcast evidence after realtime tick");
    assert_eq!(evidence.strategy_id, strategy_id);
    assert_eq!(
        evidence.memory_snapshot.runtime_mode,
        RuntimeTradingMode::PaperSimulated
    );
    assert!(!evidence.memory_snapshot.provider_order_submission_attached);
    assert!(
        !evidence
            .memory_snapshot
            .venue_adapter_boundary
            .provider_order_submission_attached
    );
    assert!(
        evidence
            .memory_snapshot
            .venue_adapter_boundary
            .rejection_before_provider_submit
    );
    assert_eq!(evidence.memory_snapshot.simulated_execution.order_count, 1);
    assert_eq!(evidence.memory_snapshot.simulated_execution.fill_count, 1);
    assert_eq!(
        evidence
            .memory_snapshot
            .simulated_execution
            .last_fill
            .as_ref()
            .map(|fill| fill.venue_id.as_str()),
        Some("paper-simulated")
    );
    assert!(evidence.runtime_events.iter().any(|event| {
        event.event_type == "market.tick" && event.source == V4_DEFAULT_MARKET_DATA_SOURCE
    }));
    assert!(evidence
        .runtime_events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_ORDER_ACKNOWLEDGED));
    assert!(evidence
        .runtime_events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_ORDER_FILLED));
    assert!(evidence
        .runtime_events
        .iter()
        .any(|event| event.event_type == EVENT_EXECUTION_FEE_CHARGED));
}
