use super::{indicator_kind_supported, validate_unknownable, IndicatorKind, LogicRule, StrategyIr};

pub(super) fn validate_signal_and_logic(ir: &StrategyIr, errors: &mut Vec<String>) {
    for (index, signal) in ir.signals.iter().enumerate() {
        if signal.signal_id.trim().is_empty() {
            errors.push(format!("signals[{index}].signal_id 是必需的"));
        }
        if signal.name.trim().is_empty() {
            errors.push(format!("signals[{index}].name 是必需的"));
        }
        if signal.indicator.inputs.is_empty() {
            errors.push(format!(
                "signals[{index}].indicator.inputs 必须包含至少一个输入"
            ));
        }
        if matches!(signal.indicator.kind, IndicatorKind::Spread)
            && signal.indicator.inputs.len() < 2
        {
            errors.push(format!(
                "signals[{index}].indicator.inputs 对于 spread 必须包含至少两个输入"
            ));
        }
        if !indicator_kind_supported(&signal.indicator.kind) {
            errors.push(format!(
                "signals[{index}].indicator.kind {:?} 不被当前运行时支持",
                signal.indicator.kind
            ));
        }
    }

    for (index, rule) in ir.logic.entry_rules.iter().enumerate() {
        validate_logic_rule(rule, &format!("logic.entry_rules[{index}]"), errors);
    }
    for (index, rule) in ir.logic.exit_rules.iter().enumerate() {
        validate_logic_rule(rule, &format!("logic.exit_rules[{index}]"), errors);
    }

    validate_unknownable(
        &ir.logic.position_sizing.value,
        "logic.position_sizing.value",
        errors,
    );
    if let Some(rule) = &ir.logic.rebalance_rule {
        validate_unknownable(&rule.frequency, "logic.rebalance_rule.frequency", errors);
    }
}

fn validate_logic_rule(rule: &LogicRule, path: &str, errors: &mut Vec<String>) {
    if rule.rule_id.trim().is_empty() {
        errors.push(format!("{path}.rule_id 是必需的"));
    }
    if rule.condition.trim().is_empty() {
        errors.push(format!("{path}.condition 是必需的"));
    }
}
