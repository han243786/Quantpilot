use std::collections::BTreeSet;

use super::{
    push_core_ir_v4_bridge_diagnostic, CoreIrV4BridgeDiagnostic, CoreIrV4BridgeDiagnosticSeverity,
};

pub(super) fn validate_core_ir_references_for_v4_bridge(
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
