use std::collections::BTreeSet;

use super::{StrategyIr, STRATEGY_IR_V0_VERSION};

pub(super) fn validate_identity_and_required_fields(ir: &StrategyIr, errors: &mut Vec<String>) {
    if ir.ir_version != STRATEGY_IR_V0_VERSION {
        errors.push(format!(
            "ir_version 必须是 {STRATEGY_IR_V0_VERSION}，但实际为 {}",
            ir.ir_version
        ));
    }

    if ir.metadata.strategy_id.trim().is_empty() {
        errors.push("metadata.strategy_id 是必需的".to_string());
    }
    if ir.metadata.name.trim().is_empty() {
        errors.push("metadata.name 是必需的".to_string());
    }
    if ir.metadata.summary.trim().is_empty() {
        errors.push("metadata.summary 是必需的".to_string());
    }

    if ir.signals.is_empty() {
        errors.push("signals 必须包含至少一个信号".to_string());
    }
    if ir.data_requirements.is_empty() {
        errors.push("data_requirements 必须包含至少一个数据要求".to_string());
    }
    if ir.logic.entry_rules.is_empty() {
        errors.push("logic.entry_rules 必须包含至少一条规则".to_string());
    }

    validate_unique_ids(
        ir.signals.iter().map(|item| item.signal_id.as_str()),
        "signals",
        errors,
    );
    validate_unique_ids(
        ir.data_requirements
            .iter()
            .map(|item| item.data_id.as_str()),
        "data_requirements",
        errors,
    );
    validate_unique_ids(
        ir.logic
            .entry_rules
            .iter()
            .chain(ir.logic.exit_rules.iter())
            .map(|item| item.rule_id.as_str()),
        "logic rules",
        errors,
    );
}

fn validate_unique_ids<'a>(
    values: impl Iterator<Item = &'a str>,
    label: &str,
    errors: &mut Vec<String>,
) {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value.to_string()) {
            errors.push(format!("{label} 包含重复的 id: {value}"));
        }
    }
}
