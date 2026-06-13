use super::*;

pub(super) fn sample_core_ir_for_v4_runtime() -> CoreStrategyIr {
    let mut core_ir = CoreStrategyIr::new(
        CoreMetadata {
            strategy_id: "runtime.compat.sample".to_string(),
            name: "Runtime Compat Sample".to_string(),
            source_kind: CoreSourceKind::StrategyIr,
        },
        ExecutionRule {
            execution_id: "exec_1".to_string(),
            venue_kind: "paper".to_string(),
            sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
            slippage_bps: 5.0,
            taker_fee_bps: 10.0,
            total_cost_buffer_bps: 20.0,
            time_in_force: CoreTimeInForce::Gtc,
            params: BTreeMap::new(),
        },
    );
    core_ir.data_bindings.push(DataBinding {
        data_id: "btc_1d".to_string(),
        kind: DataBindingKind::KlineSeries,
        source_hints: BTreeMap::new(),
    });
    core_ir.indicators.push(IndicatorNode {
        indicator_id: "ma_cross_1".to_string(),
        kind: CoreIndicatorKind::MaCross,
        inputs: vec![SeriesExpr::DataRef {
            data_id: "btc_1d".to_string(),
        }],
        spread_spec: None,
        custom_expr: None,
        params: BTreeMap::new(),
    });
    core_ir.signal_rules.push(SignalRule {
        signal_id: "signal_1".to_string(),
        indicator_id: "ma_cross_1".to_string(),
        signal_kind: SignalKind::Long,
        condition: moving_average_compare_expr("btc_1d", 20, ComparisonOp::Gt, 100).unwrap(),
    });
    core_ir.agent_policies.push(AgentPolicy {
        agent_id: "agent_1".to_string(),
        name: "Weighted Agent".to_string(),
        kind: AgentPolicyKind::WeightedSignals,
        input_signal_ids: vec!["signal_1".to_string()],
        rebalance_symbols: Vec::new(),
        rebalance_schedule: None,
        rebalance_allocation_kind: None,
        rebalance_rank_method: None,
        rebalance_score_normalize: None,
        rebalance_target_weights: Vec::new(),
        decision_threshold: Some(0.05),
        max_quantity_ratio: 0.2,
        spread_trigger_bps: None,
        enabled: true,
    });
    core_ir.risk_policies.push(RiskPolicy {
        policy_id: "risk_1".to_string(),
        name: "Risk Guard".to_string(),
        observed_agent_ids: vec!["agent_1".to_string()],
        max_position_ratio: 0.3,
        max_single_weight: None,
        max_concentration_ratio: None,
        max_symbol_net_exposure_ratio: None,
        max_portfolio_net_exposure_ratio: None,
        max_turnover: None,
        min_trade_weight: None,
        max_new_positions_per_rebalance: None,
        max_total_leverage: 1.0,
        max_exchange_leverage: 1.0,
        min_action_interval_ms: 1_000,
        enabled: true,
        max_cross_symbol_leverage: None,
    });
    core_ir
}

pub(super) fn sample_compat_graph() -> V4MachineGraphContract {
    let bridge_report = bridge_core_ir_to_v4_machine_graph(&sample_core_ir_for_v4_runtime());
    bridge_report.graph.unwrap()
}

pub(super) fn runtime_simulated_market_matrix() -> VenueCapabilityMatrix {
    let mut matrix = unsupported_v4_first_wave_matrix("paper-local");
    let market = matrix
        .capabilities
        .iter_mut()
        .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
        .unwrap();
    market.source = CapabilitySupportSource::RuntimeSimulated;
    market.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
    let gtc = matrix
        .capabilities
        .iter_mut()
        .find(|entry| entry.capability == ExecutionCapabilityKind::Gtc)
        .unwrap();
    gtc.source = CapabilitySupportSource::RuntimeSimulated;
    gtc.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
    matrix
}

pub(super) fn provider_native_market_matrix_for_paper() -> VenueCapabilityMatrix {
    let mut matrix = unsupported_v4_first_wave_matrix("paper-local");
    let market = matrix
        .capabilities
        .iter_mut()
        .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
        .unwrap();
    market.source = CapabilitySupportSource::ProviderNative;
    market.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
    matrix
}

pub(super) fn provider_native_market_matrix_for_live_actual() -> VenueCapabilityMatrix {
    let mut matrix = unsupported_v4_first_wave_matrix("paper-local");
    let market = matrix
        .capabilities
        .iter_mut()
        .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
        .unwrap();
    market.source = CapabilitySupportSource::ProviderNative;
    market.supported_modes = vec![RuntimeTradingMode::LiveActual];
    matrix
}

pub(super) fn runtime_simulated_market_matrix_for_live_actual() -> VenueCapabilityMatrix {
    let mut matrix = unsupported_v4_first_wave_matrix("paper-local");
    let market = matrix
        .capabilities
        .iter_mut()
        .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
        .unwrap();
    market.source = CapabilitySupportSource::RuntimeSimulated;
    market.supported_modes = vec![RuntimeTradingMode::LiveActual];
    matrix
}

pub(super) fn sample_runtime() -> V4PaperSimulatedRuntime {
    V4PaperSimulatedRuntime::new_with_execution_capabilities(
        sample_compat_graph(),
        runtime_simulated_market_matrix(),
        vec![ExecutionCapabilityKind::Market],
    )
    .unwrap()
}

pub(super) fn nested_observation_graph(parent_matches: bool) -> V4MachineGraphContract {
    let mut graph = sample_compat_graph();
    let observation = graph
        .machines
        .iter_mut()
        .find(|machine| machine.machine_id == V4_COMPAT_OBSERVATION_MACHINE_ID)
        .unwrap();
    observation.transitions[0].to_state = "idle".to_string();
    observation.transitions[0].event.event_type = if parent_matches {
        V4_COMPAT_CORE_IR_LOADED_EVENT.to_string()
    } else {
        V4_COMPAT_OBSERVATION_READY_EVENT.to_string()
    };
    observation.states[0].child_machine = Some(Box::new(V4MachineContract {
        schema_version: qrpc_core_ir::v4::V4_MACHINE_CONTRACT_VERSION.to_string(),
        machine_id: "data.market.child".to_string(),
        template: observation.template.clone(),
        states: vec![
            MachineState {
                state_id: "idle".to_string(),
                group_id: Some("child_flow".to_string()),
                initial: true,
                terminal: false,
                child_machine: None,
            },
            MachineState {
                state_id: "ready".to_string(),
                group_id: Some("child_flow".to_string()),
                initial: false,
                terminal: false,
                child_machine: None,
            },
        ],
        state_groups: vec![StateGroup {
            group_id: "child_flow".to_string(),
            state_ids: vec!["idle".to_string(), "ready".to_string()],
            conflict_policy: qrpc_core_ir::v4::TransitionConflictPolicy::FirstMatch,
            timeout_ms: None,
        }],
        transitions: vec![MachineTransition {
            transition_id: "data.market.child.t0".to_string(),
            from_state: "idle".to_string(),
            to_state: "ready".to_string(),
            event: MachineEventSelector {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: None,
                freshness: None,
            },
            guard: None,
            priority: 0,
            action: Some(MachineActionSpec {
                emits: Vec::new(),
                memory_writes: vec!["price".to_string()],
                diagnostics: Vec::new(),
            }),
        }],
        memory: vec![MachineMemoryField {
            name: "price".to_string(),
            type_name: "price".to_string(),
            type_ref: None,
            default_value: Some(json!(0.0)),
            nullable: false,
        }],
        cache_policy: MachineCachePolicy::ReturnLastThenRecover,
        silence_policy: MachineSilencePolicy::SoftDormantAfter { ttl_ms: 60_000 },
        recovery_policy: MachineRecoveryPolicy::AsyncRecover,
        priority: 0,
        metadata: BTreeMap::new(),
    }));
    let catalog = graph.event_catalog.as_mut().unwrap();
    for event in &mut catalog.events {
        if event.event_type == V4_COMPAT_CORE_IR_LOADED_EVENT {
            event
                .allowed_consumers
                .push("data.market.child".to_string());
            event.payload_fields.push(MachineEventPayloadField {
                name: "price".to_string(),
                type_name: "price".to_string(),
                required: true,
                nullable: false,
            });
        }
        if event.event_type == V4_COMPAT_OBSERVATION_READY_EVENT {
            event
                .allowed_consumers
                .push(V4_COMPAT_OBSERVATION_MACHINE_ID.to_string());
        }
    }
    graph
}
