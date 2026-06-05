mod event_catalog_builder;
mod machine_builder;

use event_catalog_builder::build_core_ir_compat_event_catalog;
use machine_builder::{
    build_core_ir_decision_machine, build_core_ir_execution_machine,
    build_core_ir_observation_machine,
};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::v4::{
    MachineGraphEdge, MachineGraphEdgeActivation, MachineGraphRiskPlane, V4MachineGraphContract,
    V4_COMPAT_DECISION_MACHINE_ID, V4_COMPAT_EXECUTION_MACHINE_ID,
    V4_COMPAT_OBSERVATION_MACHINE_ID, V4_COMPAT_OBSERVATION_READY_EVENT,
    V4_COMPAT_RISK_APPROVED_EVENT, V4_CORE_IR_COMPAT_BRIDGE_VERSION,
    V4_MACHINE_GRAPH_CONTRACT_VERSION, V4_RISK_PLANE_MIN_PRIORITY,
};
pub(super) fn build_core_ir_compat_machine_graph(
    core_ir: &crate::CoreStrategyIr,
) -> V4MachineGraphContract {
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

fn compat_edge_metadata(core_role: &str) -> BTreeMap<String, Value> {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "core_role".to_string(),
        Value::String(core_role.to_string()),
    );
    metadata
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
