mod reference_validation;

use reference_validation::validate_core_ir_references_for_v4_bridge;
use std::collections::BTreeSet;

use super::{
    push_core_ir_v4_bridge_diagnostic, CoreIrV4BridgeDiagnostic, CoreIrV4BridgeDiagnosticSeverity,
};

pub(super) fn validate_core_ir_for_v4_bridge(
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
