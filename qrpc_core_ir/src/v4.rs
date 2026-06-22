mod backtest_artifact_contract;
mod compile_time_capability_report;
mod complexity_budget_contract;
mod core_ir_compat_bridge;
mod developer_learning_pipeline_contract;
mod machine_contract;
mod machine_graph_contract;
mod plugin_governance_contract;
mod qs_state_machine_profile;
mod qs_type_system_contract;
mod reproducibility_contract;
mod runtime_mode_contract;
mod schema_identity_constants;
mod static_contract_bundle;
mod venue_capability_matrix;
mod version_manifest;

pub use backtest_artifact_contract::*;
pub use compile_time_capability_report::*;
pub use complexity_budget_contract::*;
pub use core_ir_compat_bridge::*;
pub use developer_learning_pipeline_contract::*;
pub use machine_contract::*;
pub use machine_graph_contract::*;
pub use plugin_governance_contract::*;
pub use qs_state_machine_profile::*;
pub use qs_type_system_contract::*;
pub use reproducibility_contract::*;
pub use runtime_mode_contract::*;
pub use schema_identity_constants::*;
pub use static_contract_bundle::*;
pub use venue_capability_matrix::*;
pub use version_manifest::*;

use qs_type_system_contract::default_qs_type_system_version;

fn default_machine_contract_version() -> String {
    V4_MACHINE_CONTRACT_VERSION.to_string()
}

fn default_machine_graph_contract_version() -> String {
    V4_MACHINE_GRAPH_CONTRACT_VERSION.to_string()
}

fn default_machine_event_catalog_version() -> String {
    V4_MACHINE_EVENT_CATALOG_VERSION.to_string()
}

fn default_v4_backtest_artifact_version() -> String {
    V4_BACKTEST_ARTIFACT_VERSION.to_string()
}

fn default_transition_conflict_policy() -> TransitionConflictPolicy {
    TransitionConflictPolicy::Error
}

fn default_machine_graph_edge_activation() -> MachineGraphEdgeActivation {
    MachineGraphEdgeActivation::Always
}

fn default_risk_plane_min_priority() -> i32 {
    V4_RISK_PLANE_MIN_PRIORITY
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        moving_average_compare_expr, AgentPolicy, AgentPolicyKind, ComparisonOp, CoreIREdge,
        CoreIndicatorKind, CoreMetadata, CoreSourceKind, CoreStrategyIr, CoreTimeInForce,
        DataBinding, DataBindingKind, ExecutionRule, ExecutionSizingKind, IndicatorNode,
        RiskPolicy, SeriesExpr, SignalKind, SignalRule,
    };
    use std::collections::{BTreeMap, BTreeSet};

    fn sample_machine() -> V4MachineContract {
        V4MachineContract {
            schema_version: V4_MACHINE_CONTRACT_VERSION.to_string(),
            machine_id: "intent.trend".to_string(),
            template: MachineTemplateKind::Decision,
            states: vec![
                MachineState {
                    state_id: "idle".to_string(),
                    group_id: Some("signal_flow".to_string()),
                    initial: true,
                    terminal: false,
                    child_machine: None,
                },
                MachineState {
                    state_id: "long_bias".to_string(),
                    group_id: Some("signal_flow".to_string()),
                    initial: false,
                    terminal: false,
                    child_machine: None,
                },
            ],
            state_groups: vec![StateGroup {
                group_id: "signal_flow".to_string(),
                state_ids: vec!["idle".to_string(), "long_bias".to_string()],
                conflict_policy: TransitionConflictPolicy::Error,
                timeout_ms: None,
            }],
            transitions: vec![MachineTransition {
                transition_id: "idle_to_long".to_string(),
                from_state: "idle".to_string(),
                to_state: "long_bias".to_string(),
                event: MachineEventSelector {
                    event_type: "bar_closed".to_string(),
                    source: Some("market.btc_1m".to_string()),
                    freshness: Some(EventFreshnessRequirement::FreshOnly),
                },
                guard: Some("ema_fast > ema_slow".to_string()),
                guard_descriptor: None,
                priority: 100,
                action: Some(MachineActionSpec {
                    emits: vec!["intent.long".to_string()],
                    memory_writes: vec!["last_signal_at".to_string()],
                    diagnostics: vec!["trend_score".to_string()],
                }),
            }],
            memory: vec![MachineMemoryField {
                name: "last_signal_at".to_string(),
                type_name: "time?".to_string(),
                type_ref: Some(QsTypeRef::Optional {
                    inner: Box::new(QsTypeRef::Scalar {
                        scalar: QsScalarTypeKind::Time,
                    }),
                }),
                default_value: None,
                nullable: true,
            }],
            cache_policy: MachineCachePolicy::ReturnLastThenRecover,
            silence_policy: MachineSilencePolicy::SoftDormantAfter { ttl_ms: 30_000 },
            recovery_policy: MachineRecoveryPolicy::AsyncRecover,
            priority: 5_200,
            metadata: BTreeMap::new(),
        }
    }

    fn sample_machine_with(
        machine_id: &str,
        template: MachineTemplateKind,
        priority: i32,
    ) -> V4MachineContract {
        let mut machine = sample_machine();
        machine.machine_id = machine_id.to_string();
        machine.template = template;
        machine.priority = priority;
        machine.transitions[0].transition_id = format!("{machine_id}.transition");
        machine
    }

    fn sample_graph_edge(
        source_machine_id: &str,
        target_machine_id: &str,
        event_type: &str,
    ) -> MachineGraphEdge {
        MachineGraphEdge {
            edge_id: format!("{source_machine_id}->{target_machine_id}"),
            source_machine_id: source_machine_id.to_string(),
            target_machine_id: target_machine_id.to_string(),
            event_type: event_type.to_string(),
            activation: MachineGraphEdgeActivation::Always,
            required: true,
            metadata: BTreeMap::new(),
        }
    }

    fn sample_event_spec(
        event_type: &str,
        source_kind: MachineEventSourceKind,
        scope: MachineEventScope,
        allowed_emitters: &[&str],
        allowed_consumers: &[&str],
    ) -> MachineEventTypeSpec {
        MachineEventTypeSpec {
            event_type: event_type.to_string(),
            source_kind,
            scope,
            payload_fields: vec![MachineEventPayloadField {
                name: "symbol".to_string(),
                type_name: "string".to_string(),
                required: true,
                nullable: false,
            }],
            allowed_emitters: allowed_emitters
                .iter()
                .map(|emitter| emitter.to_string())
                .collect(),
            allowed_consumers: allowed_consumers
                .iter()
                .map(|consumer| consumer.to_string())
                .collect(),
            replayable: true,
        }
    }

    fn sample_event_catalog() -> MachineEventCatalog {
        MachineEventCatalog {
            schema_version: V4_MACHINE_EVENT_CATALOG_VERSION.to_string(),
            events: vec![
                sample_event_spec(
                    "market.tick",
                    MachineEventSourceKind::MarketData,
                    MachineEventScope::Runtime,
                    &["market.btc_1m"],
                    &["data.market"],
                ),
                sample_event_spec(
                    "bar_closed",
                    MachineEventSourceKind::Machine,
                    MachineEventScope::Graph,
                    &["data.market"],
                    &["intent.trend"],
                ),
                sample_event_spec(
                    "intent.long",
                    MachineEventSourceKind::Machine,
                    MachineEventScope::Graph,
                    &["intent.trend"],
                    &["risk.guard"],
                ),
                sample_event_spec(
                    "risk.approved",
                    MachineEventSourceKind::RiskPlane,
                    MachineEventScope::Graph,
                    &["risk.guard"],
                    &["execution.router"],
                ),
            ],
            metadata: BTreeMap::new(),
        }
    }

    fn sample_machine_graph() -> V4MachineGraphContract {
        let mut data = sample_machine_with("data.market", MachineTemplateKind::Observation, 8_000);
        data.transitions[0].event.event_type = "market.tick".to_string();
        data.transitions[0].event.source = Some("market.btc_1m".to_string());
        data.transitions[0].action = Some(MachineActionSpec {
            emits: vec!["bar_closed".to_string()],
            memory_writes: vec!["last_signal_at".to_string()],
            diagnostics: vec!["market_bar".to_string()],
        });

        let mut intent = sample_machine_with("intent.trend", MachineTemplateKind::Decision, 5_200);
        intent.transitions[0].event.event_type = "bar_closed".to_string();
        intent.transitions[0].event.source = Some("data.market".to_string());
        intent.transitions[0].action = Some(MachineActionSpec {
            emits: vec!["intent.long".to_string()],
            memory_writes: vec!["last_signal_at".to_string()],
            diagnostics: vec!["trend_score".to_string()],
        });

        let mut risk = sample_machine_with("risk.guard", MachineTemplateKind::Decision, 9_500);
        risk.transitions[0].event.event_type = "intent.long".to_string();
        risk.transitions[0].event.source = Some("intent.trend".to_string());
        risk.transitions[0].action = Some(MachineActionSpec {
            emits: vec!["risk.approved".to_string()],
            memory_writes: vec!["last_signal_at".to_string()],
            diagnostics: vec!["risk_decision".to_string()],
        });

        let mut execution =
            sample_machine_with("execution.router", MachineTemplateKind::Execution, 4_000);
        execution.transitions[0].event.event_type = "risk.approved".to_string();
        execution.transitions[0].event.source = Some("risk.guard".to_string());
        execution.transitions[0].action = Some(MachineActionSpec {
            emits: Vec::new(),
            memory_writes: vec!["last_signal_at".to_string()],
            diagnostics: vec!["route_order".to_string()],
        });

        V4MachineGraphContract {
            schema_version: V4_MACHINE_GRAPH_CONTRACT_VERSION.to_string(),
            graph_id: "strategy.v4.sample".to_string(),
            machines: vec![data, intent, risk, execution],
            edges: vec![
                sample_graph_edge("data.market", "intent.trend", "bar_closed"),
                sample_graph_edge("intent.trend", "risk.guard", "intent.long"),
                sample_graph_edge("risk.guard", "execution.router", "risk.approved"),
            ],
            event_catalog: Some(sample_event_catalog()),
            risk_plane: Some(MachineGraphRiskPlane {
                required: true,
                machine_ids: vec!["risk.guard".to_string()],
                min_priority: V4_RISK_PLANE_MIN_PRIORITY,
            }),
            metadata: BTreeMap::new(),
        }
    }

    fn sample_static_contract_bundle() -> V4StaticContractBundle {
        V4StaticContractBundle {
            machine_graphs: vec![sample_machine_graph()],
            venue_matrices: vec![unsupported_v4_first_wave_matrix("paper-local")],
            ..V4StaticContractBundle::default()
        }
    }

    fn sample_paper_simulated_market_matrix() -> VenueCapabilityMatrix {
        let mut matrix = unsupported_v4_first_wave_matrix("paper-local");
        let market = matrix
            .capabilities
            .iter_mut()
            .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
            .unwrap();
        market.source = CapabilitySupportSource::RuntimeSimulated;
        market.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
        matrix
    }

    fn sample_compile_time_capability_request() -> V4CompileTimeCapabilityRequest {
        V4CompileTimeCapabilityRequest {
            schema_version: V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION.to_string(),
            graph_id: "strategy.v4.sample".to_string(),
            venue_id: "paper-local".to_string(),
            runtime_mode: RuntimeTradingMode::PaperSimulated,
            required_execution_capabilities: vec![ExecutionCapabilityKind::Market],
            required_type_refs: vec![QsTypeRef::Scalar {
                scalar: QsScalarTypeKind::Price,
            }],
            required_plugin_ids: vec!["pure.indicator.zscore".to_string()],
        }
    }

    fn sample_pure_plugin_manifest() -> PluginManifestSpec {
        PluginManifestSpec {
            plugin_id: "pure.indicator.zscore".to_string(),
            name: "ZScore".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::Pure,
            input_schema: Some(QsTypeRef::List {
                item: Box::new(QsTypeRef::Scalar {
                    scalar: QsScalarTypeKind::Price,
                }),
                max_items: 256,
            }),
            output_schema: Some(QsTypeRef::Scalar {
                scalar: QsScalarTypeKind::Decimal,
            }),
            deterministic: true,
            side_effect: PluginSideEffect::None,
            runtime_permission: PluginRuntimePermission::None,
            network_permission: PluginNetworkPermission::None,
            capability_matrix: None,
            test_fixture_id: "fixture.zscore.basic".to_string(),
        }
    }

    fn sample_core_ir_for_v4_bridge() -> CoreStrategyIr {
        let mut core_ir = CoreStrategyIr::new(
            CoreMetadata {
                strategy_id: "legacy.sample".to_string(),
                name: "Legacy Sample".to_string(),
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

    #[test]
    fn core_ir_v4_bridge_maps_legacy_core_ir_to_default_machines() {
        let report = bridge_core_ir_to_v4_machine_graph(&sample_core_ir_for_v4_bridge());

        assert_eq!(report.verdict, CoreIrV4BridgeVerdict::Accepted);
        assert_eq!(report.validate_for_phase4(), Ok(()));
        assert!(!report.lowering_attached);
        assert!(!report.runtime_attached);

        let graph = report.graph.as_ref().unwrap();
        assert_eq!(graph.machines.len(), 3);
        assert!(graph.machines.iter().any(|machine| {
            machine.machine_id == V4_COMPAT_OBSERVATION_MACHINE_ID
                && machine.template == MachineTemplateKind::Observation
        }));
        assert!(graph.machines.iter().any(|machine| {
            machine.machine_id == V4_COMPAT_DECISION_MACHINE_ID
                && machine.template == MachineTemplateKind::Decision
                && machine.priority >= V4_RISK_PLANE_MIN_PRIORITY
        }));
        assert!(graph.machines.iter().any(|machine| {
            machine.machine_id == V4_COMPAT_EXECUTION_MACHINE_ID
                && machine.template == MachineTemplateKind::Execution
        }));
        assert_eq!(
            graph.risk_plane.as_ref().unwrap().machine_ids,
            vec![V4_COMPAT_DECISION_MACHINE_ID.to_string()]
        );
        assert!(graph.edges.iter().any(|edge| {
            edge.source_machine_id == V4_COMPAT_DECISION_MACHINE_ID
                && edge.target_machine_id == V4_COMPAT_EXECUTION_MACHINE_ID
                && edge.event_type == V4_COMPAT_RISK_APPROVED_EVENT
        }));
    }

    #[test]
    fn core_ir_v4_bridge_rejects_missing_data_bindings() {
        let mut core_ir = sample_core_ir_for_v4_bridge();
        core_ir.data_bindings.clear();

        let report = bridge_core_ir_to_v4_machine_graph(&core_ir);

        assert_eq!(report.verdict, CoreIrV4BridgeVerdict::Rejected);
        assert!(report.graph.is_none());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "V4BRIDGE002"));
    }

    #[test]
    fn core_ir_v4_bridge_rejects_missing_risk_policies() {
        let mut core_ir = sample_core_ir_for_v4_bridge();
        core_ir.risk_policies.clear();

        let report = bridge_core_ir_to_v4_machine_graph(&core_ir);

        assert_eq!(report.verdict, CoreIrV4BridgeVerdict::Rejected);
        assert!(report.graph.is_none());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "V4BRIDGE003"));
    }

    #[test]
    fn core_ir_v4_bridge_rejects_unknown_core_ir_edge_endpoint() {
        let mut core_ir = sample_core_ir_for_v4_bridge();
        core_ir.edges.push(CoreIREdge {
            source: "missing_node".to_string(),
            target: "exec_1".to_string(),
            port: None,
        });

        let report = bridge_core_ir_to_v4_machine_graph(&core_ir);

        assert_eq!(report.verdict, CoreIrV4BridgeVerdict::Rejected);
        assert!(report.graph.is_none());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "V4BRIDGE031"));
    }

    #[test]
    fn core_ir_v4_bridge_rejects_core_ir_cycle() {
        let mut core_ir = sample_core_ir_for_v4_bridge();
        core_ir.edges = vec![
            CoreIREdge {
                source: "btc_1d".to_string(),
                target: "ma_cross_1".to_string(),
                port: None,
            },
            CoreIREdge {
                source: "ma_cross_1".to_string(),
                target: "btc_1d".to_string(),
                port: None,
            },
        ];

        let report = bridge_core_ir_to_v4_machine_graph(&core_ir);

        assert_eq!(report.verdict, CoreIrV4BridgeVerdict::Rejected);
        assert!(report.graph.is_none());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "V4BRIDGE020"));
    }

    #[test]
    fn machine_contract_accepts_flat_state_group() {
        let machine = sample_machine();
        assert_eq!(machine.validate_static_contract(), Ok(()));
    }

    #[test]
    fn machine_contract_accepts_depth_two_child_machine() {
        let mut parent = sample_machine();
        let mut child = sample_machine();
        child.machine_id = "intent.trend.child".to_string();
        parent.states[0].child_machine = Some(Box::new(child));

        assert_eq!(parent.validate_static_contract(), Ok(()));
    }

    #[test]
    fn machine_contract_rejects_depth_three_child_machine() {
        let mut parent = sample_machine();
        let mut child = sample_machine();
        child.machine_id = "intent.trend.child".to_string();
        let mut grandchild = sample_machine();
        grandchild.machine_id = "intent.trend.grandchild".to_string();
        child.states[0].child_machine = Some(Box::new(grandchild));
        parent.states[0].child_machine = Some(Box::new(child));

        let errors = parent.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("max nested machine depth 2")));
    }

    #[test]
    fn machine_contract_rejects_transition_without_event() {
        let mut machine = sample_machine();
        machine.transitions[0].event.event_type.clear();

        let errors = machine.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("must declare an event_type")));
    }

    #[test]
    fn machine_contract_accepts_structured_guard_descriptor_readiness() {
        let mut machine = sample_machine();
        machine.transitions[0].guard = None;
        machine.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "trend_guard".to_string(),
            reads: vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "ema_fast".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::ReadonlyRuntimeFact,
                    path: "clock.tick_ms".to_string(),
                },
            ],
            parameter_paths: vec!["guard.threshold".to_string(), "timeout.ms".to_string()],
            conditions: vec![MachineGuardConditionSpec {
                condition_id: "ema_above_threshold".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "ema_fast".to_string(),
                },
                comparator: MachineGuardConditionComparator::GreaterThan,
                right_parameter_path: "guard.threshold".to_string(),
            }],
            policy: Some(MachineGuardPolicySpec {
                timeout_ms: Some(500),
                cooldown_ms: Some(1_000),
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            }),
            explanation: Some("first structured guard descriptor surface".to_string()),
        });

        assert_eq!(machine.validate_static_contract(), Ok(()));
        let readiness = machine.transitions[0]
            .guard_descriptor
            .as_ref()
            .unwrap()
            .readiness();
        assert_eq!(readiness.guard_id, "trend_guard");
        assert_eq!(readiness.read_count, 3);
        assert_eq!(readiness.event_payload_read_count, 1);
        assert_eq!(readiness.machine_memory_read_count, 1);
        assert_eq!(readiness.readonly_runtime_fact_read_count, 1);
        assert_eq!(readiness.parameter_path_count, 2);
        assert_eq!(readiness.guard_parameter_path_count, 0);
        assert_eq!(readiness.timeout_parameter_path_count, 1);
        assert_eq!(readiness.cooldown_parameter_path_count, 0);
        assert_eq!(readiness.threshold_parameter_path_count, 1);
        assert_eq!(readiness.risk_limit_parameter_path_count, 0);
        assert_eq!(readiness.condition_count, 1);
        assert_eq!(readiness.greater_than_condition_count, 1);
        assert_eq!(readiness.equal_condition_count, 0);
        assert_eq!(readiness.less_than_condition_count, 0);
        assert_eq!(readiness.condition_event_payload_read_count, 1);
        assert_eq!(readiness.condition_machine_memory_read_count, 0);
        assert_eq!(readiness.condition_threshold_parameter_path_count, 1);
        assert_eq!(readiness.condition_timeout_parameter_path_count, 0);
        assert!(readiness.policy_declared);
        assert!(readiness.timing_policy_declared);
        assert!(readiness.timeout_declared);
        assert!(readiness.cooldown_declared);
        assert!(readiness.fallback_declared);
        assert!(readiness.fallback_fail_closed_declared);
        assert!(!readiness.execution_enabled);
        assert_eq!(
            readiness.execution_state,
            MachineGuardExecutionReadinessState::DisabledFailClosed
        );
        assert_eq!(
            readiness.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            readiness.execution_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );
    }

    #[test]
    fn machine_contract_projects_guard_descriptors_for_workspace() {
        let mut machine = sample_machine();
        machine.transitions[0].guard = None;
        machine.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "trend_guard".to_string(),
            reads: vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "ema_fast".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
            ],
            parameter_paths: vec![
                "guard.threshold".to_string(),
                "risk.max_notional".to_string(),
            ],
            conditions: vec![MachineGuardConditionSpec {
                condition_id: "ema_threshold_check".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "ema_fast".to_string(),
                },
                comparator: MachineGuardConditionComparator::GreaterThanOrEqual,
                right_parameter_path: "guard.threshold".to_string(),
            }],
            policy: Some(MachineGuardPolicySpec {
                timeout_ms: Some(250),
                cooldown_ms: Some(2_000),
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            }),
            explanation: Some("workspace projection surface".to_string()),
        });

        let projections = machine.guard_descriptor_projections();

        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.transition_id, "idle_to_long");
        assert_eq!(projection.from_state, "idle");
        assert_eq!(projection.to_state, "long_bias");
        assert_eq!(projection.event_type, "bar_closed");
        assert_eq!(projection.event_source.as_deref(), Some("market.btc_1m"));
        assert_eq!(projection.readiness.guard_id, "trend_guard");
        assert_eq!(projection.readiness.read_count, 2);
        assert_eq!(projection.readiness.parameter_path_count, 2);
        assert_eq!(projection.readiness.threshold_parameter_path_count, 1);
        assert_eq!(projection.readiness.risk_limit_parameter_path_count, 1);
        assert_eq!(projection.readiness.condition_count, 1);
        assert_eq!(
            projection.readiness.greater_than_or_equal_condition_count,
            1
        );
        assert_eq!(projection.readiness.greater_than_condition_count, 0);
        assert_eq!(projection.readiness.condition_event_payload_read_count, 1);
        assert_eq!(
            projection
                .readiness
                .condition_threshold_parameter_path_count,
            1
        );
        assert_eq!(
            projection
                .readiness
                .condition_risk_limit_parameter_path_count,
            0
        );
        assert!(projection.readiness.policy_declared);
        assert!(projection.readiness.timing_policy_declared);
        assert!(!projection.readiness.execution_enabled);
        assert_eq!(projection.reads.len(), 2);
        assert_eq!(projection.read_projections.len(), 2);
        assert_eq!(projection.read_projections[0].source_label, "event_payload");
        assert_eq!(
            projection.read_projections[0].binding_scope,
            MachineGuardReadBindingScope::EventPayloadField
        );
        assert_eq!(projection.read_projections[0].path, "ema_fast");
        assert_eq!(
            projection.read_projections[1].source_label,
            "machine_memory"
        );
        assert_eq!(
            projection.read_projections[1].binding_scope,
            MachineGuardReadBindingScope::MachineMemoryField
        );
        assert_eq!(projection.read_projections[1].path, "last_signal_at");
        assert_eq!(
            projection.parameter_paths,
            vec![
                "guard.threshold".to_string(),
                "risk.max_notional".to_string()
            ]
        );
        assert_eq!(
            projection.parameter_path_kinds,
            vec![
                MachineGuardParameterPathKind::Threshold,
                MachineGuardParameterPathKind::RiskLimit,
            ]
        );
        assert_eq!(projection.parameter_path_projections.len(), 2);
        assert_eq!(
            projection.parameter_path_projections[0].kind,
            Some(MachineGuardParameterPathKind::Threshold)
        );
        assert!(projection.parameter_path_projections[0].proposal_only);
        assert!(!projection.parameter_path_projections[0].active_strategy_write_enabled);
        assert_eq!(
            projection.parameter_path_projections[1].kind,
            Some(MachineGuardParameterPathKind::RiskLimit)
        );
        assert!(projection.parameter_path_projections[1].proposal_only);
        assert!(!projection.parameter_path_projections[1].active_strategy_write_enabled);
        assert_eq!(projection.conditions.len(), 1);
        assert_eq!(projection.conditions[0].condition_id, "ema_threshold_check");
        assert_eq!(projection.condition_projections.len(), 1);
        assert_eq!(
            projection.condition_projections[0].right_parameter_path_kind,
            Some(MachineGuardParameterPathKind::Threshold)
        );
        assert!(!projection.condition_projections[0].evaluation_enabled);
        assert_eq!(
            projection.condition_projections[0].evaluation_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            projection.condition_projections[0].evaluation_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );
        assert_eq!(
            projection.condition_projections[0]
                .left_read_projection
                .as_ref()
                .unwrap()
                .binding_scope,
            MachineGuardReadBindingScope::EventPayloadField
        );
        assert_eq!(
            projection.condition_projections[0]
                .left_read_projection
                .as_ref()
                .unwrap()
                .path,
            "ema_fast"
        );
        assert_eq!(
            projection.condition_projections[0].right_parameter_path,
            "guard.threshold"
        );
        let right_parameter_projection = projection.condition_projections[0]
            .right_parameter_path_projection
            .as_ref()
            .unwrap();
        assert_eq!(
            right_parameter_projection.kind,
            Some(MachineGuardParameterPathKind::Threshold)
        );
        assert!(right_parameter_projection.proposal_only);
        assert!(!right_parameter_projection.active_strategy_write_enabled);
        let policy = projection.policy.as_ref().unwrap();
        assert_eq!(policy.timeout_ms, Some(250));
        assert_eq!(policy.cooldown_ms, Some(2_000));
        assert_eq!(
            policy.fallback,
            Some(MachineGuardFallbackPolicy::FailClosed)
        );
        let policy_projection = projection.policy_projection.as_ref().unwrap();
        assert!(policy_projection.timing_policy_declared);
        assert!(policy_projection.timeout_declared);
        assert!(policy_projection.cooldown_declared);
        assert!(policy_projection.fallback_declared);
        assert!(policy_projection.fallback_fail_closed_declared);
        assert_eq!(policy_projection.timeout_ms, Some(250));
        assert_eq!(policy_projection.cooldown_ms, Some(2_000));
        assert_eq!(
            policy_projection.fallback,
            Some(MachineGuardFallbackPolicy::FailClosed)
        );
        assert!(!policy_projection.timing_execution_enabled);
        assert!(!policy_projection.fallback_execution_enabled);
        assert!(!policy_projection.active_strategy_write_enabled);
        assert_eq!(
            policy_projection.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            policy_projection.execution_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );
    }

    #[test]
    fn machine_graph_projects_guard_descriptors_with_machine_context() {
        let mut graph = sample_machine_graph();
        let intent = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "intent.trend")
            .unwrap();
        intent.transitions[0].guard = None;
        intent.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "intent_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            parameter_paths: vec!["guard.threshold".to_string()],
            conditions: vec![MachineGuardConditionSpec {
                condition_id: "intent_symbol_threshold_check".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "symbol".to_string(),
                },
                comparator: MachineGuardConditionComparator::NotEqual,
                right_parameter_path: "guard.threshold".to_string(),
            }],
            policy: Some(MachineGuardPolicySpec {
                timeout_ms: Some(150),
                cooldown_ms: None,
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            }),
            explanation: Some("graph projection surface".to_string()),
        });

        let projections = graph.guard_descriptor_projections();

        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.machine_id, "intent.trend");
        assert_eq!(projection.machine_template, MachineTemplateKind::Decision);
        assert_eq!(projection.guard.transition_id, "intent.trend.transition");
        assert_eq!(projection.guard.event_type, "bar_closed");
        assert_eq!(
            projection.guard.event_source.as_deref(),
            Some("data.market")
        );
        assert_eq!(projection.guard.readiness.guard_id, "intent_guard");
        assert!(!projection.guard.readiness.execution_enabled);
        let summary = graph.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guard_id_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.guarded_transition_count, 1);
        assert_eq!(summary.guarded_event_type_count, 1);
        assert_eq!(summary.guarded_event_source_count, 1);
        assert_eq!(summary.observation_guard_descriptor_count, 0);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.execution_guard_descriptor_count, 0);
        assert_eq!(summary.event_source_declared_count, 1);
        assert_eq!(summary.event_source_missing_count, 0);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.read_count, 1);
        assert_eq!(summary.event_payload_read_count, 1);
        assert_eq!(summary.parameterized_guard_descriptor_count, 1);
        assert_eq!(summary.parameter_path_count, 1);
        assert_eq!(summary.guard_parameter_path_count, 0);
        assert_eq!(summary.timeout_parameter_path_count, 0);
        assert_eq!(summary.cooldown_parameter_path_count, 0);
        assert_eq!(summary.threshold_parameter_path_count, 1);
        assert_eq!(summary.risk_limit_parameter_path_count, 0);
        assert_eq!(summary.parameter_path_proposal_only_count, 1);
        assert_eq!(summary.proposal_only_guard_descriptor_count, 1);
        assert_eq!(
            summary.parameter_path_active_strategy_write_enabled_count,
            0
        );
        assert_eq!(
            summary.parameter_path_active_strategy_write_disabled_count,
            1
        );
        assert_eq!(summary.conditional_guard_descriptor_count, 1);
        assert_eq!(summary.condition_count, 1);
        assert_eq!(summary.equal_condition_count, 0);
        assert_eq!(summary.not_equal_condition_count, 1);
        assert_eq!(summary.greater_than_condition_count, 0);
        assert_eq!(summary.greater_than_or_equal_condition_count, 0);
        assert_eq!(summary.less_than_condition_count, 0);
        assert_eq!(summary.less_than_or_equal_condition_count, 0);
        assert_eq!(summary.condition_event_payload_read_count, 1);
        assert_eq!(summary.condition_machine_memory_read_count, 0);
        assert_eq!(summary.condition_readonly_runtime_fact_read_count, 0);
        assert_eq!(summary.condition_guard_parameter_path_count, 0);
        assert_eq!(summary.condition_timeout_parameter_path_count, 0);
        assert_eq!(summary.condition_cooldown_parameter_path_count, 0);
        assert_eq!(summary.condition_threshold_parameter_path_count, 1);
        assert_eq!(summary.condition_risk_limit_parameter_path_count, 0);
        assert_eq!(summary.condition_evaluation_enabled_count, 0);
        assert_eq!(
            summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(summary.condition_evaluation_disabled_fail_closed_count, 1);
        assert_eq!(summary.policy_declared_count, 1);
        assert_eq!(summary.timing_policy_declared_count, 1);
        assert_eq!(summary.timeout_declared_count, 1);
        assert_eq!(summary.cooldown_declared_count, 0);
        assert_eq!(summary.fallback_declared_count, 1);
        assert_eq!(summary.fallback_fail_closed_declared_count, 1);
        assert_eq!(summary.policy_timing_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(
            summary.policy_timing_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(summary.policy_fallback_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_fallback_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(summary.policy_active_strategy_write_enabled_count, 0);
        assert_eq!(summary.policy_active_strategy_write_disabled_count, 1);
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            1
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn static_contract_bundle_projects_guard_descriptors_with_graph_context() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        risk.transitions[0].guard = None;
        risk.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "risk_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            parameter_paths: vec!["risk.max_notional".to_string(), "cooldown.ms".to_string()],
            conditions: vec![MachineGuardConditionSpec {
                condition_id: "symbol_risk_limit_check".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "symbol".to_string(),
                },
                comparator: MachineGuardConditionComparator::NotEqual,
                right_parameter_path: "risk.max_notional".to_string(),
            }],
            policy: Some(MachineGuardPolicySpec {
                timeout_ms: None,
                cooldown_ms: Some(3_000),
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            }),
            explanation: Some("bundle projection surface".to_string()),
        });

        let projections = bundle.guard_descriptor_projections();

        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.graph_id, "strategy.v4.sample");
        assert_eq!(projection.guard.machine_id, "risk.guard");
        assert_eq!(
            projection.guard.machine_template,
            MachineTemplateKind::Decision
        );
        assert_eq!(
            projection.guard.guard.transition_id,
            "risk.guard.transition"
        );
        assert_eq!(projection.guard.guard.event_type, "intent.long");
        assert_eq!(
            projection.guard.guard.event_source.as_deref(),
            Some("intent.trend")
        );
        assert_eq!(projection.guard.guard.readiness.guard_id, "risk_guard");
        assert_eq!(projection.guard.guard.read_projections.len(), 1);
        assert_eq!(
            projection.guard.guard.read_projections[0].binding_scope,
            MachineGuardReadBindingScope::EventPayloadField
        );
        assert_eq!(
            projection.guard.guard.read_projections[0].source_label,
            "event_payload"
        );
        assert_eq!(
            projection.guard.guard.parameter_paths,
            vec!["risk.max_notional".to_string(), "cooldown.ms".to_string()]
        );
        assert_eq!(
            projection.guard.guard.parameter_path_kinds,
            vec![
                MachineGuardParameterPathKind::RiskLimit,
                MachineGuardParameterPathKind::Cooldown,
            ]
        );
        assert_eq!(projection.guard.guard.parameter_path_projections.len(), 2);
        assert_eq!(
            projection.guard.guard.parameter_path_projections[0].kind,
            Some(MachineGuardParameterPathKind::RiskLimit)
        );
        assert!(projection.guard.guard.parameter_path_projections[0].proposal_only);
        assert!(
            !projection.guard.guard.parameter_path_projections[0].active_strategy_write_enabled
        );
        assert_eq!(
            projection.guard.guard.parameter_path_projections[1].kind,
            Some(MachineGuardParameterPathKind::Cooldown)
        );
        assert!(projection.guard.guard.parameter_path_projections[1].proposal_only);
        assert!(
            !projection.guard.guard.parameter_path_projections[1].active_strategy_write_enabled
        );
        let summary = bundle.guard_descriptor_summary();
        let graph_summary = bundle
            .machine_graphs
            .first()
            .expect("bundle should contain one test graph")
            .guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guard_id_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.guarded_transition_count, 1);
        assert_eq!(summary.guarded_event_type_count, 1);
        assert_eq!(summary.guarded_event_source_count, 1);
        assert_eq!(summary.observation_guard_descriptor_count, 0);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.execution_guard_descriptor_count, 0);
        assert_eq!(summary.event_source_declared_count, 1);
        assert_eq!(summary.event_source_missing_count, 0);
        let summary_parity_fields = [
            (
                "guard_descriptor_count",
                summary.guard_descriptor_count,
                graph_summary.guard_descriptor_count,
            ),
            (
                "guard_id_count",
                summary.guard_id_count,
                graph_summary.guard_id_count,
            ),
            (
                "guarded_machine_count",
                summary.guarded_machine_count,
                graph_summary.guarded_machine_count,
            ),
            (
                "guarded_transition_count",
                summary.guarded_transition_count,
                graph_summary.guarded_transition_count,
            ),
            (
                "guarded_event_type_count",
                summary.guarded_event_type_count,
                graph_summary.guarded_event_type_count,
            ),
            (
                "guarded_event_source_count",
                summary.guarded_event_source_count,
                graph_summary.guarded_event_source_count,
            ),
            (
                "observation_guard_descriptor_count",
                summary.observation_guard_descriptor_count,
                graph_summary.observation_guard_descriptor_count,
            ),
            (
                "decision_guard_descriptor_count",
                summary.decision_guard_descriptor_count,
                graph_summary.decision_guard_descriptor_count,
            ),
            (
                "execution_guard_descriptor_count",
                summary.execution_guard_descriptor_count,
                graph_summary.execution_guard_descriptor_count,
            ),
            (
                "event_source_declared_count",
                summary.event_source_declared_count,
                graph_summary.event_source_declared_count,
            ),
            (
                "event_source_missing_count",
                summary.event_source_missing_count,
                graph_summary.event_source_missing_count,
            ),
            (
                "read_guard_descriptor_count",
                summary.read_guard_descriptor_count,
                graph_summary.read_guard_descriptor_count,
            ),
            ("read_count", summary.read_count, graph_summary.read_count),
            (
                "event_payload_read_count",
                summary.event_payload_read_count,
                graph_summary.event_payload_read_count,
            ),
            (
                "machine_memory_read_count",
                summary.machine_memory_read_count,
                graph_summary.machine_memory_read_count,
            ),
            (
                "readonly_runtime_fact_read_count",
                summary.readonly_runtime_fact_read_count,
                graph_summary.readonly_runtime_fact_read_count,
            ),
            (
                "parameterized_guard_descriptor_count",
                summary.parameterized_guard_descriptor_count,
                graph_summary.parameterized_guard_descriptor_count,
            ),
            (
                "parameter_path_count",
                summary.parameter_path_count,
                graph_summary.parameter_path_count,
            ),
            (
                "guard_parameter_path_count",
                summary.guard_parameter_path_count,
                graph_summary.guard_parameter_path_count,
            ),
            (
                "timeout_parameter_path_count",
                summary.timeout_parameter_path_count,
                graph_summary.timeout_parameter_path_count,
            ),
            (
                "cooldown_parameter_path_count",
                summary.cooldown_parameter_path_count,
                graph_summary.cooldown_parameter_path_count,
            ),
            (
                "threshold_parameter_path_count",
                summary.threshold_parameter_path_count,
                graph_summary.threshold_parameter_path_count,
            ),
            (
                "risk_limit_parameter_path_count",
                summary.risk_limit_parameter_path_count,
                graph_summary.risk_limit_parameter_path_count,
            ),
            (
                "parameter_path_proposal_only_count",
                summary.parameter_path_proposal_only_count,
                graph_summary.parameter_path_proposal_only_count,
            ),
            (
                "proposal_only_guard_descriptor_count",
                summary.proposal_only_guard_descriptor_count,
                graph_summary.proposal_only_guard_descriptor_count,
            ),
            (
                "parameter_path_active_strategy_write_enabled_count",
                summary.parameter_path_active_strategy_write_enabled_count,
                graph_summary.parameter_path_active_strategy_write_enabled_count,
            ),
            (
                "parameter_path_active_strategy_write_disabled_count",
                summary.parameter_path_active_strategy_write_disabled_count,
                graph_summary.parameter_path_active_strategy_write_disabled_count,
            ),
            (
                "conditional_guard_descriptor_count",
                summary.conditional_guard_descriptor_count,
                graph_summary.conditional_guard_descriptor_count,
            ),
            (
                "condition_count",
                summary.condition_count,
                graph_summary.condition_count,
            ),
            (
                "equal_condition_count",
                summary.equal_condition_count,
                graph_summary.equal_condition_count,
            ),
            (
                "not_equal_condition_count",
                summary.not_equal_condition_count,
                graph_summary.not_equal_condition_count,
            ),
            (
                "greater_than_condition_count",
                summary.greater_than_condition_count,
                graph_summary.greater_than_condition_count,
            ),
            (
                "greater_than_or_equal_condition_count",
                summary.greater_than_or_equal_condition_count,
                graph_summary.greater_than_or_equal_condition_count,
            ),
            (
                "less_than_condition_count",
                summary.less_than_condition_count,
                graph_summary.less_than_condition_count,
            ),
            (
                "less_than_or_equal_condition_count",
                summary.less_than_or_equal_condition_count,
                graph_summary.less_than_or_equal_condition_count,
            ),
            (
                "condition_event_payload_read_count",
                summary.condition_event_payload_read_count,
                graph_summary.condition_event_payload_read_count,
            ),
            (
                "condition_machine_memory_read_count",
                summary.condition_machine_memory_read_count,
                graph_summary.condition_machine_memory_read_count,
            ),
            (
                "condition_readonly_runtime_fact_read_count",
                summary.condition_readonly_runtime_fact_read_count,
                graph_summary.condition_readonly_runtime_fact_read_count,
            ),
            (
                "condition_guard_parameter_path_count",
                summary.condition_guard_parameter_path_count,
                graph_summary.condition_guard_parameter_path_count,
            ),
            (
                "condition_timeout_parameter_path_count",
                summary.condition_timeout_parameter_path_count,
                graph_summary.condition_timeout_parameter_path_count,
            ),
            (
                "condition_cooldown_parameter_path_count",
                summary.condition_cooldown_parameter_path_count,
                graph_summary.condition_cooldown_parameter_path_count,
            ),
            (
                "condition_threshold_parameter_path_count",
                summary.condition_threshold_parameter_path_count,
                graph_summary.condition_threshold_parameter_path_count,
            ),
            (
                "condition_risk_limit_parameter_path_count",
                summary.condition_risk_limit_parameter_path_count,
                graph_summary.condition_risk_limit_parameter_path_count,
            ),
            (
                "condition_evaluation_enabled_count",
                summary.condition_evaluation_enabled_count,
                graph_summary.condition_evaluation_enabled_count,
            ),
            (
                "condition_evaluation_disabled_fail_closed_guard_descriptor_count",
                summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
                graph_summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
            ),
            (
                "condition_evaluation_disabled_fail_closed_count",
                summary.condition_evaluation_disabled_fail_closed_count,
                graph_summary.condition_evaluation_disabled_fail_closed_count,
            ),
            (
                "policy_declared_count",
                summary.policy_declared_count,
                graph_summary.policy_declared_count,
            ),
            (
                "timing_policy_declared_count",
                summary.timing_policy_declared_count,
                graph_summary.timing_policy_declared_count,
            ),
            (
                "timeout_declared_count",
                summary.timeout_declared_count,
                graph_summary.timeout_declared_count,
            ),
            (
                "cooldown_declared_count",
                summary.cooldown_declared_count,
                graph_summary.cooldown_declared_count,
            ),
            (
                "fallback_declared_count",
                summary.fallback_declared_count,
                graph_summary.fallback_declared_count,
            ),
            (
                "fallback_fail_closed_declared_count",
                summary.fallback_fail_closed_declared_count,
                graph_summary.fallback_fail_closed_declared_count,
            ),
            (
                "policy_timing_execution_enabled_count",
                summary.policy_timing_execution_enabled_count,
                graph_summary.policy_timing_execution_enabled_count,
            ),
            (
                "policy_execution_disabled_fail_closed_guard_descriptor_count",
                summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
                graph_summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
            ),
            (
                "policy_timing_execution_disabled_fail_closed_count",
                summary.policy_timing_execution_disabled_fail_closed_count,
                graph_summary.policy_timing_execution_disabled_fail_closed_count,
            ),
            (
                "policy_fallback_execution_enabled_count",
                summary.policy_fallback_execution_enabled_count,
                graph_summary.policy_fallback_execution_enabled_count,
            ),
            (
                "policy_fallback_execution_disabled_fail_closed_count",
                summary.policy_fallback_execution_disabled_fail_closed_count,
                graph_summary.policy_fallback_execution_disabled_fail_closed_count,
            ),
            (
                "policy_active_strategy_write_enabled_count",
                summary.policy_active_strategy_write_enabled_count,
                graph_summary.policy_active_strategy_write_enabled_count,
            ),
            (
                "policy_active_strategy_write_disabled_count",
                summary.policy_active_strategy_write_disabled_count,
                graph_summary.policy_active_strategy_write_disabled_count,
            ),
            (
                "active_strategy_write_disabled_guard_descriptor_count",
                summary.active_strategy_write_disabled_guard_descriptor_count,
                graph_summary.active_strategy_write_disabled_guard_descriptor_count,
            ),
            (
                "execution_enabled_count",
                summary.execution_enabled_count,
                graph_summary.execution_enabled_count,
            ),
            (
                "execution_disabled_fail_closed_count",
                summary.execution_disabled_fail_closed_count,
                graph_summary.execution_disabled_fail_closed_count,
            ),
        ];
        for (field, bundle_count, graph_count) in summary_parity_fields {
            assert_eq!(
                bundle_count, graph_count,
                "bundle/graph summary mismatch: {field}"
            );
        }
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.read_count, 1);
        assert_eq!(summary.event_payload_read_count, 1);
        assert_eq!(summary.parameterized_guard_descriptor_count, 1);
        assert_eq!(summary.parameter_path_count, 2);
        assert_eq!(summary.cooldown_parameter_path_count, 1);
        assert_eq!(summary.risk_limit_parameter_path_count, 1);
        assert_eq!(summary.parameter_path_proposal_only_count, 2);
        assert_eq!(summary.proposal_only_guard_descriptor_count, 1);
        assert_eq!(
            summary.parameter_path_active_strategy_write_enabled_count,
            0
        );
        assert_eq!(
            summary.parameter_path_active_strategy_write_disabled_count,
            2
        );
        assert_eq!(summary.conditional_guard_descriptor_count, 1);
        assert_eq!(summary.condition_count, 1);
        assert_eq!(summary.not_equal_condition_count, 1);
        assert_eq!(summary.equal_condition_count, 0);
        assert_eq!(summary.condition_event_payload_read_count, 1);
        assert_eq!(summary.condition_risk_limit_parameter_path_count, 1);
        assert_eq!(summary.condition_cooldown_parameter_path_count, 0);
        assert_eq!(summary.condition_evaluation_enabled_count, 0);
        assert_eq!(
            summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(summary.condition_evaluation_disabled_fail_closed_count, 1);
        let condition_projection = &projection.guard.guard.condition_projections[0];
        assert!(!condition_projection.evaluation_enabled);
        assert_eq!(
            condition_projection.evaluation_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            condition_projection.evaluation_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );
        assert_eq!(
            condition_projection
                .left_read_projection
                .as_ref()
                .unwrap()
                .binding_scope,
            MachineGuardReadBindingScope::EventPayloadField
        );
        assert_eq!(
            condition_projection
                .right_parameter_path_projection
                .as_ref()
                .unwrap()
                .kind,
            Some(MachineGuardParameterPathKind::RiskLimit)
        );
        assert!(
            condition_projection
                .right_parameter_path_projection
                .as_ref()
                .unwrap()
                .proposal_only
        );
        assert!(
            !condition_projection
                .right_parameter_path_projection
                .as_ref()
                .unwrap()
                .active_strategy_write_enabled
        );
        assert_eq!(summary.policy_declared_count, 1);
        assert_eq!(summary.timing_policy_declared_count, 1);
        assert_eq!(summary.cooldown_declared_count, 1);
        assert_eq!(summary.fallback_declared_count, 1);
        assert_eq!(summary.fallback_fail_closed_declared_count, 1);
        assert_eq!(summary.policy_timing_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(
            summary.policy_timing_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(summary.policy_fallback_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_fallback_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(summary.policy_active_strategy_write_enabled_count, 0);
        assert_eq!(summary.policy_active_strategy_write_disabled_count, 1);
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            1
        );
        let policy_projection = projection.guard.guard.policy_projection.as_ref().unwrap();
        assert!(policy_projection.timing_policy_declared);
        assert!(!policy_projection.timeout_declared);
        assert!(policy_projection.cooldown_declared);
        assert!(policy_projection.fallback_fail_closed_declared);
        assert!(!policy_projection.timing_execution_enabled);
        assert!(!policy_projection.fallback_execution_enabled);
        assert!(!policy_projection.active_strategy_write_enabled);
        assert_eq!(
            policy_projection.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
        assert!(!projection.guard.guard.readiness.execution_enabled);
        assert_eq!(
            projection.guard.guard.readiness.execution_state,
            MachineGuardExecutionReadinessState::DisabledFailClosed
        );
        assert_eq!(
            projection.guard.guard.readiness.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            projection.guard.guard.readiness.execution_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );
    }

    #[test]
    fn static_contract_bundle_summarizes_guard_descriptors_across_graphs() {
        fn attach_risk_guard_descriptor(graph: &mut V4MachineGraphContract, guard_id: &str) {
            let risk = graph
                .machines
                .iter_mut()
                .find(|machine| machine.machine_id == "risk.guard")
                .unwrap();
            risk.transitions[0].guard = None;
            risk.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
                guard_id: guard_id.to_string(),
                reads: vec![MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "symbol".to_string(),
                }],
                parameter_paths: vec!["risk.max_notional".to_string()],
                conditions: Vec::new(),
                policy: None,
                explanation: Some("bundle multi-graph summary surface".to_string()),
            });
        }

        let mut bundle = sample_static_contract_bundle();
        bundle.machine_graphs[0].graph_id = "strategy.v4.alpha".to_string();
        let mut second_graph = sample_machine_graph();
        second_graph.graph_id = "strategy.v4.beta".to_string();
        bundle.machine_graphs.push(second_graph);
        attach_risk_guard_descriptor(&mut bundle.machine_graphs[0], "risk_guard_alpha");
        attach_risk_guard_descriptor(&mut bundle.machine_graphs[1], "risk_guard_beta");

        let projections = bundle.guard_descriptor_projections();
        let graph_ids = projections
            .iter()
            .map(|projection| projection.graph_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(projections.len(), 2);
        assert_eq!(graph_ids.len(), 2);
        assert!(graph_ids.contains("strategy.v4.alpha"));
        assert!(graph_ids.contains("strategy.v4.beta"));

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 2);
        assert_eq!(summary.guard_id_count, 2);
        assert_eq!(summary.guarded_machine_count, 2);
        assert_eq!(summary.guarded_transition_count, 2);
        assert_eq!(summary.guarded_event_type_count, 2);
        assert_eq!(summary.guarded_event_source_count, 2);
        assert_eq!(summary.event_source_declared_count, 2);
        assert_eq!(summary.event_source_missing_count, 0);
        assert_eq!(summary.decision_guard_descriptor_count, 2);
        assert_eq!(summary.read_guard_descriptor_count, 2);
        assert_eq!(summary.read_count, 2);
        assert_eq!(summary.event_payload_read_count, 2);
        assert_eq!(summary.parameterized_guard_descriptor_count, 2);
        assert_eq!(summary.parameter_path_count, 2);
        assert_eq!(summary.risk_limit_parameter_path_count, 2);
        assert_eq!(summary.parameter_path_proposal_only_count, 2);
        assert_eq!(summary.proposal_only_guard_descriptor_count, 2);
        assert_eq!(
            summary.parameter_path_active_strategy_write_disabled_count,
            2
        );
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            2
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 2);
    }

    #[test]
    fn static_contract_bundle_summarizes_guard_descriptor_mixed_event_sources() {
        fn attach_risk_guard_descriptor(graph: &mut V4MachineGraphContract, guard_id: &str) {
            let risk = graph
                .machines
                .iter_mut()
                .find(|machine| machine.machine_id == "risk.guard")
                .unwrap();
            risk.transitions[0].guard = None;
            risk.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
                guard_id: guard_id.to_string(),
                reads: Vec::new(),
                parameter_paths: Vec::new(),
                conditions: Vec::new(),
                policy: None,
                explanation: Some("bundle mixed event source summary surface".to_string()),
            });
        }

        let mut bundle = sample_static_contract_bundle();
        bundle.machine_graphs[0].graph_id = "strategy.v4.alpha".to_string();
        let mut second_graph = sample_machine_graph();
        second_graph.graph_id = "strategy.v4.beta".to_string();
        let beta_risk = second_graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        beta_risk.transitions[0].event.source = None;
        bundle.machine_graphs.push(second_graph);
        attach_risk_guard_descriptor(&mut bundle.machine_graphs[0], "risk_guard_alpha");
        attach_risk_guard_descriptor(&mut bundle.machine_graphs[1], "risk_guard_beta");

        let projections = bundle.guard_descriptor_projections();
        let event_sources = projections
            .iter()
            .filter_map(|projection| projection.guard.guard.event_source.as_deref())
            .collect::<BTreeSet<_>>();
        assert_eq!(projections.len(), 2);
        assert_eq!(event_sources.len(), 1);
        assert!(event_sources.contains("intent.trend"));

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 2);
        assert_eq!(summary.guard_id_count, 2);
        assert_eq!(summary.guarded_machine_count, 2);
        assert_eq!(summary.guarded_transition_count, 2);
        assert_eq!(summary.guarded_event_type_count, 2);
        assert_eq!(summary.guarded_event_source_count, 1);
        assert_eq!(summary.event_source_declared_count, 1);
        assert_eq!(summary.event_source_missing_count, 1);
        assert_eq!(summary.decision_guard_descriptor_count, 2);
        assert_eq!(summary.read_guard_descriptor_count, 0);
        assert_eq!(summary.parameterized_guard_descriptor_count, 0);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 2);
    }

    #[test]
    fn static_contract_bundle_summarizes_guard_descriptor_duplicate_ids_across_graphs() {
        fn attach_risk_guard_descriptor(graph: &mut V4MachineGraphContract, guard_id: &str) {
            let risk = graph
                .machines
                .iter_mut()
                .find(|machine| machine.machine_id == "risk.guard")
                .unwrap();
            risk.transitions[0].guard = None;
            risk.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
                guard_id: guard_id.to_string(),
                reads: Vec::new(),
                parameter_paths: Vec::new(),
                conditions: Vec::new(),
                policy: None,
                explanation: Some("bundle duplicate guard id summary surface".to_string()),
            });
        }

        let mut bundle = sample_static_contract_bundle();
        bundle.machine_graphs[0].graph_id = "strategy.v4.alpha".to_string();
        let mut second_graph = sample_machine_graph();
        second_graph.graph_id = "strategy.v4.beta".to_string();
        bundle.machine_graphs.push(second_graph);
        attach_risk_guard_descriptor(&mut bundle.machine_graphs[0], "risk_guard_shared");
        attach_risk_guard_descriptor(&mut bundle.machine_graphs[1], "risk_guard_shared");

        let projections = bundle.guard_descriptor_projections();
        let guard_ids = projections
            .iter()
            .map(|projection| projection.guard.guard.readiness.guard_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(projections.len(), 2);
        assert_eq!(guard_ids.len(), 1);
        assert!(guard_ids.contains("risk_guard_shared"));

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 2);
        assert_eq!(summary.guard_id_count, 1);
        assert_eq!(summary.guarded_machine_count, 2);
        assert_eq!(summary.guarded_transition_count, 2);
        assert_eq!(summary.guarded_event_type_count, 2);
        assert_eq!(summary.guarded_event_source_count, 2);
        assert_eq!(summary.event_source_declared_count, 2);
        assert_eq!(summary.event_source_missing_count, 0);
        assert_eq!(summary.decision_guard_descriptor_count, 2);
        assert_eq!(summary.read_guard_descriptor_count, 0);
        assert_eq!(summary.parameterized_guard_descriptor_count, 0);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 2);
    }

    #[test]
    fn static_contract_bundle_summarizes_guard_descriptor_templates_across_graphs() {
        fn attach_guard_descriptor(
            graph: &mut V4MachineGraphContract,
            machine_id: &str,
            guard_id: &str,
        ) {
            let machine = graph
                .machines
                .iter_mut()
                .find(|machine| machine.machine_id == machine_id)
                .unwrap();
            machine.transitions[0].guard = None;
            machine.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
                guard_id: guard_id.to_string(),
                reads: Vec::new(),
                parameter_paths: Vec::new(),
                conditions: Vec::new(),
                policy: None,
                explanation: Some("bundle template summary surface".to_string()),
            });
        }

        let mut bundle = sample_static_contract_bundle();
        bundle.machine_graphs[0].graph_id = "strategy.v4.alpha".to_string();
        let mut second_graph = sample_machine_graph();
        second_graph.graph_id = "strategy.v4.beta".to_string();
        bundle.machine_graphs.push(second_graph);
        attach_guard_descriptor(
            &mut bundle.machine_graphs[0],
            "data.market",
            "observation_guard",
        );
        attach_guard_descriptor(
            &mut bundle.machine_graphs[0],
            "execution.router",
            "execution_guard",
        );
        attach_guard_descriptor(
            &mut bundle.machine_graphs[1],
            "risk.guard",
            "decision_guard",
        );

        let projections = bundle.guard_descriptor_projections();
        let graph_ids = projections
            .iter()
            .map(|projection| projection.graph_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(projections.len(), 3);
        assert_eq!(graph_ids.len(), 2);

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 3);
        assert_eq!(summary.guard_id_count, 3);
        assert_eq!(summary.guarded_machine_count, 3);
        assert_eq!(summary.guarded_transition_count, 3);
        assert_eq!(summary.guarded_event_type_count, 3);
        assert_eq!(summary.guarded_event_source_count, 3);
        assert_eq!(summary.event_source_declared_count, 3);
        assert_eq!(summary.event_source_missing_count, 0);
        assert_eq!(summary.observation_guard_descriptor_count, 1);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.execution_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 0);
        assert_eq!(summary.parameterized_guard_descriptor_count, 0);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 3);
    }

    #[test]
    fn static_contract_bundle_summarizes_guard_descriptor_read_sources_across_graphs() {
        fn attach_risk_guard_descriptor(
            graph: &mut V4MachineGraphContract,
            guard_id: &str,
            reads: Vec<MachineGuardReadRef>,
        ) {
            let risk = graph
                .machines
                .iter_mut()
                .find(|machine| machine.machine_id == "risk.guard")
                .unwrap();
            risk.transitions[0].guard = None;
            risk.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
                guard_id: guard_id.to_string(),
                reads,
                parameter_paths: Vec::new(),
                conditions: Vec::new(),
                policy: None,
                explanation: Some("bundle read source summary surface".to_string()),
            });
        }

        let mut bundle = sample_static_contract_bundle();
        bundle.machine_graphs[0].graph_id = "strategy.v4.alpha".to_string();
        let mut second_graph = sample_machine_graph();
        second_graph.graph_id = "strategy.v4.beta".to_string();
        bundle.machine_graphs.push(second_graph);
        attach_risk_guard_descriptor(
            &mut bundle.machine_graphs[0],
            "risk_guard_alpha",
            vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::ReadonlyRuntimeFact,
                    path: "runtime.mode".to_string(),
                },
            ],
        );
        attach_risk_guard_descriptor(
            &mut bundle.machine_graphs[1],
            "risk_guard_beta",
            vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
        );

        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 2);
        assert_eq!(projections[0].guard.guard.read_projections.len(), 2);
        assert_eq!(projections[1].guard.guard.read_projections.len(), 1);

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 2);
        assert_eq!(summary.guard_id_count, 2);
        assert_eq!(summary.guarded_machine_count, 2);
        assert_eq!(summary.guarded_transition_count, 2);
        assert_eq!(summary.guarded_event_type_count, 2);
        assert_eq!(summary.guarded_event_source_count, 2);
        assert_eq!(summary.decision_guard_descriptor_count, 2);
        assert_eq!(summary.read_guard_descriptor_count, 2);
        assert_eq!(summary.read_count, 3);
        assert_eq!(summary.event_payload_read_count, 1);
        assert_eq!(summary.machine_memory_read_count, 1);
        assert_eq!(summary.readonly_runtime_fact_read_count, 1);
        assert_eq!(summary.parameterized_guard_descriptor_count, 0);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 2);
    }

    #[test]
    fn static_contract_bundle_summarizes_guard_descriptor_condition_operands_across_graphs() {
        fn attach_risk_guard_descriptor(
            graph: &mut V4MachineGraphContract,
            guard_id: &str,
            reads: Vec<MachineGuardReadRef>,
            parameter_paths: Vec<String>,
            conditions: Vec<MachineGuardConditionSpec>,
        ) {
            let risk = graph
                .machines
                .iter_mut()
                .find(|machine| machine.machine_id == "risk.guard")
                .unwrap();
            risk.transitions[0].guard = None;
            risk.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
                guard_id: guard_id.to_string(),
                reads,
                parameter_paths,
                conditions,
                policy: None,
                explanation: Some("bundle condition operand summary surface".to_string()),
            });
        }

        let mut bundle = sample_static_contract_bundle();
        bundle.machine_graphs[0].graph_id = "strategy.v4.alpha".to_string();
        let mut second_graph = sample_machine_graph();
        second_graph.graph_id = "strategy.v4.beta".to_string();
        bundle.machine_graphs.push(second_graph);
        attach_risk_guard_descriptor(
            &mut bundle.machine_graphs[0],
            "risk_guard_alpha",
            vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::ReadonlyRuntimeFact,
                    path: "runtime.mode".to_string(),
                },
            ],
            vec!["guard.enabled".to_string(), "timeout.ms".to_string()],
            vec![
                MachineGuardConditionSpec {
                    condition_id: "memory_guard_check".to_string(),
                    left_read: MachineGuardReadRef {
                        source: MachineGuardReadSource::MachineMemory,
                        path: "last_signal_at".to_string(),
                    },
                    comparator: MachineGuardConditionComparator::GreaterThan,
                    right_parameter_path: "guard.enabled".to_string(),
                },
                MachineGuardConditionSpec {
                    condition_id: "runtime_timeout_check".to_string(),
                    left_read: MachineGuardReadRef {
                        source: MachineGuardReadSource::ReadonlyRuntimeFact,
                        path: "runtime.mode".to_string(),
                    },
                    comparator: MachineGuardConditionComparator::Equal,
                    right_parameter_path: "timeout.ms".to_string(),
                },
            ],
        );
        attach_risk_guard_descriptor(
            &mut bundle.machine_graphs[1],
            "risk_guard_beta",
            vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            vec!["risk.max_notional".to_string()],
            vec![MachineGuardConditionSpec {
                condition_id: "event_risk_limit_check".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "symbol".to_string(),
                },
                comparator: MachineGuardConditionComparator::NotEqual,
                right_parameter_path: "risk.max_notional".to_string(),
            }],
        );

        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 2);
        assert_eq!(projections[0].guard.guard.condition_projections.len(), 2);
        assert_eq!(projections[1].guard.guard.condition_projections.len(), 1);

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 2);
        assert_eq!(summary.conditional_guard_descriptor_count, 2);
        assert_eq!(summary.condition_count, 3);
        assert_eq!(summary.equal_condition_count, 1);
        assert_eq!(summary.not_equal_condition_count, 1);
        assert_eq!(summary.greater_than_condition_count, 1);
        assert_eq!(summary.condition_event_payload_read_count, 1);
        assert_eq!(summary.condition_machine_memory_read_count, 1);
        assert_eq!(summary.condition_readonly_runtime_fact_read_count, 1);
        assert_eq!(summary.condition_guard_parameter_path_count, 1);
        assert_eq!(summary.condition_timeout_parameter_path_count, 1);
        assert_eq!(summary.condition_risk_limit_parameter_path_count, 1);
        assert_eq!(summary.condition_evaluation_enabled_count, 0);
        assert_eq!(
            summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
            2
        );
        assert_eq!(summary.condition_evaluation_disabled_fail_closed_count, 3);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 2);
    }

    #[test]
    fn static_contract_bundle_summarizes_guard_descriptor_policy_mix_across_graphs() {
        fn attach_risk_guard_descriptor(
            graph: &mut V4MachineGraphContract,
            guard_id: &str,
            policy: MachineGuardPolicySpec,
        ) {
            let risk = graph
                .machines
                .iter_mut()
                .find(|machine| machine.machine_id == "risk.guard")
                .unwrap();
            risk.transitions[0].guard = None;
            risk.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
                guard_id: guard_id.to_string(),
                reads: Vec::new(),
                parameter_paths: Vec::new(),
                conditions: Vec::new(),
                policy: Some(policy),
                explanation: Some("bundle policy summary surface".to_string()),
            });
        }

        let mut bundle = sample_static_contract_bundle();
        bundle.machine_graphs[0].graph_id = "strategy.v4.alpha".to_string();
        let mut second_graph = sample_machine_graph();
        second_graph.graph_id = "strategy.v4.beta".to_string();
        bundle.machine_graphs.push(second_graph);
        attach_risk_guard_descriptor(
            &mut bundle.machine_graphs[0],
            "risk_guard_alpha",
            MachineGuardPolicySpec {
                timeout_ms: Some(500),
                cooldown_ms: None,
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            },
        );
        attach_risk_guard_descriptor(
            &mut bundle.machine_graphs[1],
            "risk_guard_beta",
            MachineGuardPolicySpec {
                timeout_ms: None,
                cooldown_ms: Some(1_000),
                fallback: None,
            },
        );

        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 2);
        assert!(projections[0].guard.guard.policy_projection.is_some());
        assert!(projections[1].guard.guard.policy_projection.is_some());

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 2);
        assert_eq!(summary.policy_declared_count, 2);
        assert_eq!(summary.timing_policy_declared_count, 2);
        assert_eq!(summary.timeout_declared_count, 1);
        assert_eq!(summary.cooldown_declared_count, 1);
        assert_eq!(summary.fallback_declared_count, 1);
        assert_eq!(summary.fallback_fail_closed_declared_count, 1);
        assert_eq!(summary.policy_timing_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
            2
        );
        assert_eq!(
            summary.policy_timing_execution_disabled_fail_closed_count,
            2
        );
        assert_eq!(summary.policy_fallback_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_fallback_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(summary.policy_active_strategy_write_enabled_count, 0);
        assert_eq!(summary.policy_active_strategy_write_disabled_count, 2);
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            2
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 2);
    }

    #[test]
    fn static_contract_bundle_summarizes_guard_descriptor_parameter_paths_across_graphs() {
        fn attach_risk_guard_descriptor(
            graph: &mut V4MachineGraphContract,
            guard_id: &str,
            parameter_paths: Vec<&str>,
        ) {
            let risk = graph
                .machines
                .iter_mut()
                .find(|machine| machine.machine_id == "risk.guard")
                .unwrap();
            risk.transitions[0].guard = None;
            risk.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
                guard_id: guard_id.to_string(),
                reads: Vec::new(),
                parameter_paths: parameter_paths
                    .into_iter()
                    .map(str::to_string)
                    .collect::<Vec<_>>(),
                conditions: Vec::new(),
                policy: None,
                explanation: Some("bundle parameter path summary surface".to_string()),
            });
        }

        let mut bundle = sample_static_contract_bundle();
        bundle.machine_graphs[0].graph_id = "strategy.v4.alpha".to_string();
        let mut second_graph = sample_machine_graph();
        second_graph.graph_id = "strategy.v4.beta".to_string();
        bundle.machine_graphs.push(second_graph);
        attach_risk_guard_descriptor(
            &mut bundle.machine_graphs[0],
            "risk_guard_alpha",
            vec!["guard.enabled", "timeout.ms", "cooldown.ms"],
        );
        attach_risk_guard_descriptor(
            &mut bundle.machine_graphs[1],
            "risk_guard_beta",
            vec!["guard.threshold", "risk.max_notional"],
        );

        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 2);
        assert!(projections.iter().all(|projection| projection
            .guard
            .guard
            .parameter_path_projections
            .iter()
            .all(|path| path.proposal_only && !path.active_strategy_write_enabled)));
        let path_kinds = projections
            .iter()
            .flat_map(|projection| projection.guard.guard.parameter_path_projections.iter())
            .map(|path| (path.path.as_str(), path.kind.unwrap()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(path_kinds.len(), 5);
        assert_eq!(
            path_kinds.get("guard.enabled"),
            Some(&MachineGuardParameterPathKind::Guard)
        );
        assert_eq!(
            path_kinds.get("timeout.ms"),
            Some(&MachineGuardParameterPathKind::Timeout)
        );
        assert_eq!(
            path_kinds.get("cooldown.ms"),
            Some(&MachineGuardParameterPathKind::Cooldown)
        );
        assert_eq!(
            path_kinds.get("guard.threshold"),
            Some(&MachineGuardParameterPathKind::Threshold)
        );
        assert_eq!(
            path_kinds.get("risk.max_notional"),
            Some(&MachineGuardParameterPathKind::RiskLimit)
        );

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 2);
        assert_eq!(summary.parameterized_guard_descriptor_count, 2);
        assert_eq!(summary.parameter_path_count, 5);
        assert_eq!(summary.guard_parameter_path_count, 1);
        assert_eq!(summary.timeout_parameter_path_count, 1);
        assert_eq!(summary.cooldown_parameter_path_count, 1);
        assert_eq!(summary.threshold_parameter_path_count, 1);
        assert_eq!(summary.risk_limit_parameter_path_count, 1);
        assert_eq!(summary.parameter_path_proposal_only_count, 5);
        assert_eq!(summary.proposal_only_guard_descriptor_count, 2);
        assert_eq!(
            summary.parameter_path_active_strategy_write_enabled_count,
            0
        );
        assert_eq!(
            summary.parameter_path_active_strategy_write_disabled_count,
            5
        );
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            2
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 2);
    }

    #[test]
    fn static_contract_bundle_summarizes_guard_descriptor_combined_surfaces_across_graphs() {
        fn attach_risk_guard_descriptor(
            graph: &mut V4MachineGraphContract,
            guard_id: &str,
            reads: Vec<MachineGuardReadRef>,
            parameter_paths: Vec<String>,
            conditions: Vec<MachineGuardConditionSpec>,
            policy: Option<MachineGuardPolicySpec>,
        ) {
            let risk = graph
                .machines
                .iter_mut()
                .find(|machine| machine.machine_id == "risk.guard")
                .unwrap();
            risk.transitions[0].guard = None;
            risk.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
                guard_id: guard_id.to_string(),
                reads,
                parameter_paths,
                conditions,
                policy,
                explanation: Some("bundle combined summary surface".to_string()),
            });
        }

        let mut bundle = sample_static_contract_bundle();
        bundle.machine_graphs[0].graph_id = "strategy.v4.alpha".to_string();
        let mut second_graph = sample_machine_graph();
        second_graph.graph_id = "strategy.v4.beta".to_string();
        bundle.machine_graphs.push(second_graph);
        attach_risk_guard_descriptor(
            &mut bundle.machine_graphs[0],
            "risk_guard_combined",
            vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "symbol".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
            ],
            vec!["guard.threshold".to_string(), "timeout.ms".to_string()],
            vec![MachineGuardConditionSpec {
                condition_id: "event_threshold_check".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "symbol".to_string(),
                },
                comparator: MachineGuardConditionComparator::GreaterThanOrEqual,
                right_parameter_path: "guard.threshold".to_string(),
            }],
            Some(MachineGuardPolicySpec {
                timeout_ms: Some(750),
                cooldown_ms: None,
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            }),
        );
        attach_risk_guard_descriptor(
            &mut bundle.machine_graphs[1],
            "risk_guard_readonly",
            vec![MachineGuardReadRef {
                source: MachineGuardReadSource::ReadonlyRuntimeFact,
                path: "runtime.mode".to_string(),
            }],
            Vec::new(),
            Vec::new(),
            None,
        );

        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 2);
        let combined = projections
            .iter()
            .find(|projection| projection.guard.guard.readiness.guard_id == "risk_guard_combined")
            .unwrap();
        assert_eq!(combined.guard.guard.readiness.read_count, 2);
        assert_eq!(combined.guard.guard.readiness.parameter_path_count, 2);
        assert_eq!(combined.guard.guard.readiness.condition_count, 1);
        assert!(combined.guard.guard.readiness.policy_declared);

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 2);
        assert_eq!(summary.guard_id_count, 2);
        assert_eq!(summary.read_guard_descriptor_count, 2);
        assert_eq!(summary.read_count, 3);
        assert_eq!(summary.event_payload_read_count, 1);
        assert_eq!(summary.machine_memory_read_count, 1);
        assert_eq!(summary.readonly_runtime_fact_read_count, 1);
        assert_eq!(summary.parameterized_guard_descriptor_count, 1);
        assert_eq!(summary.parameter_path_count, 2);
        assert_eq!(summary.threshold_parameter_path_count, 1);
        assert_eq!(summary.timeout_parameter_path_count, 1);
        assert_eq!(summary.parameter_path_proposal_only_count, 2);
        assert_eq!(summary.proposal_only_guard_descriptor_count, 1);
        assert_eq!(
            summary.parameter_path_active_strategy_write_disabled_count,
            2
        );
        assert_eq!(summary.conditional_guard_descriptor_count, 1);
        assert_eq!(summary.condition_count, 1);
        assert_eq!(summary.greater_than_or_equal_condition_count, 1);
        assert_eq!(summary.condition_event_payload_read_count, 1);
        assert_eq!(summary.condition_threshold_parameter_path_count, 1);
        assert_eq!(summary.condition_evaluation_enabled_count, 0);
        assert_eq!(
            summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(summary.condition_evaluation_disabled_fail_closed_count, 1);
        assert_eq!(summary.policy_declared_count, 1);
        assert_eq!(summary.timing_policy_declared_count, 1);
        assert_eq!(summary.timeout_declared_count, 1);
        assert_eq!(summary.fallback_declared_count, 1);
        assert_eq!(summary.fallback_fail_closed_declared_count, 1);
        assert_eq!(
            summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(
            summary.policy_timing_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(
            summary.policy_fallback_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(summary.policy_active_strategy_write_disabled_count, 1);
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            1
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 2);
    }

    #[test]
    fn static_contract_bundle_projects_guard_descriptor_fail_closed_blockers_across_surfaces() {
        fn attach_risk_guard_descriptor(
            graph: &mut V4MachineGraphContract,
            guard_id: &str,
            reads: Vec<MachineGuardReadRef>,
            parameter_paths: Vec<String>,
            conditions: Vec<MachineGuardConditionSpec>,
            policy: Option<MachineGuardPolicySpec>,
        ) {
            let risk = graph
                .machines
                .iter_mut()
                .find(|machine| machine.machine_id == "risk.guard")
                .unwrap();
            risk.transitions[0].guard = None;
            risk.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
                guard_id: guard_id.to_string(),
                reads,
                parameter_paths,
                conditions,
                policy,
                explanation: Some("bundle fail-closed blocker projection surface".to_string()),
            });
        }

        let mut bundle = sample_static_contract_bundle();
        bundle.machine_graphs[0].graph_id = "strategy.v4.alpha".to_string();
        let mut second_graph = sample_machine_graph();
        second_graph.graph_id = "strategy.v4.beta".to_string();
        bundle.machine_graphs.push(second_graph);
        attach_risk_guard_descriptor(
            &mut bundle.machine_graphs[0],
            "risk_guard_blocked",
            vec![MachineGuardReadRef {
                source: MachineGuardReadSource::ReadonlyRuntimeFact,
                path: "runtime.mode".to_string(),
            }],
            vec!["timeout.ms".to_string()],
            vec![MachineGuardConditionSpec {
                condition_id: "runtime_timeout_check".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::ReadonlyRuntimeFact,
                    path: "runtime.mode".to_string(),
                },
                comparator: MachineGuardConditionComparator::Equal,
                right_parameter_path: "timeout.ms".to_string(),
            }],
            Some(MachineGuardPolicySpec {
                timeout_ms: Some(500),
                cooldown_ms: None,
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            }),
        );
        attach_risk_guard_descriptor(
            &mut bundle.machine_graphs[1],
            "risk_guard_readonly",
            vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            Vec::new(),
            Vec::new(),
            None,
        );

        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 2);
        assert!(projections.iter().all(|projection| {
            projection.guard.guard.readiness.execution_state
                == MachineGuardExecutionReadinessState::DisabledFailClosed
                && projection.guard.guard.readiness.execution_blocker_code
                    == MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
                && projection.guard.guard.readiness.execution_blocker_reason
                    == MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        }));

        let blocked = projections
            .iter()
            .find(|projection| projection.guard.guard.readiness.guard_id == "risk_guard_blocked")
            .unwrap();
        let condition = &blocked.guard.guard.condition_projections[0];
        assert!(!condition.evaluation_enabled);
        assert_eq!(
            condition.evaluation_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            condition.evaluation_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );
        let policy = blocked.guard.guard.policy_projection.as_ref().unwrap();
        assert!(!policy.timing_execution_enabled);
        assert!(!policy.fallback_execution_enabled);
        assert!(!policy.active_strategy_write_enabled);
        assert_eq!(
            policy.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            policy.execution_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 2);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 2);
        assert_eq!(summary.condition_evaluation_enabled_count, 0);
        assert_eq!(summary.condition_evaluation_disabled_fail_closed_count, 1);
        assert_eq!(
            summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(summary.policy_timing_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_timing_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(summary.policy_fallback_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_fallback_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(summary.policy_active_strategy_write_enabled_count, 0);
        assert_eq!(summary.policy_active_strategy_write_disabled_count, 1);
        assert_eq!(
            summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            1
        );
    }

    #[test]
    fn static_contract_bundle_projects_guard_descriptors_in_input_order() {
        fn attach_guard_descriptor(
            graph: &mut V4MachineGraphContract,
            machine_id: &str,
            guard_id: &str,
        ) {
            let machine = graph
                .machines
                .iter_mut()
                .find(|machine| machine.machine_id == machine_id)
                .unwrap();
            machine.transitions[0].guard = None;
            machine.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
                guard_id: guard_id.to_string(),
                reads: Vec::new(),
                parameter_paths: Vec::new(),
                conditions: Vec::new(),
                policy: None,
                explanation: Some("bundle projection order surface".to_string()),
            });
        }

        let mut bundle = sample_static_contract_bundle();
        bundle.machine_graphs[0].graph_id = "strategy.v4.alpha".to_string();
        let mut second_graph = sample_machine_graph();
        second_graph.graph_id = "strategy.v4.beta".to_string();
        bundle.machine_graphs.push(second_graph);
        attach_guard_descriptor(
            &mut bundle.machine_graphs[0],
            "data.market",
            "alpha_observation_guard",
        );
        attach_guard_descriptor(
            &mut bundle.machine_graphs[0],
            "risk.guard",
            "alpha_risk_guard",
        );
        attach_guard_descriptor(
            &mut bundle.machine_graphs[1],
            "execution.router",
            "beta_execution_guard",
        );

        let projection_order = bundle
            .guard_descriptor_projections()
            .into_iter()
            .map(|projection| {
                (
                    projection.graph_id,
                    projection.guard.machine_id,
                    projection.guard.guard.readiness.guard_id,
                    projection.guard.guard.transition_id,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            projection_order,
            vec![
                (
                    "strategy.v4.alpha".to_string(),
                    "data.market".to_string(),
                    "alpha_observation_guard".to_string(),
                    "data.market.transition".to_string(),
                ),
                (
                    "strategy.v4.alpha".to_string(),
                    "risk.guard".to_string(),
                    "alpha_risk_guard".to_string(),
                    "risk.guard.transition".to_string(),
                ),
                (
                    "strategy.v4.beta".to_string(),
                    "execution.router".to_string(),
                    "beta_execution_guard".to_string(),
                    "execution.router.transition".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn static_contract_bundle_projects_child_machine_guard_descriptors_with_graph_context() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        risk.transitions[0].guard = None;
        risk.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "risk_parent_guard".to_string(),
            reads: Vec::new(),
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: Some("parent nested projection surface".to_string()),
        });

        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "risk_child_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::ReadonlyRuntimeFact,
                path: "runtime.mode".to_string(),
            }],
            parameter_paths: vec!["risk.max_position".to_string()],
            conditions: Vec::new(),
            policy: None,
            explanation: Some("child nested projection surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 2);
        assert_eq!(projections[0].graph_id, "strategy.v4.sample");
        assert_eq!(projections[0].guard.machine_id, "risk.guard");
        assert_eq!(
            projections[0].guard.guard.readiness.guard_id,
            "risk_parent_guard"
        );
        assert_eq!(projections[1].graph_id, "strategy.v4.sample");
        assert_eq!(projections[1].guard.machine_id, "risk.guard.child");
        assert_eq!(
            projections[1].guard.machine_template,
            MachineTemplateKind::Decision
        );
        assert_eq!(
            projections[1].guard.guard.transition_id,
            "risk.guard.child.transition"
        );
        assert_eq!(
            projections[1].guard.guard.readiness.guard_id,
            "risk_child_guard"
        );
        assert_eq!(projections[1].guard.guard.read_projections.len(), 1);
        assert_eq!(
            projections[1].guard.guard.read_projections[0].binding_scope,
            MachineGuardReadBindingScope::ReadonlyRuntimeFact
        );
        assert_eq!(
            projections[1].guard.guard.parameter_path_projections[0].kind,
            Some(MachineGuardParameterPathKind::RiskLimit)
        );
        assert!(projections[1].guard.guard.parameter_path_projections[0].proposal_only);
        assert!(
            !projections[1].guard.guard.parameter_path_projections[0].active_strategy_write_enabled
        );

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 2);
        assert_eq!(summary.guard_id_count, 2);
        assert_eq!(summary.guarded_machine_count, 2);
        assert_eq!(summary.decision_guard_descriptor_count, 2);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.readonly_runtime_fact_read_count, 1);
        assert_eq!(summary.parameterized_guard_descriptor_count, 1);
        assert_eq!(summary.risk_limit_parameter_path_count, 1);
        assert_eq!(summary.proposal_only_guard_descriptor_count, 1);
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            1
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 2);
    }

    #[test]
    fn static_contract_bundle_summarizes_child_guard_descriptor_event_source_context() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &[],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = None;
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "risk_child_missing_source_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: Some("child event source summary surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(bundle.validate_static_contract(), Ok(()));
        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        assert_eq!(projections[0].graph_id, "strategy.v4.sample");
        assert_eq!(projections[0].guard.machine_id, "risk.guard.child");
        assert_eq!(
            projections[0].guard.guard.readiness.guard_id,
            "risk_child_missing_source_guard"
        );
        assert_eq!(projections[0].guard.guard.event_type, "risk.child.check");
        assert_eq!(projections[0].guard.guard.event_source, None);

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.guarded_event_type_count, 1);
        assert_eq!(summary.guarded_event_source_count, 0);
        assert_eq!(summary.event_source_declared_count, 0);
        assert_eq!(summary.event_source_missing_count, 1);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.event_payload_read_count, 1);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn static_contract_bundle_summarizes_child_guard_descriptor_condition_operands() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();

        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "risk_child_condition_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: vec!["guard.threshold".to_string()],
            conditions: vec![MachineGuardConditionSpec {
                condition_id: "child_memory_threshold_check".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                comparator: MachineGuardConditionComparator::LessThanOrEqual,
                right_parameter_path: "guard.threshold".to_string(),
            }],
            policy: None,
            explanation: Some("child condition operand summary surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        assert_eq!(projections[0].graph_id, "strategy.v4.sample");
        assert_eq!(projections[0].guard.machine_id, "risk.guard.child");
        assert_eq!(
            projections[0].guard.guard.readiness.guard_id,
            "risk_child_condition_guard"
        );
        assert_eq!(projections[0].guard.guard.condition_projections.len(), 1);
        let condition = &projections[0].guard.guard.condition_projections[0];
        assert_eq!(condition.condition_id, "child_memory_threshold_check");
        assert_eq!(
            condition
                .left_read_projection
                .as_ref()
                .unwrap()
                .binding_scope,
            MachineGuardReadBindingScope::MachineMemoryField
        );
        assert_eq!(
            condition
                .right_parameter_path_projection
                .as_ref()
                .unwrap()
                .kind,
            Some(MachineGuardParameterPathKind::Threshold)
        );
        assert!(!condition.evaluation_enabled);
        assert_eq!(
            condition.evaluation_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.read_count, 1);
        assert_eq!(summary.machine_memory_read_count, 1);
        assert_eq!(summary.parameterized_guard_descriptor_count, 1);
        assert_eq!(summary.parameter_path_count, 1);
        assert_eq!(summary.threshold_parameter_path_count, 1);
        assert_eq!(summary.parameter_path_proposal_only_count, 1);
        assert_eq!(summary.conditional_guard_descriptor_count, 1);
        assert_eq!(summary.condition_count, 1);
        assert_eq!(summary.less_than_or_equal_condition_count, 1);
        assert_eq!(summary.condition_machine_memory_read_count, 1);
        assert_eq!(summary.condition_threshold_parameter_path_count, 1);
        assert_eq!(summary.condition_evaluation_enabled_count, 0);
        assert_eq!(
            summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(summary.condition_evaluation_disabled_fail_closed_count, 1);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn static_contract_bundle_summarizes_child_guard_descriptor_policy_surface() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();

        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "risk_child_policy_guard".to_string(),
            reads: Vec::new(),
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: Some(MachineGuardPolicySpec {
                timeout_ms: Some(250),
                cooldown_ms: Some(1_000),
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            }),
            explanation: Some("child policy summary surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        assert_eq!(projections[0].graph_id, "strategy.v4.sample");
        assert_eq!(projections[0].guard.machine_id, "risk.guard.child");
        assert_eq!(
            projections[0].guard.guard.readiness.guard_id,
            "risk_child_policy_guard"
        );
        assert!(projections[0].guard.guard.readiness.policy_declared);
        assert!(projections[0].guard.guard.readiness.timing_policy_declared);
        assert!(
            projections[0]
                .guard
                .guard
                .readiness
                .fallback_fail_closed_declared
        );
        let policy = projections[0]
            .guard
            .guard
            .policy_projection
            .as_ref()
            .unwrap();
        assert_eq!(policy.timeout_ms, Some(250));
        assert_eq!(policy.cooldown_ms, Some(1_000));
        assert_eq!(
            policy.fallback,
            Some(MachineGuardFallbackPolicy::FailClosed)
        );
        assert!(!policy.timing_execution_enabled);
        assert!(!policy.fallback_execution_enabled);
        assert!(!policy.active_strategy_write_enabled);
        assert_eq!(
            policy.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            policy.execution_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.policy_declared_count, 1);
        assert_eq!(summary.timing_policy_declared_count, 1);
        assert_eq!(summary.timeout_declared_count, 1);
        assert_eq!(summary.cooldown_declared_count, 1);
        assert_eq!(summary.fallback_declared_count, 1);
        assert_eq!(summary.fallback_fail_closed_declared_count, 1);
        assert_eq!(summary.policy_timing_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(
            summary.policy_timing_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(summary.policy_fallback_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_fallback_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(summary.policy_active_strategy_write_enabled_count, 0);
        assert_eq!(summary.policy_active_strategy_write_disabled_count, 1);
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            1
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn static_contract_bundle_projects_child_guard_descriptor_fail_closed_blockers() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();

        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "risk_child_blocked_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: vec!["timeout.ms".to_string()],
            conditions: vec![MachineGuardConditionSpec {
                condition_id: "child_timeout_memory_check".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                comparator: MachineGuardConditionComparator::GreaterThanOrEqual,
                right_parameter_path: "timeout.ms".to_string(),
            }],
            policy: Some(MachineGuardPolicySpec {
                timeout_ms: Some(500),
                cooldown_ms: None,
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            }),
            explanation: Some("child fail-closed blocker surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.graph_id, "strategy.v4.sample");
        assert_eq!(projection.guard.machine_id, "risk.guard.child");
        assert_eq!(
            projection.guard.guard.readiness.guard_id,
            "risk_child_blocked_guard"
        );
        assert_eq!(
            projection.guard.guard.readiness.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            projection.guard.guard.readiness.execution_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );

        let condition = &projection.guard.guard.condition_projections[0];
        assert!(!condition.evaluation_enabled);
        assert_eq!(
            condition.evaluation_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            condition.evaluation_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );

        let policy = projection.guard.guard.policy_projection.as_ref().unwrap();
        assert!(!policy.timing_execution_enabled);
        assert!(!policy.fallback_execution_enabled);
        assert!(!policy.active_strategy_write_enabled);
        assert_eq!(
            policy.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            policy.execution_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
        assert_eq!(summary.condition_evaluation_enabled_count, 0);
        assert_eq!(summary.condition_evaluation_disabled_fail_closed_count, 1);
        assert_eq!(
            summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(summary.policy_timing_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_timing_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(summary.policy_fallback_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_fallback_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(
            summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
            1
        );
    }

    #[test]
    fn static_contract_bundle_projects_child_guard_descriptor_parameter_path_context() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_child_parameter_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: vec![
                "timeout.ms".to_string(),
                "cooldown.ms".to_string(),
                "risk.max_position".to_string(),
            ],
            conditions: Vec::new(),
            policy: None,
            explanation: Some("bundle child parameter path projection surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(bundle.validate_static_contract(), Ok(()));
        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.graph_id, "strategy.v4.sample");
        assert_eq!(projection.guard.machine_id, "risk.guard.child");
        assert_eq!(
            projection.guard.guard.readiness.guard_id,
            "bundle_child_parameter_guard"
        );
        let parameter_paths = &projection.guard.guard.parameter_path_projections;
        assert_eq!(parameter_paths.len(), 3);
        assert_eq!(
            parameter_paths[0].kind,
            Some(MachineGuardParameterPathKind::Timeout)
        );
        assert_eq!(
            parameter_paths[1].kind,
            Some(MachineGuardParameterPathKind::Cooldown)
        );
        assert_eq!(
            parameter_paths[2].kind,
            Some(MachineGuardParameterPathKind::RiskLimit)
        );
        assert!(parameter_paths
            .iter()
            .all(|path| path.proposal_only && !path.active_strategy_write_enabled));

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.parameterized_guard_descriptor_count, 1);
        assert_eq!(summary.parameter_path_count, 3);
        assert_eq!(summary.timeout_parameter_path_count, 1);
        assert_eq!(summary.cooldown_parameter_path_count, 1);
        assert_eq!(summary.risk_limit_parameter_path_count, 1);
        assert_eq!(summary.parameter_path_proposal_only_count, 3);
        assert_eq!(summary.proposal_only_guard_descriptor_count, 1);
        assert_eq!(
            summary.parameter_path_active_strategy_write_enabled_count,
            0
        );
        assert_eq!(
            summary.parameter_path_active_strategy_write_disabled_count,
            3
        );
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            1
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn static_contract_bundle_projects_child_guard_descriptor_read_context() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_child_read_guard".to_string(),
            reads: vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "symbol".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::ReadonlyRuntimeFact,
                    path: "runtime.mode".to_string(),
                },
            ],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: Some("bundle child read projection surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(bundle.validate_static_contract(), Ok(()));
        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.graph_id, "strategy.v4.sample");
        assert_eq!(projection.guard.machine_id, "risk.guard.child");
        assert_eq!(
            projection.guard.guard.readiness.guard_id,
            "bundle_child_read_guard"
        );
        assert_eq!(projection.guard.guard.readiness.read_count, 3);
        assert_eq!(projection.guard.guard.read_projections.len(), 3);
        assert_eq!(
            projection.guard.guard.read_projections[0].source_label,
            "event_payload"
        );
        assert_eq!(
            projection.guard.guard.read_projections[0].binding_scope,
            MachineGuardReadBindingScope::EventPayloadField
        );
        assert_eq!(projection.guard.guard.read_projections[0].path, "symbol");
        assert_eq!(
            projection.guard.guard.read_projections[1].source_label,
            "machine_memory"
        );
        assert_eq!(
            projection.guard.guard.read_projections[1].binding_scope,
            MachineGuardReadBindingScope::MachineMemoryField
        );
        assert_eq!(
            projection.guard.guard.read_projections[1].path,
            "last_signal_at"
        );
        assert_eq!(
            projection.guard.guard.read_projections[2].source_label,
            "readonly_runtime_fact"
        );
        assert_eq!(
            projection.guard.guard.read_projections[2].binding_scope,
            MachineGuardReadBindingScope::ReadonlyRuntimeFact
        );
        assert_eq!(
            projection.guard.guard.read_projections[2].path,
            "runtime.mode"
        );

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.read_count, 3);
        assert_eq!(summary.event_payload_read_count, 1);
        assert_eq!(summary.machine_memory_read_count, 1);
        assert_eq!(summary.readonly_runtime_fact_read_count, 1);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn static_contract_bundle_projects_child_guard_descriptor_transition_context() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_child_transition_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: Some("bundle child transition projection surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(bundle.validate_static_contract(), Ok(()));
        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.graph_id, "strategy.v4.sample");
        assert_eq!(projection.guard.machine_id, "risk.guard.child");
        assert_eq!(
            projection.guard.machine_template,
            MachineTemplateKind::Decision
        );
        assert_eq!(
            projection.guard.guard.transition_id,
            "risk.guard.child.transition"
        );
        assert_eq!(projection.guard.guard.from_state, "idle");
        assert_eq!(projection.guard.guard.to_state, "long_bias");
        assert_eq!(projection.guard.guard.event_type, "risk.child.check");
        assert_eq!(
            projection.guard.guard.event_source.as_deref(),
            Some("risk.guard")
        );
        assert_eq!(
            projection.guard.guard.readiness.guard_id,
            "bundle_child_transition_guard"
        );

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guard_id_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.guarded_transition_count, 1);
        assert_eq!(summary.guarded_event_type_count, 1);
        assert_eq!(summary.guarded_event_source_count, 1);
        assert_eq!(summary.event_source_declared_count, 1);
        assert_eq!(summary.event_source_missing_count, 0);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.read_count, 1);
        assert_eq!(summary.machine_memory_read_count, 1);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn machine_contract_rejects_invalid_structured_guard_descriptor() {
        let mut machine = sample_machine();
        machine.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "".to_string(),
            reads: vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "unknown_memory".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::ReadonlyRuntimeFact,
                    path: "provider.secret".to_string(),
                },
            ],
            parameter_paths: vec!["".to_string(), "graph.edges".to_string()],
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });

        let errors = machine.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("structured guard must declare guard_id")));
        assert!(errors
            .iter()
            .any(|message| message.contains("has an empty read path")));
        assert!(errors
            .iter()
            .any(|message| message.contains("reads undeclared memory field `unknown_memory`")));
        assert!(errors.iter().any(|message| {
            message.contains("reads unknown readonly runtime fact `provider.secret`")
        }));
        assert!(errors
            .iter()
            .any(|message| message.contains("has an empty parameter path")));
        assert!(errors.iter().any(|message| {
            message.contains("parameter path `graph.edges`")
                && message.contains("outside the proposal-only guard boundary")
        }));
    }

    #[test]
    fn machine_contract_rejects_duplicate_structured_guard_descriptor_inputs() {
        let mut machine = sample_machine();
        machine.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "duplicate_guard".to_string(),
            reads: vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
            ],
            parameter_paths: vec!["guard.threshold".to_string(), "GUARD.THRESHOLD".to_string()],
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });

        let errors = machine.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("declares duplicate machine_memory read `last_signal_at`")
        }));
        assert!(
            errors
                .iter()
                .any(|message| message
                    .contains("declares duplicate parameter path `GUARD.THRESHOLD`"))
        );
    }

    #[test]
    fn machine_contract_rejects_invalid_structured_guard_descriptor_conditions() {
        let mut machine = sample_machine();
        machine.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "condition_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: vec!["guard.threshold".to_string()],
            conditions: vec![
                MachineGuardConditionSpec {
                    condition_id: "".to_string(),
                    left_read: MachineGuardReadRef {
                        source: MachineGuardReadSource::MachineMemory,
                        path: "last_signal_at".to_string(),
                    },
                    comparator: MachineGuardConditionComparator::GreaterThan,
                    right_parameter_path: "guard.threshold".to_string(),
                },
                MachineGuardConditionSpec {
                    condition_id: "missing_read".to_string(),
                    left_read: MachineGuardReadRef {
                        source: MachineGuardReadSource::EventPayload,
                        path: "missing_payload".to_string(),
                    },
                    comparator: MachineGuardConditionComparator::Equal,
                    right_parameter_path: "guard.threshold".to_string(),
                },
                MachineGuardConditionSpec {
                    condition_id: "missing_parameter".to_string(),
                    left_read: MachineGuardReadRef {
                        source: MachineGuardReadSource::MachineMemory,
                        path: "last_signal_at".to_string(),
                    },
                    comparator: MachineGuardConditionComparator::LessThan,
                    right_parameter_path: "guard.missing".to_string(),
                },
            ],
            policy: None,
            explanation: None,
        });

        let errors = machine.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("has a condition without condition_id")));
        assert!(errors.iter().any(|message| {
            message.contains("condition `missing_read`")
                && message.contains("references undeclared event_payload read `missing_payload`")
        }));
        assert!(errors.iter().any(|message| {
            message.contains("condition `missing_parameter`")
                && message.contains("references undeclared parameter path `guard.missing`")
        }));
    }

    #[test]
    fn machine_contract_rejects_invalid_structured_guard_descriptor_policy() {
        let mut machine = sample_machine();
        machine.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "policy_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: vec!["cooldown.ms".to_string()],
            conditions: Vec::new(),
            policy: Some(MachineGuardPolicySpec {
                timeout_ms: Some(0),
                cooldown_ms: Some(0),
                fallback: None,
            }),
            explanation: None,
        });

        let errors = machine.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("timeout_ms must be greater than zero")));
        assert!(errors
            .iter()
            .any(|message| message.contains("cooldown_ms must be greater than zero")));

        let descriptor = machine.transitions[0].guard_descriptor.as_mut().unwrap();
        descriptor.policy = Some(MachineGuardPolicySpec {
            timeout_ms: None,
            cooldown_ms: None,
            fallback: None,
        });

        let errors = machine.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message
                .contains("policy must declare timeout_ms, cooldown_ms, or fallback")));
    }

    #[test]
    fn machine_contract_rejects_unknown_transition_state() {
        let mut machine = sample_machine();
        machine.transitions[0].to_state = "nested.child".to_string();

        let errors = machine.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("unknown to_state")));
    }

    #[test]
    fn machine_graph_accepts_top_level_dag_with_risk_plane() {
        let graph = sample_machine_graph();

        assert_eq!(graph.validate_static_contract(), Ok(()));
    }

    #[test]
    fn machine_graph_accepts_guard_descriptor_event_payload_read_from_catalog() {
        let mut graph = sample_machine_graph();
        let intent = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "intent.trend")
            .unwrap();
        intent.transitions[0].guard = None;
        intent.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bar_symbol_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            parameter_paths: vec!["guard.allowed_symbol".to_string()],
            conditions: Vec::new(),
            policy: None,
            explanation: Some("guard reads a declared event payload field".to_string()),
        });

        assert_eq!(graph.validate_static_contract(), Ok(()));
    }

    #[test]
    fn machine_graph_accepts_child_guard_descriptor_event_payload_read_from_catalog() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_payload_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: Some("child guard reads a declared event payload field".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(graph.validate_static_contract(), Ok(()));
    }

    #[test]
    fn machine_graph_projects_child_guard_descriptor_event_source_context() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_source_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: Some("child graph event source projection surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(graph.validate_static_contract(), Ok(()));
        let projections = graph.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.machine_id, "risk.guard.child");
        assert_eq!(projection.machine_template, MachineTemplateKind::Decision);
        assert_eq!(
            projection.guard.transition_id,
            "risk.guard.child.transition"
        );
        assert_eq!(projection.guard.event_type, "risk.child.check");
        assert_eq!(projection.guard.event_source.as_deref(), Some("risk.guard"));
        assert_eq!(
            projection.guard.readiness.guard_id,
            "child_graph_source_guard"
        );
        assert_eq!(projection.guard.read_projections.len(), 1);
        assert_eq!(
            projection.guard.read_projections[0].binding_scope,
            MachineGuardReadBindingScope::EventPayloadField
        );

        let summary = graph.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guard_id_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.guarded_transition_count, 1);
        assert_eq!(summary.guarded_event_type_count, 1);
        assert_eq!(summary.guarded_event_source_count, 1);
        assert_eq!(summary.event_source_declared_count, 1);
        assert_eq!(summary.event_source_missing_count, 0);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.event_payload_read_count, 1);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn machine_graph_projects_child_guard_descriptor_transition_context() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_transition_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: Some("child graph transition projection surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(graph.validate_static_contract(), Ok(()));
        let projections = graph.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.machine_id, "risk.guard.child");
        assert_eq!(projection.machine_template, MachineTemplateKind::Decision);
        assert_eq!(
            projection.guard.transition_id,
            "risk.guard.child.transition"
        );
        assert_eq!(projection.guard.from_state, "idle");
        assert_eq!(projection.guard.to_state, "long_bias");
        assert_eq!(projection.guard.event_type, "risk.child.check");
        assert_eq!(projection.guard.event_source.as_deref(), Some("risk.guard"));
        assert_eq!(
            projection.guard.readiness.guard_id,
            "child_graph_transition_guard"
        );

        let summary = graph.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guard_id_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.guarded_transition_count, 1);
        assert_eq!(summary.guarded_event_type_count, 1);
        assert_eq!(summary.guarded_event_source_count, 1);
        assert_eq!(summary.event_source_declared_count, 1);
        assert_eq!(summary.event_source_missing_count, 0);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.read_count, 1);
        assert_eq!(summary.machine_memory_read_count, 1);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn machine_graph_summarizes_child_guard_descriptor_missing_event_source_context() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &[],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = None;
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_missing_source_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: Some("child graph missing event source summary surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(graph.validate_static_contract(), Ok(()));
        let projections = graph.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.machine_id, "risk.guard.child");
        assert_eq!(projection.guard.event_type, "risk.child.check");
        assert_eq!(projection.guard.event_source, None);
        assert_eq!(
            projection.guard.readiness.guard_id,
            "child_graph_missing_source_guard"
        );

        let summary = graph.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.guarded_event_type_count, 1);
        assert_eq!(summary.guarded_event_source_count, 0);
        assert_eq!(summary.event_source_declared_count, 0);
        assert_eq!(summary.event_source_missing_count, 1);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.event_payload_read_count, 1);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn machine_graph_projects_child_guard_descriptor_read_context() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_read_guard".to_string(),
            reads: vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "symbol".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::ReadonlyRuntimeFact,
                    path: "runtime.mode".to_string(),
                },
            ],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: Some("child graph read projection surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(graph.validate_static_contract(), Ok(()));
        let projections = graph.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.machine_id, "risk.guard.child");
        assert_eq!(
            projection.guard.readiness.guard_id,
            "child_graph_read_guard"
        );
        assert_eq!(projection.guard.readiness.read_count, 3);
        assert_eq!(projection.guard.read_projections.len(), 3);
        assert_eq!(
            projection.guard.read_projections[0].source_label,
            "event_payload"
        );
        assert_eq!(
            projection.guard.read_projections[0].binding_scope,
            MachineGuardReadBindingScope::EventPayloadField
        );
        assert_eq!(projection.guard.read_projections[0].path, "symbol");
        assert_eq!(
            projection.guard.read_projections[1].source_label,
            "machine_memory"
        );
        assert_eq!(
            projection.guard.read_projections[1].binding_scope,
            MachineGuardReadBindingScope::MachineMemoryField
        );
        assert_eq!(projection.guard.read_projections[1].path, "last_signal_at");
        assert_eq!(
            projection.guard.read_projections[2].source_label,
            "readonly_runtime_fact"
        );
        assert_eq!(
            projection.guard.read_projections[2].binding_scope,
            MachineGuardReadBindingScope::ReadonlyRuntimeFact
        );
        assert_eq!(projection.guard.read_projections[2].path, "runtime.mode");

        let summary = graph.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.read_count, 3);
        assert_eq!(summary.event_payload_read_count, 1);
        assert_eq!(summary.machine_memory_read_count, 1);
        assert_eq!(summary.readonly_runtime_fact_read_count, 1);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn machine_graph_summarizes_child_guard_descriptor_condition_operands() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_condition_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: vec!["guard.threshold".to_string()],
            conditions: vec![MachineGuardConditionSpec {
                condition_id: "child_graph_memory_threshold_check".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                comparator: MachineGuardConditionComparator::LessThanOrEqual,
                right_parameter_path: "guard.threshold".to_string(),
            }],
            policy: None,
            explanation: Some("child graph condition operand summary surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(graph.validate_static_contract(), Ok(()));
        let projections = graph.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.machine_id, "risk.guard.child");
        assert_eq!(
            projection.guard.readiness.guard_id,
            "child_graph_condition_guard"
        );
        assert_eq!(projection.guard.condition_projections.len(), 1);
        let condition = &projection.guard.condition_projections[0];
        assert_eq!(condition.condition_id, "child_graph_memory_threshold_check");
        assert_eq!(
            condition
                .left_read_projection
                .as_ref()
                .unwrap()
                .binding_scope,
            MachineGuardReadBindingScope::MachineMemoryField
        );
        assert_eq!(
            condition
                .right_parameter_path_projection
                .as_ref()
                .unwrap()
                .kind,
            Some(MachineGuardParameterPathKind::Threshold)
        );
        assert!(!condition.evaluation_enabled);
        assert_eq!(
            condition.evaluation_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );

        let summary = graph.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.read_count, 1);
        assert_eq!(summary.machine_memory_read_count, 1);
        assert_eq!(summary.parameterized_guard_descriptor_count, 1);
        assert_eq!(summary.parameter_path_count, 1);
        assert_eq!(summary.threshold_parameter_path_count, 1);
        assert_eq!(summary.parameter_path_proposal_only_count, 1);
        assert_eq!(summary.conditional_guard_descriptor_count, 1);
        assert_eq!(summary.condition_count, 1);
        assert_eq!(summary.less_than_or_equal_condition_count, 1);
        assert_eq!(summary.condition_machine_memory_read_count, 1);
        assert_eq!(summary.condition_threshold_parameter_path_count, 1);
        assert_eq!(summary.condition_evaluation_enabled_count, 0);
        assert_eq!(
            summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(summary.condition_evaluation_disabled_fail_closed_count, 1);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn machine_graph_summarizes_child_guard_descriptor_policy_surface() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_policy_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: Some(MachineGuardPolicySpec {
                timeout_ms: Some(250),
                cooldown_ms: Some(1_000),
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            }),
            explanation: Some("child graph policy summary surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(graph.validate_static_contract(), Ok(()));
        let projections = graph.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.machine_id, "risk.guard.child");
        assert_eq!(
            projection.guard.readiness.guard_id,
            "child_graph_policy_guard"
        );
        assert!(projection.guard.readiness.policy_declared);
        assert!(projection.guard.readiness.timing_policy_declared);
        assert!(projection.guard.readiness.fallback_fail_closed_declared);
        let policy = projection.guard.policy_projection.as_ref().unwrap();
        assert!(!policy.timing_execution_enabled);
        assert!(!policy.fallback_execution_enabled);
        assert!(!policy.active_strategy_write_enabled);
        assert_eq!(
            policy.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );

        let summary = graph.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.read_count, 1);
        assert_eq!(summary.machine_memory_read_count, 1);
        assert_eq!(summary.policy_declared_count, 1);
        assert_eq!(summary.timing_policy_declared_count, 1);
        assert_eq!(summary.timeout_declared_count, 1);
        assert_eq!(summary.cooldown_declared_count, 1);
        assert_eq!(summary.fallback_declared_count, 1);
        assert_eq!(summary.fallback_fail_closed_declared_count, 1);
        assert_eq!(
            summary.policy_timing_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(
            summary.policy_fallback_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(
            summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(summary.policy_active_strategy_write_enabled_count, 0);
        assert_eq!(summary.policy_active_strategy_write_disabled_count, 1);
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            1
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn machine_graph_accepts_child_guard_descriptor_full_static_surface() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].guard = None;
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_full_static_guard".to_string(),
            reads: vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "symbol".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::ReadonlyRuntimeFact,
                    path: "runtime.mode".to_string(),
                },
            ],
            parameter_paths: vec!["guard.threshold".to_string(), "timeout.ms".to_string()],
            conditions: vec![MachineGuardConditionSpec {
                condition_id: "child_graph_runtime_timeout_check".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::ReadonlyRuntimeFact,
                    path: "runtime.mode".to_string(),
                },
                comparator: MachineGuardConditionComparator::Equal,
                right_parameter_path: "timeout.ms".to_string(),
            }],
            policy: Some(MachineGuardPolicySpec {
                timeout_ms: Some(500),
                cooldown_ms: Some(1_000),
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            }),
            explanation: Some("child graph full static guard descriptor surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(graph.validate_static_contract(), Ok(()));
        let projections = graph.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.machine_id, "risk.guard.child");
        assert_eq!(projection.guard.event_type, "risk.child.check");
        assert_eq!(projection.guard.event_source.as_deref(), Some("risk.guard"));
        assert_eq!(
            projection.guard.readiness.guard_id,
            "child_graph_full_static_guard"
        );
        assert_eq!(projection.guard.readiness.read_count, 3);
        assert_eq!(projection.guard.readiness.parameter_path_count, 2);
        assert_eq!(projection.guard.readiness.condition_count, 1);
        assert!(projection.guard.readiness.policy_declared);
        assert!(!projection.guard.readiness.execution_enabled);
        assert_eq!(
            projection.guard.readiness.execution_state,
            MachineGuardExecutionReadinessState::DisabledFailClosed
        );
        assert_eq!(
            projection.guard.readiness.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(projection.guard.read_projections.len(), 3);
        assert_eq!(
            projection.guard.read_projections[0].binding_scope,
            MachineGuardReadBindingScope::EventPayloadField
        );
        assert_eq!(
            projection.guard.read_projections[1].binding_scope,
            MachineGuardReadBindingScope::MachineMemoryField
        );
        assert_eq!(
            projection.guard.read_projections[2].binding_scope,
            MachineGuardReadBindingScope::ReadonlyRuntimeFact
        );
        let condition = &projection.guard.condition_projections[0];
        assert!(!condition.evaluation_enabled);
        assert_eq!(
            condition.evaluation_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        let policy = projection.guard.policy_projection.as_ref().unwrap();
        assert!(!policy.timing_execution_enabled);
        assert!(!policy.fallback_execution_enabled);
        assert!(!policy.active_strategy_write_enabled);

        let summary = graph.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guard_id_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.guarded_transition_count, 1);
        assert_eq!(summary.guarded_event_type_count, 1);
        assert_eq!(summary.guarded_event_source_count, 1);
        assert_eq!(summary.event_source_declared_count, 1);
        assert_eq!(summary.event_source_missing_count, 0);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.read_count, 3);
        assert_eq!(summary.event_payload_read_count, 1);
        assert_eq!(summary.machine_memory_read_count, 1);
        assert_eq!(summary.readonly_runtime_fact_read_count, 1);
        assert_eq!(summary.parameterized_guard_descriptor_count, 1);
        assert_eq!(summary.parameter_path_count, 2);
        assert_eq!(summary.threshold_parameter_path_count, 1);
        assert_eq!(summary.timeout_parameter_path_count, 1);
        assert_eq!(summary.proposal_only_guard_descriptor_count, 1);
        assert_eq!(summary.conditional_guard_descriptor_count, 1);
        assert_eq!(summary.condition_readonly_runtime_fact_read_count, 1);
        assert_eq!(summary.condition_timeout_parameter_path_count, 1);
        assert_eq!(
            summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(summary.policy_declared_count, 1);
        assert_eq!(summary.timeout_declared_count, 1);
        assert_eq!(summary.cooldown_declared_count, 1);
        assert_eq!(summary.fallback_fail_closed_declared_count, 1);
        assert_eq!(
            summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            1
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn machine_graph_projects_child_guard_descriptor_fail_closed_blockers() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_blocked_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: vec!["timeout.ms".to_string()],
            conditions: vec![MachineGuardConditionSpec {
                condition_id: "child_graph_timeout_memory_check".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                comparator: MachineGuardConditionComparator::GreaterThanOrEqual,
                right_parameter_path: "timeout.ms".to_string(),
            }],
            policy: Some(MachineGuardPolicySpec {
                timeout_ms: Some(500),
                cooldown_ms: None,
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            }),
            explanation: Some("child graph fail-closed blocker surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(graph.validate_static_contract(), Ok(()));
        let projections = graph.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.machine_id, "risk.guard.child");
        assert_eq!(
            projection.guard.readiness.guard_id,
            "child_graph_blocked_guard"
        );
        assert_eq!(
            projection.guard.readiness.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            projection.guard.readiness.execution_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );

        let condition = &projection.guard.condition_projections[0];
        assert!(!condition.evaluation_enabled);
        assert_eq!(
            condition.evaluation_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            condition.evaluation_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );

        let policy = projection.guard.policy_projection.as_ref().unwrap();
        assert!(!policy.timing_execution_enabled);
        assert!(!policy.fallback_execution_enabled);
        assert!(!policy.active_strategy_write_enabled);
        assert_eq!(
            policy.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(
            policy.execution_blocker_reason,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
        );

        let summary = graph.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
        assert_eq!(summary.condition_evaluation_enabled_count, 0);
        assert_eq!(summary.condition_evaluation_disabled_fail_closed_count, 1);
        assert_eq!(
            summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(summary.policy_timing_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_timing_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(summary.policy_fallback_execution_enabled_count, 0);
        assert_eq!(
            summary.policy_fallback_execution_disabled_fail_closed_count,
            1
        );
        assert_eq!(
            summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
            1
        );
    }

    #[test]
    fn machine_graph_projects_child_guard_descriptor_parameter_path_context() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_parameter_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: vec![
                "timeout.ms".to_string(),
                "cooldown.ms".to_string(),
                "risk.max_position".to_string(),
            ],
            conditions: Vec::new(),
            policy: None,
            explanation: Some("child graph parameter path projection surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(graph.validate_static_contract(), Ok(()));
        let projections = graph.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.machine_id, "risk.guard.child");
        assert_eq!(
            projection.guard.readiness.guard_id,
            "child_graph_parameter_guard"
        );
        let parameter_paths = &projection.guard.parameter_path_projections;
        assert_eq!(parameter_paths.len(), 3);
        assert_eq!(
            parameter_paths[0].kind,
            Some(MachineGuardParameterPathKind::Timeout)
        );
        assert_eq!(
            parameter_paths[1].kind,
            Some(MachineGuardParameterPathKind::Cooldown)
        );
        assert_eq!(
            parameter_paths[2].kind,
            Some(MachineGuardParameterPathKind::RiskLimit)
        );
        assert!(parameter_paths
            .iter()
            .all(|path| path.proposal_only && !path.active_strategy_write_enabled));

        let summary = graph.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.parameterized_guard_descriptor_count, 1);
        assert_eq!(summary.parameter_path_count, 3);
        assert_eq!(summary.timeout_parameter_path_count, 1);
        assert_eq!(summary.cooldown_parameter_path_count, 1);
        assert_eq!(summary.risk_limit_parameter_path_count, 1);
        assert_eq!(summary.parameter_path_proposal_only_count, 3);
        assert_eq!(summary.proposal_only_guard_descriptor_count, 1);
        assert_eq!(
            summary.parameter_path_active_strategy_write_enabled_count,
            0
        );
        assert_eq!(
            summary.parameter_path_active_strategy_write_disabled_count,
            3
        );
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            1
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn machine_graph_rejects_guard_descriptor_unknown_event_payload_read() {
        let mut graph = sample_machine_graph();
        let intent = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "intent.trend")
            .unwrap();
        intent.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "missing_payload_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "missing_payload".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("structured guard `missing_payload_guard`")
                && message.contains("unknown event payload field `missing_payload`")
                && message.contains("event `bar_closed`")
        }));
    }

    #[test]
    fn machine_graph_rejects_child_guard_descriptor_unknown_event_payload_read() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "missing_child_payload_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "missing_payload".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("machine `risk.guard.child`")
                && message.contains("structured guard `missing_child_payload_guard`")
                && message.contains("unknown event payload field `missing_payload`")
                && message.contains("event `risk.child.check`")
        }));
    }

    #[test]
    fn machine_graph_rejects_child_guard_descriptor_duplicate_inputs() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_duplicate_input_guard".to_string(),
            reads: vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
            ],
            parameter_paths: vec!["guard.threshold".to_string(), "GUARD.THRESHOLD".to_string()],
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `child_graph_duplicate_input_guard`")
                && message.contains("declares duplicate machine_memory read `last_signal_at`")
        }));
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `child_graph_duplicate_input_guard`")
                && message.contains("declares duplicate parameter path `GUARD.THRESHOLD`")
        }));
    }

    #[test]
    fn machine_graph_rejects_child_guard_descriptor_unknown_readonly_runtime_fact() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_unknown_runtime_fact_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::ReadonlyRuntimeFact,
                path: "provider.secret".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `child_graph_unknown_runtime_fact_guard`")
                && message.contains("reads unknown readonly runtime fact `provider.secret`")
        }));
    }

    #[test]
    fn machine_graph_rejects_child_guard_descriptor_unknown_machine_memory_read() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_unknown_memory_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "unknown_memory".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `child_graph_unknown_memory_guard`")
                && message.contains("reads undeclared memory field `unknown_memory`")
        }));
    }

    #[test]
    fn machine_graph_rejects_child_guard_descriptor_base_hygiene_violations() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "".to_string(),
            }],
            parameter_paths: vec!["".to_string()],
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard must declare guard_id")
        }));
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("has an empty read path")
        }));
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("has an empty parameter path")
        }));
    }

    #[test]
    fn machine_graph_rejects_child_guard_descriptor_event_party_violations() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["other.risk"],
                &["other.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_event_party_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("machine `risk.guard.child`")
                && message.contains("transition `risk.guard.child.transition`")
                && message.contains("is not an allowed consumer")
                && message.contains("event `risk.child.check`")
        }));
        assert!(errors.iter().any(|message| {
            message.contains("machine `risk.guard.child`")
                && message.contains("transition `risk.guard.child.transition`")
                && message.contains("source `risk.guard` is not an allowed emitter")
                && message.contains("event `risk.child.check`")
        }));
    }

    #[test]
    fn machine_graph_rejects_child_guard_descriptor_unknown_transition_event() {
        let mut graph = sample_machine_graph();
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.missing".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_unknown_event_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("machine `risk.guard.child`")
                && message.contains("transition `risk.guard.child.transition`")
                && message.contains("event_type `risk.child.missing`")
                && message.contains("must be declared in event_catalog")
        }));
    }

    #[test]
    fn machine_graph_rejects_child_guard_descriptor_forbidden_parameter_path() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_forbidden_parameter_guard".to_string(),
            reads: Vec::new(),
            parameter_paths: vec!["active_strategy.position".to_string()],
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `child_graph_forbidden_parameter_guard`")
                && message.contains("parameter path `active_strategy.position`")
                && message.contains("outside the proposal-only guard boundary")
        }));
    }

    #[test]
    fn machine_graph_rejects_child_guard_descriptor_invalid_condition_operand() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "child_graph_invalid_condition_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: vec!["guard.threshold".to_string()],
            conditions: vec![MachineGuardConditionSpec {
                condition_id: "missing_child_parameter".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                comparator: MachineGuardConditionComparator::LessThan,
                right_parameter_path: "guard.missing".to_string(),
            }],
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `child_graph_invalid_condition_guard`")
                && message.contains("condition `missing_child_parameter`")
                && message.contains("references undeclared parameter path `guard.missing`")
        }));
    }

    #[test]
    fn machine_graph_rejects_cycle() {
        let mut graph = sample_machine_graph();
        graph.edges.push(sample_graph_edge(
            "execution.router",
            "intent.trend",
            "risk.approved",
        ));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("machine graph must be acyclic")));
    }

    #[test]
    fn machine_graph_rejects_unknown_edge_target() {
        let mut graph = sample_machine_graph();
        graph.edges[0].target_machine_id = "missing.machine".to_string();

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("unknown target_machine_id")));
    }

    #[test]
    fn machine_graph_requires_risk_plane_for_execution() {
        let mut graph = sample_machine_graph();
        graph.risk_plane = None;

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("dedicated risk_plane")));
    }

    #[test]
    fn machine_graph_rejects_execution_bypass_edge() {
        let mut graph = sample_machine_graph();
        graph.edges.push(sample_graph_edge(
            "intent.trend",
            "execution.router",
            "intent.long",
        ));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("must originate from risk_plane")));
    }

    #[test]
    fn machine_graph_requires_high_priority_decision_risk_machine() {
        let mut graph = sample_machine_graph();
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        risk.priority = 100;

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("below min_priority")));
    }

    #[test]
    fn machine_event_catalog_accepts_strong_events() {
        let catalog = sample_event_catalog();

        assert_eq!(catalog.validate_static_contract(), Ok(()));
    }

    #[test]
    fn machine_event_catalog_rejects_untyped_payload_field() {
        let mut catalog = sample_event_catalog();
        catalog.events[0].payload_fields[0].type_name.clear();

        let errors = catalog.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("must declare a type_name")));
    }

    #[test]
    fn machine_graph_requires_event_catalog_for_events() {
        let mut graph = sample_machine_graph();
        graph.event_catalog = None;

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("must declare event_catalog")));
    }

    #[test]
    fn machine_graph_rejects_unknown_transition_event() {
        let mut graph = sample_machine_graph();
        graph.machines[0].transitions[0].event.event_type = "unknown.event".to_string();

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("must be declared in event_catalog")));
    }

    #[test]
    fn machine_graph_rejects_event_emitter_not_allowed() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .iter_mut()
            .find(|event| event.event_type == "risk.approved")
            .unwrap()
            .allowed_emitters = vec!["other.risk".to_string()];

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("not an allowed emitter")));
    }

    #[test]
    fn qs_state_machine_profile_default_is_valid() {
        let profile = default_v4_qs_state_machine_profile();

        assert_eq!(profile.validate_static_contract(), Ok(()));
        assert!(profile.state_policy.allow_state_groups);
        assert!(profile.state_policy.allow_nested_state_machines);
        assert!(
            profile
                .risk_plane_policy
                .dedicated_high_priority_risk_plane_required
        );
    }

    #[test]
    fn qs_state_machine_profile_requires_all_three_templates() {
        let mut profile = default_v4_qs_state_machine_profile();
        profile
            .allowed_templates
            .retain(|template| !matches!(template, MachineTemplateKind::Execution));

        let errors = profile.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| { message.contains("must allow") && message.contains("Execution") }));
    }

    #[test]
    fn qs_state_machine_profile_rejects_direct_order_submit() {
        let mut profile = default_v4_qs_state_machine_profile();
        profile.action_block_policy.allow_direct_order_submit = true;

        let errors = profile.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("must not submit orders directly")));
    }

    #[test]
    fn qs_state_machine_profile_requires_nested_state_machines_enabled() {
        let mut profile = default_v4_qs_state_machine_profile();
        profile.state_policy.allow_nested_state_machines = false;

        let errors = profile.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("nested state machines")));
    }

    #[test]
    fn qs_state_machine_profile_requires_high_priority_risk_plane() {
        let mut profile = default_v4_qs_state_machine_profile();
        profile
            .risk_plane_policy
            .dedicated_high_priority_risk_plane_required = false;

        let errors = profile.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("high-priority risk safety plane")));
    }

    #[test]
    fn runtime_mode_contract_default_is_valid() {
        let contract = default_v4_runtime_mode_contract();

        assert_eq!(contract.validate_static_contract(), Ok(()));
        assert_eq!(
            contract.settlement_authority_for(RuntimeTradingMode::LiveSimulated),
            Some(RuntimeSettlementAuthority::LocalSimulated)
        );
    }

    #[test]
    fn runtime_mode_contract_requires_all_four_modes() {
        let mut contract = default_v4_runtime_mode_contract();
        contract
            .modes
            .retain(|spec| spec.mode != RuntimeTradingMode::LiveSimulated);

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("LiveSimulated")));
    }

    #[test]
    fn runtime_mode_contract_rejects_live_simulated_provider_submission() {
        let mut contract = default_v4_runtime_mode_contract();
        let live_simulated = contract
            .modes
            .iter_mut()
            .find(|spec| spec.mode == RuntimeTradingMode::LiveSimulated)
            .unwrap();
        live_simulated.provider_order_submission_allowed = true;

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("provider_order_submission_allowed")));
    }

    #[test]
    fn runtime_mode_contract_requires_execution_events() {
        let mut contract = default_v4_runtime_mode_contract();
        contract.modes[0]
            .required_events
            .retain(|event| *event != RuntimeExecutionEventKind::FeeCharged);

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| message.contains("FeeCharged")));
    }

    #[test]
    fn qs_type_system_contract_default_is_valid() {
        let contract = default_v4_qs_type_system_contract();

        assert_eq!(contract.validate_static_contract(), Ok(()));
        assert_eq!(
            contract.validate_type_ref(&QsTypeRef::Fresh {
                inner: Box::new(QsTypeRef::List {
                    item: Box::new(QsTypeRef::Scalar {
                        scalar: QsScalarTypeKind::Price,
                    }),
                    max_items: 256,
                }),
            }),
            Ok(())
        );
    }

    #[test]
    fn qs_type_system_contract_requires_first_wave_scalar_types() {
        let mut contract = default_v4_qs_type_system_contract();
        contract
            .scalar_types
            .retain(|scalar| *scalar != QsScalarTypeKind::RuntimeMode);

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| message.contains("RuntimeMode")));
    }

    #[test]
    fn qs_type_system_contract_rejects_duplicate_composite_types() {
        let mut contract = default_v4_qs_type_system_contract();
        contract
            .composite_types
            .push(contract.composite_types[0].clone());

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("duplicate composite type")));
    }

    #[test]
    fn qs_type_system_rejects_unbounded_list_ref() {
        let contract = default_v4_qs_type_system_contract();

        let errors = contract
            .validate_type_ref(&QsTypeRef::List {
                item: Box::new(QsTypeRef::Scalar {
                    scalar: QsScalarTypeKind::Symbol,
                }),
                max_items: 0,
            })
            .unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("requires max_items greater than 0")));
    }

    #[test]
    fn qs_type_system_rejects_over_budget_map_ref() {
        let contract = default_v4_qs_type_system_contract();

        let errors = contract
            .validate_type_ref(&QsTypeRef::Map {
                key: QsScalarTypeKind::Symbol,
                value: Box::new(QsTypeRef::Scalar {
                    scalar: QsScalarTypeKind::Decimal,
                }),
                max_items: 10_001,
            })
            .unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("exceeds upper bound")));
    }

    #[test]
    fn qs_type_system_rejects_excessive_nesting() {
        let mut contract = default_v4_qs_type_system_contract();
        contract.max_nesting_depth = 2;

        let errors = contract
            .validate_type_ref(&QsTypeRef::Optional {
                inner: Box::new(QsTypeRef::Fresh {
                    inner: Box::new(QsTypeRef::Scalar {
                        scalar: QsScalarTypeKind::Price,
                    }),
                }),
            })
            .unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("exceeds max_nesting_depth")));
    }

    #[test]
    fn static_contract_bundle_accepts_complete_phase_one_bundle() {
        let bundle = sample_static_contract_bundle();

        assert_eq!(bundle.validate_static_contract(), Ok(()));
    }

    #[test]
    fn static_contract_bundle_rejects_child_guard_descriptor_unknown_event_payload_read() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_missing_child_payload_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "missing_payload".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = bundle.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("machine `risk.guard.child`")
                && message.contains("structured guard `bundle_missing_child_payload_guard`")
                && message.contains("unknown event payload field `missing_payload`")
                && message.contains("event `risk.child.check`")
        }));
    }

    #[test]
    fn static_contract_bundle_rejects_child_guard_descriptor_event_party_violations() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["other.risk"],
                &["other.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_child_event_party_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = bundle.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("machine `risk.guard.child`")
                && message.contains("transition `risk.guard.child.transition`")
                && message.contains("is not an allowed consumer")
                && message.contains("event `risk.child.check`")
        }));
        assert!(errors.iter().any(|message| {
            message.contains("machine `risk.guard.child`")
                && message.contains("transition `risk.guard.child.transition`")
                && message.contains("source `risk.guard` is not an allowed emitter")
                && message.contains("event `risk.child.check`")
        }));
    }

    #[test]
    fn static_contract_bundle_rejects_child_guard_descriptor_unknown_transition_event() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.missing".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_child_unknown_event_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::EventPayload,
                path: "symbol".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = bundle.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("machine `risk.guard.child`")
                && message.contains("transition `risk.guard.child.transition`")
                && message.contains("event_type `risk.child.missing`")
                && message.contains("must be declared in event_catalog")
        }));
    }

    #[test]
    fn static_contract_bundle_rejects_child_guard_descriptor_forbidden_parameter_path() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_child_forbidden_parameter_guard".to_string(),
            reads: Vec::new(),
            parameter_paths: vec!["active_strategy.position".to_string()],
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = bundle.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `bundle_child_forbidden_parameter_guard`")
                && message.contains("parameter path `active_strategy.position`")
                && message.contains("outside the proposal-only guard boundary")
        }));
    }

    #[test]
    fn static_contract_bundle_rejects_child_guard_descriptor_invalid_condition_operand() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_child_invalid_condition_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "last_signal_at".to_string(),
            }],
            parameter_paths: vec!["guard.threshold".to_string()],
            conditions: vec![MachineGuardConditionSpec {
                condition_id: "missing_child_parameter".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                comparator: MachineGuardConditionComparator::LessThan,
                right_parameter_path: "guard.missing".to_string(),
            }],
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = bundle.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `bundle_child_invalid_condition_guard`")
                && message.contains("condition `missing_child_parameter`")
                && message.contains("references undeclared parameter path `guard.missing`")
        }));
    }

    #[test]
    fn static_contract_bundle_rejects_child_guard_descriptor_invalid_policy() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_child_invalid_policy_guard".to_string(),
            reads: Vec::new(),
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: Some(MachineGuardPolicySpec {
                timeout_ms: Some(0),
                cooldown_ms: Some(0),
                fallback: None,
            }),
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = bundle.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `bundle_child_invalid_policy_guard`")
                && message.contains("timeout_ms must be greater than zero")
        }));
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `bundle_child_invalid_policy_guard`")
                && message.contains("cooldown_ms must be greater than zero")
        }));
    }

    #[test]
    fn static_contract_bundle_rejects_child_guard_descriptor_duplicate_inputs() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_child_duplicate_input_guard".to_string(),
            reads: vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
            ],
            parameter_paths: vec!["guard.threshold".to_string(), "GUARD.THRESHOLD".to_string()],
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = bundle.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `bundle_child_duplicate_input_guard`")
                && message.contains("declares duplicate machine_memory read `last_signal_at`")
        }));
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `bundle_child_duplicate_input_guard`")
                && message.contains("declares duplicate parameter path `GUARD.THRESHOLD`")
        }));
    }

    #[test]
    fn static_contract_bundle_rejects_child_guard_descriptor_unknown_readonly_runtime_fact() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_child_unknown_runtime_fact_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::ReadonlyRuntimeFact,
                path: "provider.secret".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = bundle.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `bundle_child_unknown_runtime_fact_guard`")
                && message.contains("reads unknown readonly runtime fact `provider.secret`")
        }));
    }

    #[test]
    fn static_contract_bundle_rejects_child_guard_descriptor_unknown_machine_memory_read() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_child_unknown_memory_guard".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "unknown_memory".to_string(),
            }],
            parameter_paths: Vec::new(),
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = bundle.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard `bundle_child_unknown_memory_guard`")
                && message.contains("reads undeclared memory field `unknown_memory`")
        }));
    }

    #[test]
    fn static_contract_bundle_rejects_child_guard_descriptor_base_hygiene_violations() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "".to_string(),
            reads: vec![MachineGuardReadRef {
                source: MachineGuardReadSource::MachineMemory,
                path: "".to_string(),
            }],
            parameter_paths: vec!["".to_string()],
            conditions: Vec::new(),
            policy: None,
            explanation: None,
        });
        risk.states[0].child_machine = Some(Box::new(child));

        let errors = bundle.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("structured guard must declare guard_id")
        }));
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("has an empty read path")
        }));
        assert!(errors.iter().any(|message| {
            message.contains("child_machine `risk.guard.child` failed static contract")
                && message.contains("has an empty parameter path")
        }));
    }

    #[test]
    fn static_contract_bundle_accepts_child_guard_descriptor_full_static_surface() {
        let mut bundle = sample_static_contract_bundle();
        let graph = bundle.machine_graphs.first_mut().unwrap();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .push(sample_event_spec(
                "risk.child.check",
                MachineEventSourceKind::Machine,
                MachineEventScope::Graph,
                &["risk.guard"],
                &["risk.guard.child"],
            ));
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        let mut child = sample_machine_with(
            "risk.guard.child",
            MachineTemplateKind::Decision,
            risk.priority + 1,
        );
        child.transitions[0].event.event_type = "risk.child.check".to_string();
        child.transitions[0].event.source = Some("risk.guard".to_string());
        child.transitions[0].guard = None;
        child.transitions[0].action = None;
        child.transitions[0].guard_descriptor = Some(MachineGuardDescriptor {
            guard_id: "bundle_child_full_static_guard".to_string(),
            reads: vec![
                MachineGuardReadRef {
                    source: MachineGuardReadSource::EventPayload,
                    path: "symbol".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::MachineMemory,
                    path: "last_signal_at".to_string(),
                },
                MachineGuardReadRef {
                    source: MachineGuardReadSource::ReadonlyRuntimeFact,
                    path: "runtime.mode".to_string(),
                },
            ],
            parameter_paths: vec!["guard.threshold".to_string(), "timeout.ms".to_string()],
            conditions: vec![MachineGuardConditionSpec {
                condition_id: "child_runtime_timeout_check".to_string(),
                left_read: MachineGuardReadRef {
                    source: MachineGuardReadSource::ReadonlyRuntimeFact,
                    path: "runtime.mode".to_string(),
                },
                comparator: MachineGuardConditionComparator::Equal,
                right_parameter_path: "timeout.ms".to_string(),
            }],
            policy: Some(MachineGuardPolicySpec {
                timeout_ms: Some(500),
                cooldown_ms: Some(1_000),
                fallback: Some(MachineGuardFallbackPolicy::FailClosed),
            }),
            explanation: Some("child full static guard descriptor surface".to_string()),
        });
        risk.states[0].child_machine = Some(Box::new(child));

        assert_eq!(bundle.validate_static_contract(), Ok(()));
        let projections = bundle.guard_descriptor_projections();
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.graph_id, "strategy.v4.sample");
        assert_eq!(projection.guard.machine_id, "risk.guard.child");
        assert_eq!(projection.guard.guard.event_type, "risk.child.check");
        assert_eq!(
            projection.guard.guard.event_source.as_deref(),
            Some("risk.guard")
        );
        assert_eq!(
            projection.guard.guard.readiness.guard_id,
            "bundle_child_full_static_guard"
        );
        assert_eq!(projection.guard.guard.readiness.read_count, 3);
        assert_eq!(projection.guard.guard.readiness.parameter_path_count, 2);
        assert_eq!(projection.guard.guard.readiness.condition_count, 1);
        assert!(projection.guard.guard.readiness.policy_declared);
        assert!(!projection.guard.guard.readiness.execution_enabled);
        assert_eq!(
            projection.guard.guard.readiness.execution_state,
            MachineGuardExecutionReadinessState::DisabledFailClosed
        );
        assert_eq!(
            projection.guard.guard.readiness.execution_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        assert_eq!(projection.guard.guard.read_projections.len(), 3);
        assert_eq!(
            projection.guard.guard.read_projections[0].binding_scope,
            MachineGuardReadBindingScope::EventPayloadField
        );
        assert_eq!(
            projection.guard.guard.read_projections[1].binding_scope,
            MachineGuardReadBindingScope::MachineMemoryField
        );
        assert_eq!(
            projection.guard.guard.read_projections[2].binding_scope,
            MachineGuardReadBindingScope::ReadonlyRuntimeFact
        );
        let condition = &projection.guard.guard.condition_projections[0];
        assert!(!condition.evaluation_enabled);
        assert_eq!(
            condition.evaluation_blocker_code,
            MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
        );
        let policy = projection.guard.guard.policy_projection.as_ref().unwrap();
        assert!(!policy.timing_execution_enabled);
        assert!(!policy.fallback_execution_enabled);
        assert!(!policy.active_strategy_write_enabled);

        let summary = bundle.guard_descriptor_summary();
        assert_eq!(summary.guard_descriptor_count, 1);
        assert_eq!(summary.guard_id_count, 1);
        assert_eq!(summary.guarded_machine_count, 1);
        assert_eq!(summary.guarded_transition_count, 1);
        assert_eq!(summary.guarded_event_type_count, 1);
        assert_eq!(summary.guarded_event_source_count, 1);
        assert_eq!(summary.event_source_declared_count, 1);
        assert_eq!(summary.event_source_missing_count, 0);
        assert_eq!(summary.decision_guard_descriptor_count, 1);
        assert_eq!(summary.read_guard_descriptor_count, 1);
        assert_eq!(summary.read_count, 3);
        assert_eq!(summary.event_payload_read_count, 1);
        assert_eq!(summary.machine_memory_read_count, 1);
        assert_eq!(summary.readonly_runtime_fact_read_count, 1);
        assert_eq!(summary.parameterized_guard_descriptor_count, 1);
        assert_eq!(summary.parameter_path_count, 2);
        assert_eq!(summary.threshold_parameter_path_count, 1);
        assert_eq!(summary.timeout_parameter_path_count, 1);
        assert_eq!(summary.proposal_only_guard_descriptor_count, 1);
        assert_eq!(summary.conditional_guard_descriptor_count, 1);
        assert_eq!(summary.condition_readonly_runtime_fact_read_count, 1);
        assert_eq!(summary.condition_timeout_parameter_path_count, 1);
        assert_eq!(
            summary.condition_evaluation_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(summary.policy_declared_count, 1);
        assert_eq!(summary.timeout_declared_count, 1);
        assert_eq!(summary.cooldown_declared_count, 1);
        assert_eq!(summary.fallback_fail_closed_declared_count, 1);
        assert_eq!(
            summary.policy_execution_disabled_fail_closed_guard_descriptor_count,
            1
        );
        assert_eq!(
            summary.active_strategy_write_disabled_guard_descriptor_count,
            1
        );
        assert_eq!(summary.execution_enabled_count, 0);
        assert_eq!(summary.execution_disabled_fail_closed_count, 1);
    }

    #[test]
    fn version_manifest_requires_schema_bump_for_semantic_change() {
        let manifest = V4VersionManifest {
            semantic_change_requires_schema_bump: false,
            ..V4VersionManifest::default()
        };

        let errors = manifest.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("semantic changes")));
    }

    #[test]
    fn plugin_governance_rejects_pure_plugin_with_network_permission() {
        let governance = PluginGovernanceContract::default();
        let mut manifest = sample_pure_plugin_manifest();
        manifest.network_permission = PluginNetworkPermission::ProviderOnly;

        let errors = governance
            .validate_plugin_manifest(
                &manifest,
                &default_v4_qs_type_system_contract(),
                &default_v4_runtime_mode_contract(),
            )
            .unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("pure plugins must not require network permission")));
    }

    #[test]
    fn reproducibility_contract_requires_risk_decision_evidence() {
        let mut contract = ReproducibilityContract::default();
        contract
            .required_evidence
            .retain(|kind| *kind != RunEvidenceKind::RiskDecisionEvidence);

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("RiskDecisionEvidence")));
    }

    #[test]
    fn complexity_budget_rejects_over_budget_graph() {
        let budget = ComplexityBudgetContract {
            max_state_count: 1,
            ..ComplexityBudgetContract::default()
        };
        let metrics = ComplexityMetrics::from_machine_graph(&sample_machine_graph(), 4, 0);

        let errors = budget.validate_metrics(&metrics).unwrap_err();
        assert!(errors.iter().any(|message| message.contains("state_count")));
    }

    #[test]
    fn learning_pipeline_contract_keeps_local_records_out_of_git() {
        let contract = DeveloperLearningPipelineContract {
            local_learning_dir_gitignored: false,
            ..DeveloperLearningPipelineContract::default()
        };

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| message.contains("gitignored")));
    }

    #[test]
    fn compile_time_capability_report_accepts_supported_phase_two_request() {
        let bundle = V4StaticContractBundle {
            machine_graphs: vec![sample_machine_graph()],
            venue_matrices: vec![sample_paper_simulated_market_matrix()],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };
        let request = sample_compile_time_capability_request();

        let report = bundle.build_compile_time_capability_report(request);

        assert_eq!(report.verdict, V4CapabilityReportVerdict::Accepted);
        assert_eq!(report.validate_for_compile(), Ok(()));
        assert!(!report.execution_submission_attached);
        assert_eq!(
            report
                .execution_entries
                .iter()
                .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
                .unwrap()
                .selected_source,
            Some(CapabilitySupportSource::RuntimeSimulated)
        );
        assert_eq!(
            report.plugin_entries[0].status,
            V4PluginCapabilityStatus::Accepted
        );
    }

    #[test]
    fn compile_time_capability_report_rejects_unsupported_required_capability() {
        let bundle = V4StaticContractBundle {
            machine_graphs: vec![sample_machine_graph()],
            venue_matrices: vec![unsupported_v4_first_wave_matrix("paper-local")],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };
        let request = sample_compile_time_capability_request();

        let report = bundle.build_compile_time_capability_report(request);

        assert_eq!(report.verdict, V4CapabilityReportVerdict::Rejected);
        assert!(report.validate_for_compile().is_err());
        assert_eq!(
            report
                .execution_entries
                .iter()
                .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
                .unwrap()
                .status,
            V4ExecutionCapabilityStatus::Unsupported
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "V4CAP202"));
    }

    #[test]
    fn compile_time_capability_report_rejects_provider_native_for_local_simulated_mode() {
        let mut matrix = unsupported_v4_first_wave_matrix("paper-local");
        let market = matrix
            .capabilities
            .iter_mut()
            .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
            .unwrap();
        market.source = CapabilitySupportSource::ProviderNative;
        market.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
        let bundle = V4StaticContractBundle {
            machine_graphs: vec![sample_machine_graph()],
            venue_matrices: vec![matrix],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };

        let report =
            bundle.build_compile_time_capability_report(sample_compile_time_capability_request());

        assert_eq!(report.verdict, V4CapabilityReportVerdict::Rejected);
        assert_eq!(
            report
                .execution_entries
                .iter()
                .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
                .unwrap()
                .status,
            V4ExecutionCapabilityStatus::ModeRejected
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("requires runtime_simulated")));
    }

    #[test]
    fn compile_time_capability_report_rejects_invalid_required_type_ref() {
        let bundle = V4StaticContractBundle {
            machine_graphs: vec![sample_machine_graph()],
            venue_matrices: vec![sample_paper_simulated_market_matrix()],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };
        let mut request = sample_compile_time_capability_request();
        request.required_type_refs = vec![QsTypeRef::List {
            item: Box::new(QsTypeRef::Scalar {
                scalar: QsScalarTypeKind::Price,
            }),
            max_items: 0,
        }];

        let report = bundle.build_compile_time_capability_report(request);

        assert_eq!(report.verdict, V4CapabilityReportVerdict::Rejected);
        assert_eq!(
            report.type_entries[0].status,
            V4TypeCapabilityStatus::Rejected
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "V4CAP100"));
    }

    #[test]
    fn compile_time_capability_report_rejects_missing_required_plugin() {
        let bundle = V4StaticContractBundle {
            machine_graphs: vec![sample_machine_graph()],
            venue_matrices: vec![sample_paper_simulated_market_matrix()],
            ..V4StaticContractBundle::default()
        };

        let report =
            bundle.build_compile_time_capability_report(sample_compile_time_capability_request());

        assert_eq!(report.verdict, V4CapabilityReportVerdict::Rejected);
        assert_eq!(
            report.plugin_entries[0].status,
            V4PluginCapabilityStatus::Missing
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "V4CAP301"));
    }

    #[test]
    fn venue_matrix_requires_provider_native_for_provider_actual_mode() {
        let contract = default_v4_runtime_mode_contract();
        let matrix = VenueCapabilityMatrix {
            schema_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
            venue_id: "paper-local".to_string(),
            capabilities: vec![VenueCapability {
                capability: ExecutionCapabilityKind::Market,
                source: CapabilitySupportSource::RuntimeSimulated,
                supported_modes: vec![RuntimeTradingMode::LiveActual],
                constraints: BTreeMap::new(),
            }],
            metadata: BTreeMap::new(),
        };

        let error = matrix
            .require_supported_for_mode(
                &ExecutionCapabilityKind::Market,
                RuntimeTradingMode::LiveActual,
                &contract,
            )
            .unwrap_err();
        assert!(error.contains("requires provider_native"));
    }

    #[test]
    fn venue_matrix_requires_runtime_simulated_for_local_simulated_mode() {
        let contract = default_v4_runtime_mode_contract();
        let matrix = VenueCapabilityMatrix {
            schema_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
            venue_id: "paper-local".to_string(),
            capabilities: vec![VenueCapability {
                capability: ExecutionCapabilityKind::Market,
                source: CapabilitySupportSource::RuntimeSimulated,
                supported_modes: vec![RuntimeTradingMode::PaperSimulated],
                constraints: BTreeMap::new(),
            }],
            metadata: BTreeMap::new(),
        };

        assert_eq!(
            matrix.require_supported_for_mode(
                &ExecutionCapabilityKind::Market,
                RuntimeTradingMode::PaperSimulated,
                &contract,
            ),
            Ok(CapabilitySupportSource::RuntimeSimulated)
        );
    }

    #[test]
    fn venue_matrix_rejects_duplicate_capabilities() {
        let matrix = VenueCapabilityMatrix {
            schema_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
            venue_id: "okx".to_string(),
            capabilities: vec![
                VenueCapability {
                    capability: ExecutionCapabilityKind::Market,
                    source: CapabilitySupportSource::ProviderNative,
                    supported_modes: vec![RuntimeTradingMode::PaperActual],
                    constraints: BTreeMap::new(),
                },
                VenueCapability {
                    capability: ExecutionCapabilityKind::Market,
                    source: CapabilitySupportSource::RuntimeSimulated,
                    supported_modes: vec![RuntimeTradingMode::PaperSimulated],
                    constraints: BTreeMap::new(),
                },
            ],
            metadata: BTreeMap::new(),
        };

        let errors = matrix.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("duplicate execution capability")));
    }

    #[test]
    fn venue_matrix_does_not_silently_support_missing_capability() {
        let matrix = VenueCapabilityMatrix {
            schema_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
            venue_id: "paper-local".to_string(),
            capabilities: vec![VenueCapability {
                capability: ExecutionCapabilityKind::Market,
                source: CapabilitySupportSource::RuntimeSimulated,
                supported_modes: vec![RuntimeTradingMode::PaperSimulated],
                constraints: BTreeMap::new(),
            }],
            metadata: BTreeMap::new(),
        };

        assert_eq!(
            matrix.require_supported(&ExecutionCapabilityKind::Market),
            Ok(CapabilitySupportSource::RuntimeSimulated)
        );
        assert!(matrix
            .require_supported(&ExecutionCapabilityKind::TrailingStop)
            .is_err());
    }

    #[test]
    fn venue_matrix_requires_explicit_first_wave_capability_sources() {
        let matrix = VenueCapabilityMatrix {
            schema_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
            venue_id: "paper-local".to_string(),
            capabilities: vec![VenueCapability {
                capability: ExecutionCapabilityKind::Market,
                source: CapabilitySupportSource::RuntimeSimulated,
                supported_modes: vec![RuntimeTradingMode::PaperSimulated],
                constraints: BTreeMap::new(),
            }],
            metadata: BTreeMap::new(),
        };

        assert_eq!(matrix.validate_static_contract(), Ok(()));

        let errors = matrix.validate_v4_first_wave_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("required execution capability")));
    }

    #[test]
    fn unsupported_first_wave_matrix_declares_every_source_without_supporting_them() {
        let matrix = unsupported_v4_first_wave_matrix("unknown-venue");

        assert_eq!(matrix.validate_v4_first_wave_contract(), Ok(()));
        assert_eq!(
            matrix.support_source(&ExecutionCapabilityKind::Market),
            CapabilitySupportSource::Unsupported
        );
        assert!(matrix
            .require_supported(&ExecutionCapabilityKind::Market)
            .is_err());
        assert_eq!(
            matrix.capabilities.len(),
            v4_first_wave_execution_capabilities().len()
        );
    }
}
