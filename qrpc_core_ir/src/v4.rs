mod backtest_artifact_contract;
mod compile_time_capability_report;
mod complexity_budget_contract;
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
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

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

fn validate_core_ir_for_v4_bridge(
    core_ir: &crate::CoreStrategyIr,
    diagnostics: &mut Vec<CoreIrV4BridgeDiagnostic>,
) {
    if core_ir.ir_version != crate::CORE_IR_V1_VERSION {
        push_core_ir_v4_bridge_diagnostic(
            diagnostics,
            CoreIrV4BridgeDiagnosticSeverity::Error,
            "V4BRIDGE000",
            "core_ir.ir_version",
            format!(
                "Core IR compatibility bridge only accepts `{}`",
                crate::CORE_IR_V1_VERSION
            ),
        );
    }
    if core_ir.metadata.strategy_id.trim().is_empty() {
        push_core_ir_v4_bridge_diagnostic(
            diagnostics,
            CoreIrV4BridgeDiagnosticSeverity::Error,
            "V4BRIDGE001",
            "core_ir.metadata.strategy_id",
            "strategy_id is required",
        );
    }
    if core_ir.data_bindings.is_empty() {
        push_core_ir_v4_bridge_diagnostic(
            diagnostics,
            CoreIrV4BridgeDiagnosticSeverity::Error,
            "V4BRIDGE002",
            "core_ir.data_bindings",
            "at least one data binding is required for ObservationMachine compatibility",
        );
    }
    if core_ir.risk_policies.is_empty() {
        push_core_ir_v4_bridge_diagnostic(
            diagnostics,
            CoreIrV4BridgeDiagnosticSeverity::Error,
            "V4BRIDGE003",
            "core_ir.risk_policies",
            "at least one risk policy is required for DecisionMachine Risk Plane compatibility",
        );
    }
    if core_ir.execution.execution_id.trim().is_empty() {
        push_core_ir_v4_bridge_diagnostic(
            diagnostics,
            CoreIrV4BridgeDiagnosticSeverity::Error,
            "V4BRIDGE004",
            "core_ir.execution.execution_id",
            "execution_id is required for ExecutionMachine compatibility",
        );
    }
    if core_ir.execution.venue_kind.trim().is_empty() {
        push_core_ir_v4_bridge_diagnostic(
            diagnostics,
            CoreIrV4BridgeDiagnosticSeverity::Error,
            "V4BRIDGE005",
            "core_ir.execution.venue_kind",
            "venue_kind is required for ExecutionMachine compatibility",
        );
    }

    let known_node_ids = collect_core_ir_node_ids(core_ir, diagnostics);
    validate_core_ir_references_for_v4_bridge(core_ir, diagnostics);
    validate_core_ir_edges_for_v4_bridge(core_ir, &known_node_ids, diagnostics);

    if let Err(errors) = core_ir.validate_dag() {
        for error in errors {
            push_core_ir_v4_bridge_diagnostic(
                diagnostics,
                CoreIrV4BridgeDiagnosticSeverity::Error,
                "V4BRIDGE020",
                "core_ir.edges",
                error,
            );
        }
    }
}

fn collect_core_ir_node_ids(
    core_ir: &crate::CoreStrategyIr,
    diagnostics: &mut Vec<CoreIrV4BridgeDiagnostic>,
) -> BTreeSet<String> {
    let mut node_ids = BTreeSet::new();

    for (index, binding) in core_ir.data_bindings.iter().enumerate() {
        insert_core_ir_node_id(
            &mut node_ids,
            diagnostics,
            format!("core_ir.data_bindings[{index}].data_id"),
            &binding.data_id,
        );
    }
    for (index, indicator) in core_ir.indicators.iter().enumerate() {
        insert_core_ir_node_id(
            &mut node_ids,
            diagnostics,
            format!("core_ir.indicators[{index}].indicator_id"),
            &indicator.indicator_id,
        );
    }
    for (index, signal) in core_ir.signal_rules.iter().enumerate() {
        insert_core_ir_node_id(
            &mut node_ids,
            diagnostics,
            format!("core_ir.signal_rules[{index}].signal_id"),
            &signal.signal_id,
        );
    }
    for (index, agent) in core_ir.agent_policies.iter().enumerate() {
        insert_core_ir_node_id(
            &mut node_ids,
            diagnostics,
            format!("core_ir.agent_policies[{index}].agent_id"),
            &agent.agent_id,
        );
    }
    for (index, risk) in core_ir.risk_policies.iter().enumerate() {
        insert_core_ir_node_id(
            &mut node_ids,
            diagnostics,
            format!("core_ir.risk_policies[{index}].policy_id"),
            &risk.policy_id,
        );
    }
    insert_core_ir_node_id(
        &mut node_ids,
        diagnostics,
        "core_ir.execution.execution_id",
        &core_ir.execution.execution_id,
    );

    node_ids
}

fn insert_core_ir_node_id(
    node_ids: &mut BTreeSet<String>,
    diagnostics: &mut Vec<CoreIrV4BridgeDiagnostic>,
    target: impl Into<String>,
    node_id: &str,
) {
    let target = target.into();
    if node_id.trim().is_empty() {
        push_core_ir_v4_bridge_diagnostic(
            diagnostics,
            CoreIrV4BridgeDiagnosticSeverity::Error,
            "V4BRIDGE010",
            target,
            "Core IR node id must not be empty",
        );
        return;
    }
    if !node_ids.insert(node_id.to_string()) {
        push_core_ir_v4_bridge_diagnostic(
            diagnostics,
            CoreIrV4BridgeDiagnosticSeverity::Error,
            "V4BRIDGE011",
            target,
            format!("duplicate Core IR node id `{node_id}`"),
        );
    }
}

fn validate_core_ir_edges_for_v4_bridge(
    core_ir: &crate::CoreStrategyIr,
    known_node_ids: &BTreeSet<String>,
    diagnostics: &mut Vec<CoreIrV4BridgeDiagnostic>,
) {
    for (index, edge) in core_ir.edges.iter().enumerate() {
        if edge.source.trim().is_empty() {
            push_core_ir_v4_bridge_diagnostic(
                diagnostics,
                CoreIrV4BridgeDiagnosticSeverity::Error,
                "V4BRIDGE030",
                format!("core_ir.edges[{index}].source"),
                "edge source is required",
            );
        } else if !known_node_ids.contains(edge.source.as_str()) {
            push_core_ir_v4_bridge_diagnostic(
                diagnostics,
                CoreIrV4BridgeDiagnosticSeverity::Error,
                "V4BRIDGE031",
                format!("core_ir.edges[{index}].source"),
                format!(
                    "edge source `{}` is not a declared Core IR node",
                    edge.source
                ),
            );
        }

        if edge.target.trim().is_empty() {
            push_core_ir_v4_bridge_diagnostic(
                diagnostics,
                CoreIrV4BridgeDiagnosticSeverity::Error,
                "V4BRIDGE032",
                format!("core_ir.edges[{index}].target"),
                "edge target is required",
            );
        } else if !known_node_ids.contains(edge.target.as_str()) {
            push_core_ir_v4_bridge_diagnostic(
                diagnostics,
                CoreIrV4BridgeDiagnosticSeverity::Error,
                "V4BRIDGE033",
                format!("core_ir.edges[{index}].target"),
                format!(
                    "edge target `{}` is not a declared Core IR node",
                    edge.target
                ),
            );
        }
    }
}

fn validate_core_ir_references_for_v4_bridge(
    core_ir: &crate::CoreStrategyIr,
    diagnostics: &mut Vec<CoreIrV4BridgeDiagnostic>,
) {
    let data_ids = core_ir
        .data_bindings
        .iter()
        .map(|binding| binding.data_id.as_str())
        .collect::<BTreeSet<_>>();
    let indicator_ids = core_ir
        .indicators
        .iter()
        .map(|indicator| indicator.indicator_id.as_str())
        .collect::<BTreeSet<_>>();
    let signal_ids = core_ir
        .signal_rules
        .iter()
        .map(|signal| signal.signal_id.as_str())
        .collect::<BTreeSet<_>>();
    let agent_ids = core_ir
        .agent_policies
        .iter()
        .map(|agent| agent.agent_id.as_str())
        .collect::<BTreeSet<_>>();

    for (index, indicator) in core_ir.indicators.iter().enumerate() {
        for (input_index, input) in indicator.inputs.iter().enumerate() {
            validate_core_ir_series_expr_refs(
                input,
                &data_ids,
                &indicator_ids,
                diagnostics,
                format!("core_ir.indicators[{index}].inputs[{input_index}]"),
            );
        }
        if let Some(spread) = &indicator.spread_spec {
            validate_core_ir_series_expr_refs(
                &spread.left,
                &data_ids,
                &indicator_ids,
                diagnostics,
                format!("core_ir.indicators[{index}].spread_spec.left"),
            );
            validate_core_ir_series_expr_refs(
                &spread.right,
                &data_ids,
                &indicator_ids,
                diagnostics,
                format!("core_ir.indicators[{index}].spread_spec.right"),
            );
        }
        if let Some(custom_expr) = &indicator.custom_expr {
            validate_core_ir_custom_value_refs(
                &custom_expr.predicate.left,
                &data_ids,
                diagnostics,
                format!("core_ir.indicators[{index}].custom_expr.predicate.left"),
            );
            validate_core_ir_custom_value_refs(
                &custom_expr.predicate.right,
                &data_ids,
                diagnostics,
                format!("core_ir.indicators[{index}].custom_expr.predicate.right"),
            );
            if let Some(strength) = &custom_expr.strength {
                validate_core_ir_custom_value_refs(
                    strength,
                    &data_ids,
                    diagnostics,
                    format!("core_ir.indicators[{index}].custom_expr.strength"),
                );
            }
        }
    }

    for (index, signal) in core_ir.signal_rules.iter().enumerate() {
        if !signal.indicator_id.trim().is_empty()
            && !indicator_ids.contains(signal.indicator_id.as_str())
        {
            push_core_ir_v4_bridge_diagnostic(
                diagnostics,
                CoreIrV4BridgeDiagnosticSeverity::Error,
                "V4BRIDGE040",
                format!("core_ir.signal_rules[{index}].indicator_id"),
                format!(
                    "signal references unknown indicator `{}`",
                    signal.indicator_id
                ),
            );
        }
        validate_core_ir_scalar_expr_refs(
            &signal.condition,
            &data_ids,
            &indicator_ids,
            diagnostics,
            format!("core_ir.signal_rules[{index}].condition"),
        );
    }

    for (index, agent) in core_ir.agent_policies.iter().enumerate() {
        for (input_index, signal_id) in agent.input_signal_ids.iter().enumerate() {
            if !signal_id.trim().is_empty() && !signal_ids.contains(signal_id.as_str()) {
                push_core_ir_v4_bridge_diagnostic(
                    diagnostics,
                    CoreIrV4BridgeDiagnosticSeverity::Error,
                    "V4BRIDGE041",
                    format!("core_ir.agent_policies[{index}].input_signal_ids[{input_index}]"),
                    format!("agent references unknown signal `{signal_id}`"),
                );
            }
        }
    }

    for (index, risk) in core_ir.risk_policies.iter().enumerate() {
        for (agent_index, agent_id) in risk.observed_agent_ids.iter().enumerate() {
            if !agent_id.trim().is_empty() && !agent_ids.contains(agent_id.as_str()) {
                push_core_ir_v4_bridge_diagnostic(
                    diagnostics,
                    CoreIrV4BridgeDiagnosticSeverity::Error,
                    "V4BRIDGE042",
                    format!("core_ir.risk_policies[{index}].observed_agent_ids[{agent_index}]"),
                    format!("risk policy references unknown agent `{agent_id}`"),
                );
            }
        }
    }
}

fn validate_core_ir_scalar_expr_refs(
    expr: &crate::ScalarExpr,
    data_ids: &BTreeSet<&str>,
    indicator_ids: &BTreeSet<&str>,
    diagnostics: &mut Vec<CoreIrV4BridgeDiagnostic>,
    target: String,
) {
    match expr {
        crate::ScalarExpr::Number { .. }
        | crate::ScalarExpr::Bool { .. }
        | crate::ScalarExpr::RawText { .. } => {}
        crate::ScalarExpr::Series { expr } => {
            validate_core_ir_series_expr_refs(expr, data_ids, indicator_ids, diagnostics, target);
        }
        crate::ScalarExpr::Ref { name } => {
            if !name.trim().is_empty() && !indicator_ids.contains(name.as_str()) {
                push_core_ir_v4_bridge_diagnostic(
                    diagnostics,
                    CoreIrV4BridgeDiagnosticSeverity::Error,
                    "V4BRIDGE043",
                    target,
                    format!("scalar ref `{name}` is not a declared indicator"),
                );
            }
        }
        crate::ScalarExpr::Compare { left, right, .. } => {
            validate_core_ir_scalar_expr_refs(
                left,
                data_ids,
                indicator_ids,
                diagnostics,
                format!("{target}.left"),
            );
            validate_core_ir_scalar_expr_refs(
                right,
                data_ids,
                indicator_ids,
                diagnostics,
                format!("{target}.right"),
            );
        }
    }
}

fn validate_core_ir_series_expr_refs(
    expr: &crate::SeriesExpr,
    data_ids: &BTreeSet<&str>,
    indicator_ids: &BTreeSet<&str>,
    diagnostics: &mut Vec<CoreIrV4BridgeDiagnostic>,
    target: String,
) {
    match expr {
        crate::SeriesExpr::DataRef { data_id } | crate::SeriesExpr::DataField { data_id, .. } => {
            if !data_id.trim().is_empty() && !data_ids.contains(data_id.as_str()) {
                push_core_ir_v4_bridge_diagnostic(
                    diagnostics,
                    CoreIrV4BridgeDiagnosticSeverity::Error,
                    "V4BRIDGE044",
                    target,
                    format!("series expression references unknown data `{data_id}`"),
                );
            }
        }
        crate::SeriesExpr::Resample { input, .. } | crate::SeriesExpr::WindowAgg { input, .. } => {
            validate_core_ir_series_expr_refs(
                input,
                data_ids,
                indicator_ids,
                diagnostics,
                format!("{target}.input"),
            );
        }
        crate::SeriesExpr::IndicatorRef { indicator_id } => {
            if !indicator_id.trim().is_empty() && !indicator_ids.contains(indicator_id.as_str()) {
                push_core_ir_v4_bridge_diagnostic(
                    diagnostics,
                    CoreIrV4BridgeDiagnosticSeverity::Error,
                    "V4BRIDGE045",
                    target,
                    format!("series expression references unknown indicator `{indicator_id}`"),
                );
            }
        }
        crate::SeriesExpr::RawText { .. } => {}
    }
}

fn validate_core_ir_custom_value_refs(
    value: &crate::CustomValueExpr,
    data_ids: &BTreeSet<&str>,
    diagnostics: &mut Vec<CoreIrV4BridgeDiagnostic>,
    target: String,
) {
    match value {
        crate::CustomValueExpr::Number { .. } => {}
        crate::CustomValueExpr::Input { data_id, .. }
        | crate::CustomValueExpr::WindowAgg { data_id, .. } => {
            if !data_id.trim().is_empty() && !data_ids.contains(data_id.as_str()) {
                push_core_ir_v4_bridge_diagnostic(
                    diagnostics,
                    CoreIrV4BridgeDiagnosticSeverity::Error,
                    "V4BRIDGE046",
                    target,
                    format!("custom expression references unknown data `{data_id}`"),
                );
            }
        }
        crate::CustomValueExpr::Binary { left, right, .. } => {
            validate_core_ir_custom_value_refs(
                left,
                data_ids,
                diagnostics,
                format!("{target}.left"),
            );
            validate_core_ir_custom_value_refs(
                right,
                data_ids,
                diagnostics,
                format!("{target}.right"),
            );
        }
        crate::CustomValueExpr::Unary { value, .. } => {
            validate_core_ir_custom_value_refs(
                value,
                data_ids,
                diagnostics,
                format!("{target}.value"),
            );
        }
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

fn default_machine_contract_version() -> String {
    V4_MACHINE_CONTRACT_VERSION.to_string()
}

fn default_machine_graph_contract_version() -> String {
    V4_MACHINE_GRAPH_CONTRACT_VERSION.to_string()
}

fn default_machine_event_catalog_version() -> String {
    V4_MACHINE_EVENT_CATALOG_VERSION.to_string()
}

fn default_core_ir_compat_bridge_version() -> String {
    V4_CORE_IR_COMPAT_BRIDGE_VERSION.to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        moving_average_compare_expr, AgentPolicy, AgentPolicyKind, ComparisonOp, CoreIREdge,
        CoreIndicatorKind, CoreMetadata, CoreSourceKind, CoreStrategyIr, CoreTimeInForce,
        DataBinding, DataBindingKind, ExecutionRule, ExecutionSizingKind, IndicatorNode,
        RiskPolicy, SeriesExpr, SignalKind, SignalRule,
    };

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
