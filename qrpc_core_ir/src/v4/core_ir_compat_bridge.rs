mod core_ir_validation;

use core_ir_validation::validate_core_ir_for_v4_bridge;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{
    EventFreshnessRequirement, MachineActionSpec, MachineCachePolicy, MachineEventCatalog,
    MachineEventPayloadField, MachineEventScope, MachineEventSelector, MachineEventSourceKind,
    MachineEventTypeSpec, MachineGraphEdge, MachineGraphEdgeActivation, MachineGraphRiskPlane,
    MachineMemoryField, MachineRecoveryPolicy, MachineSilencePolicy, MachineState,
    MachineTemplateKind, MachineTransition, QsScalarTypeKind, QsTypeRef, StateGroup,
    TransitionConflictPolicy, V4MachineContract, V4MachineGraphContract,
    V4_COMPAT_CORE_IR_LOADED_EVENT, V4_COMPAT_DECISION_MACHINE_ID, V4_COMPAT_EXECUTION_MACHINE_ID,
    V4_COMPAT_OBSERVATION_MACHINE_ID, V4_COMPAT_OBSERVATION_READY_EVENT,
    V4_COMPAT_RISK_APPROVED_EVENT, V4_CORE_IR_COMPAT_BRIDGE_VERSION, V4_MACHINE_CONTRACT_VERSION,
    V4_MACHINE_EVENT_CATALOG_VERSION, V4_MACHINE_GRAPH_CONTRACT_VERSION,
    V4_RISK_PLANE_MIN_PRIORITY,
};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoreIrV4CompatibilityReport {
    #[serde(default = "default_core_ir_compat_bridge_version")]
    pub schema_version: String,
    pub verdict: CoreIrV4BridgeVerdict,
    pub core_ir_version: String,
    pub strategy_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph: Option<V4MachineGraphContract>,
    #[serde(default)]
    pub diagnostics: Vec<CoreIrV4BridgeDiagnostic>,
    #[serde(default)]
    pub lowering_attached: bool,
    #[serde(default)]
    pub runtime_attached: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoreIrV4BridgeVerdict {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoreIrV4BridgeDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoreIrV4BridgeDiagnostic {
    pub severity: CoreIrV4BridgeDiagnosticSeverity,
    pub code: String,
    pub target: String,
    pub message: String,
}

impl CoreIrV4CompatibilityReport {
    pub fn validate_for_phase4(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_CORE_IR_COMPAT_BRIDGE_VERSION {
            errors.push(format!(
                "core ir compatibility report schema_version must be `{}`",
                V4_CORE_IR_COMPAT_BRIDGE_VERSION
            ));
        }
        if self.verdict != CoreIrV4BridgeVerdict::Accepted {
            errors.push("core ir compatibility report verdict must be accepted".to_string());
        }
        if self.lowering_attached {
            errors.push(
                "core ir compatibility bridge must not attach v4 lowering in Phase 4".to_string(),
            );
        }
        if self.runtime_attached {
            errors.push(
                "core ir compatibility bridge must not attach runtime in Phase 4".to_string(),
            );
        }
        for diagnostic in &self.diagnostics {
            if diagnostic.severity == CoreIrV4BridgeDiagnosticSeverity::Error {
                errors.push(format!(
                    "{} {}: {}",
                    diagnostic.code, diagnostic.target, diagnostic.message
                ));
            }
        }
        match &self.graph {
            Some(graph) => {
                errors.extend(graph.validate_static_contract().err().unwrap_or_default())
            }
            None => errors.push(
                "core ir compatibility bridge must produce a machine graph when accepted"
                    .to_string(),
            ),
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn bridge_core_ir_to_v4_machine_graph(
    core_ir: &crate::CoreStrategyIr,
) -> CoreIrV4CompatibilityReport {
    let mut diagnostics = Vec::new();

    validate_core_ir_for_v4_bridge(core_ir, &mut diagnostics);

    let mut graph = if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == CoreIrV4BridgeDiagnosticSeverity::Error)
    {
        None
    } else {
        Some(build_core_ir_compat_machine_graph(core_ir))
    };

    if let Some(candidate_graph) = &graph {
        if let Err(errors) = candidate_graph.validate_static_contract() {
            for error in errors {
                push_core_ir_v4_bridge_diagnostic(
                    &mut diagnostics,
                    CoreIrV4BridgeDiagnosticSeverity::Error,
                    "V4BRIDGE900",
                    "machine_graph",
                    error,
                );
            }
        }
    }

    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == CoreIrV4BridgeDiagnosticSeverity::Error)
    {
        graph = None;
    }

    let verdict = if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == CoreIrV4BridgeDiagnosticSeverity::Error)
    {
        CoreIrV4BridgeVerdict::Rejected
    } else {
        CoreIrV4BridgeVerdict::Accepted
    };

    CoreIrV4CompatibilityReport {
        schema_version: V4_CORE_IR_COMPAT_BRIDGE_VERSION.to_string(),
        verdict,
        core_ir_version: core_ir.ir_version.clone(),
        strategy_id: core_ir.metadata.strategy_id.clone(),
        graph,
        diagnostics,
        lowering_attached: false,
        runtime_attached: false,
    }
}

fn build_core_ir_compat_machine_graph(core_ir: &crate::CoreStrategyIr) -> V4MachineGraphContract {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "compat_bridge_version".to_string(),
        Value::String(V4_CORE_IR_COMPAT_BRIDGE_VERSION.to_string()),
    );
    metadata.insert(
        "core_ir_version".to_string(),
        Value::String(core_ir.ir_version.clone()),
    );
    metadata.insert(
        "core_strategy_id".to_string(),
        Value::String(core_ir.metadata.strategy_id.clone()),
    );
    metadata.insert(
        "core_strategy_name".to_string(),
        Value::String(core_ir.metadata.name.clone()),
    );
    metadata.insert(
        "core_source_kind".to_string(),
        Value::String(format!("{:?}", core_ir.metadata.source_kind)),
    );
    metadata.insert(
        "compat_semantics".to_string(),
        Value::String("static_only_default_machine_instances_without_runtime_lowering".to_string()),
    );
    metadata.insert(
        "core_edge_count".to_string(),
        Value::from(core_ir.edges.len() as u64),
    );
    metadata.insert(
        "core_edge_labels".to_string(),
        core_ir_edge_labels(&core_ir.edges),
    );

    V4MachineGraphContract {
        schema_version: V4_MACHINE_GRAPH_CONTRACT_VERSION.to_string(),
        graph_id: format!(
            "compat.{}",
            sanitize_core_ir_compat_id(&core_ir.metadata.strategy_id, "strategy")
        ),
        machines: vec![
            build_core_ir_observation_machine(core_ir),
            build_core_ir_decision_machine(core_ir),
            build_core_ir_execution_machine(core_ir),
        ],
        edges: vec![
            MachineGraphEdge {
                edge_id: "compat.observation_to_decision".to_string(),
                source_machine_id: V4_COMPAT_OBSERVATION_MACHINE_ID.to_string(),
                target_machine_id: V4_COMPAT_DECISION_MACHINE_ID.to_string(),
                event_type: V4_COMPAT_OBSERVATION_READY_EVENT.to_string(),
                activation: MachineGraphEdgeActivation::Always,
                required: true,
                metadata: compat_edge_metadata("data_indicator_to_signal_agent_risk"),
            },
            MachineGraphEdge {
                edge_id: "compat.decision_to_execution".to_string(),
                source_machine_id: V4_COMPAT_DECISION_MACHINE_ID.to_string(),
                target_machine_id: V4_COMPAT_EXECUTION_MACHINE_ID.to_string(),
                event_type: V4_COMPAT_RISK_APPROVED_EVENT.to_string(),
                activation: MachineGraphEdgeActivation::Always,
                required: true,
                metadata: compat_edge_metadata("risk_plane_to_execution"),
            },
        ],
        event_catalog: Some(build_core_ir_compat_event_catalog(core_ir)),
        risk_plane: Some(MachineGraphRiskPlane {
            required: true,
            machine_ids: vec![V4_COMPAT_DECISION_MACHINE_ID.to_string()],
            min_priority: V4_RISK_PLANE_MIN_PRIORITY,
        }),
        metadata,
    }
}

fn build_core_ir_observation_machine(core_ir: &crate::CoreStrategyIr) -> V4MachineContract {
    let mut metadata = compat_machine_metadata("data_and_indicator");
    metadata.insert(
        "core_data_binding_ids".to_string(),
        string_value_array(
            core_ir
                .data_bindings
                .iter()
                .map(|binding| binding.data_id.clone()),
        ),
    );
    metadata.insert(
        "core_indicator_ids".to_string(),
        string_value_array(
            core_ir
                .indicators
                .iter()
                .map(|indicator| indicator.indicator_id.clone()),
        ),
    );

    compat_machine(
        V4_COMPAT_OBSERVATION_MACHINE_ID,
        MachineTemplateKind::Observation,
        8_000,
        V4_COMPAT_CORE_IR_LOADED_EVENT,
        None,
        vec![V4_COMPAT_OBSERVATION_READY_EVENT.to_string()],
        vec![
            count_memory_field("data_binding_count", core_ir.data_bindings.len()),
            count_memory_field("indicator_count", core_ir.indicators.len()),
        ],
        vec!["observe_data_and_update_indicators".to_string()],
        metadata,
    )
}

fn build_core_ir_decision_machine(core_ir: &crate::CoreStrategyIr) -> V4MachineContract {
    let mut metadata = compat_machine_metadata("signal_agent_risk_plane");
    metadata.insert(
        "core_signal_ids".to_string(),
        string_value_array(
            core_ir
                .signal_rules
                .iter()
                .map(|signal| signal.signal_id.clone()),
        ),
    );
    metadata.insert(
        "core_agent_ids".to_string(),
        string_value_array(
            core_ir
                .agent_policies
                .iter()
                .map(|agent| agent.agent_id.clone()),
        ),
    );
    metadata.insert(
        "core_risk_policy_ids".to_string(),
        string_value_array(
            core_ir
                .risk_policies
                .iter()
                .map(|risk| risk.policy_id.clone()),
        ),
    );

    compat_machine(
        V4_COMPAT_DECISION_MACHINE_ID,
        MachineTemplateKind::Decision,
        9_500,
        V4_COMPAT_OBSERVATION_READY_EVENT,
        Some(V4_COMPAT_OBSERVATION_MACHINE_ID),
        vec![V4_COMPAT_RISK_APPROVED_EVENT.to_string()],
        vec![
            count_memory_field("signal_rule_count", core_ir.signal_rules.len()),
            count_memory_field("agent_policy_count", core_ir.agent_policies.len()),
            count_memory_field("risk_policy_count", core_ir.risk_policies.len()),
        ],
        vec!["evaluate_intent_agent_and_risk_plane".to_string()],
        metadata,
    )
}

fn build_core_ir_execution_machine(core_ir: &crate::CoreStrategyIr) -> V4MachineContract {
    let mut metadata = compat_machine_metadata("execution");
    metadata.insert(
        "core_execution_id".to_string(),
        Value::String(core_ir.execution.execution_id.clone()),
    );
    metadata.insert(
        "core_venue_kind".to_string(),
        Value::String(core_ir.execution.venue_kind.clone()),
    );
    metadata.insert(
        "core_sizing_kind".to_string(),
        Value::String(format!("{:?}", core_ir.execution.sizing_kind)),
    );
    metadata.insert(
        "core_time_in_force".to_string(),
        Value::String(format!("{:?}", core_ir.execution.time_in_force)),
    );

    compat_machine(
        V4_COMPAT_EXECUTION_MACHINE_ID,
        MachineTemplateKind::Execution,
        4_000,
        V4_COMPAT_RISK_APPROVED_EVENT,
        Some(V4_COMPAT_DECISION_MACHINE_ID),
        Vec::new(),
        vec![MachineMemoryField {
            name: "execution_config_present".to_string(),
            type_name: "bool".to_string(),
            type_ref: Some(QsTypeRef::Scalar {
                scalar: QsScalarTypeKind::Bool,
            }),
            default_value: Some(Value::Bool(true)),
            nullable: false,
        }],
        vec!["route_legacy_execution_rule".to_string()],
        metadata,
    )
}

fn compat_machine(
    machine_id: &str,
    template: MachineTemplateKind,
    priority: i32,
    input_event: &str,
    input_source: Option<&str>,
    emitted_events: Vec<String>,
    memory: Vec<MachineMemoryField>,
    diagnostics: Vec<String>,
    metadata: BTreeMap<String, Value>,
) -> V4MachineContract {
    let memory_writes = memory.iter().map(|field| field.name.clone()).collect();

    V4MachineContract {
        schema_version: V4_MACHINE_CONTRACT_VERSION.to_string(),
        machine_id: machine_id.to_string(),
        template,
        states: vec![
            MachineState {
                state_id: "idle".to_string(),
                group_id: Some("compat_flow".to_string()),
                initial: true,
                terminal: false,
                child_machine: None,
            },
            MachineState {
                state_id: "ready".to_string(),
                group_id: Some("compat_flow".to_string()),
                initial: false,
                terminal: false,
                child_machine: None,
            },
        ],
        state_groups: vec![StateGroup {
            group_id: "compat_flow".to_string(),
            state_ids: vec!["idle".to_string(), "ready".to_string()],
            conflict_policy: TransitionConflictPolicy::Error,
            timeout_ms: None,
        }],
        transitions: vec![MachineTransition {
            transition_id: format!("{machine_id}.idle_to_ready"),
            from_state: "idle".to_string(),
            to_state: "ready".to_string(),
            event: MachineEventSelector {
                event_type: input_event.to_string(),
                source: input_source.map(str::to_string),
                freshness: Some(EventFreshnessRequirement::FreshOrStale),
            },
            guard: None,
            priority,
            action: Some(MachineActionSpec {
                emits: emitted_events,
                memory_writes,
                diagnostics,
            }),
        }],
        memory,
        cache_policy: MachineCachePolicy::ReturnLastThenRecover,
        silence_policy: MachineSilencePolicy::SoftDormantAfter { ttl_ms: 60_000 },
        recovery_policy: MachineRecoveryPolicy::AsyncRecover,
        priority,
        metadata,
    }
}

fn build_core_ir_compat_event_catalog(core_ir: &crate::CoreStrategyIr) -> MachineEventCatalog {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "compat_bridge_version".to_string(),
        Value::String(V4_CORE_IR_COMPAT_BRIDGE_VERSION.to_string()),
    );
    metadata.insert(
        "core_strategy_id".to_string(),
        Value::String(core_ir.metadata.strategy_id.clone()),
    );

    MachineEventCatalog {
        schema_version: V4_MACHINE_EVENT_CATALOG_VERSION.to_string(),
        events: vec![
            MachineEventTypeSpec {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source_kind: MachineEventSourceKind::Runtime,
                scope: MachineEventScope::Runtime,
                payload_fields: vec![MachineEventPayloadField {
                    name: "strategy_id".to_string(),
                    type_name: "string".to_string(),
                    required: true,
                    nullable: false,
                }],
                allowed_emitters: Vec::new(),
                allowed_consumers: vec![V4_COMPAT_OBSERVATION_MACHINE_ID.to_string()],
                replayable: true,
            },
            MachineEventTypeSpec {
                event_type: V4_COMPAT_OBSERVATION_READY_EVENT.to_string(),
                source_kind: MachineEventSourceKind::Machine,
                scope: MachineEventScope::Graph,
                payload_fields: vec![MachineEventPayloadField {
                    name: "data_binding_count".to_string(),
                    type_name: "u64".to_string(),
                    required: true,
                    nullable: false,
                }],
                allowed_emitters: vec![V4_COMPAT_OBSERVATION_MACHINE_ID.to_string()],
                allowed_consumers: vec![V4_COMPAT_DECISION_MACHINE_ID.to_string()],
                replayable: true,
            },
            MachineEventTypeSpec {
                event_type: V4_COMPAT_RISK_APPROVED_EVENT.to_string(),
                source_kind: MachineEventSourceKind::RiskPlane,
                scope: MachineEventScope::Graph,
                payload_fields: vec![MachineEventPayloadField {
                    name: "execution_id".to_string(),
                    type_name: "string".to_string(),
                    required: true,
                    nullable: false,
                }],
                allowed_emitters: vec![V4_COMPAT_DECISION_MACHINE_ID.to_string()],
                allowed_consumers: vec![V4_COMPAT_EXECUTION_MACHINE_ID.to_string()],
                replayable: true,
            },
        ],
        metadata,
    }
}

fn compat_machine_metadata(core_role: &str) -> BTreeMap<String, Value> {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "core_role".to_string(),
        Value::String(core_role.to_string()),
    );
    metadata.insert(
        "compat_bridge_version".to_string(),
        Value::String(V4_CORE_IR_COMPAT_BRIDGE_VERSION.to_string()),
    );
    metadata
}

fn compat_edge_metadata(core_role: &str) -> BTreeMap<String, Value> {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "core_role".to_string(),
        Value::String(core_role.to_string()),
    );
    metadata
}

fn count_memory_field(name: &str, count: usize) -> MachineMemoryField {
    MachineMemoryField {
        name: name.to_string(),
        type_name: "u64".to_string(),
        type_ref: None,
        default_value: Some(Value::from(count as u64)),
        nullable: false,
    }
}

fn core_ir_edge_labels(edges: &[crate::CoreIREdge]) -> Value {
    string_value_array(edges.iter().map(|edge| match &edge.port {
        Some(port) => format!("{} -> {}@{}", edge.source, edge.target, port),
        None => format!("{} -> {}", edge.source, edge.target),
    }))
}

fn string_value_array(values: impl IntoIterator<Item = String>) -> Value {
    Value::Array(values.into_iter().map(Value::String).collect())
}

fn sanitize_core_ir_compat_id(raw: &str, fallback: &str) -> String {
    let sanitized = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();

    if sanitized.is_empty() {
        fallback.to_string()
    } else {
        sanitized
    }
}

fn default_core_ir_compat_bridge_version() -> String {
    V4_CORE_IR_COMPAT_BRIDGE_VERSION.to_string()
}

fn push_core_ir_v4_bridge_diagnostic(
    diagnostics: &mut Vec<CoreIrV4BridgeDiagnostic>,
    severity: CoreIrV4BridgeDiagnosticSeverity,
    code: impl Into<String>,
    target: impl Into<String>,
    message: impl Into<String>,
) {
    diagnostics.push(CoreIrV4BridgeDiagnostic {
        severity,
        code: code.into(),
        target: target.into(),
        message: message.into(),
    });
}
