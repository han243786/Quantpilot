use super::{validate_unknownable, validate_unknownable_opt, StrategyIr};

pub(super) fn validate_data_and_execution(ir: &StrategyIr, errors: &mut Vec<String>) {
    for (index, requirement) in ir.data_requirements.iter().enumerate() {
        if requirement.data_id.trim().is_empty() {
            errors.push(format!("data_requirements[{index}].data_id 是必需的"));
        }
        if requirement.fields.is_empty() {
            errors.push(format!(
                "data_requirements[{index}].fields 必须包含至少一个字段"
            ));
        }
        validate_unknownable(
            &requirement.venue,
            &format!("data_requirements[{index}].venue"),
            errors,
        );
        validate_unknownable(
            &requirement.symbol,
            &format!("data_requirements[{index}].symbol"),
            errors,
        );
        validate_unknownable(
            &requirement.granularity,
            &format!("data_requirements[{index}].granularity"),
            errors,
        );
        validate_unknownable(
            &requirement.lookback,
            &format!("data_requirements[{index}].lookback"),
            errors,
        );
    }

    validate_unknownable(&ir.execution.venue_type, "execution.venue_type", errors);
    validate_unknownable(&ir.execution.order_type, "execution.order_type", errors);
    validate_unknownable(
        &ir.execution.slippage_model,
        "execution.slippage_model",
        errors,
    );
    validate_unknownable_opt(
        ir.execution.time_in_force.as_ref(),
        "execution.time_in_force",
        errors,
    );
    validate_unknownable_opt(
        ir.execution.latency_assumption_ms.as_ref(),
        "execution.latency_assumption_ms",
        errors,
    );
    validate_unknownable_opt(
        ir.execution.capital_base.as_ref(),
        "execution.capital_base",
        errors,
    );

    if let Some(profile) = &ir.execution_profile {
        if profile.profile_id.trim() != "paper" {
            errors.push("execution_profile.profile_id 在当前运行时中必须为 \"paper\"".to_string());
        }
        if let Some(value) = profile.fee_bps {
            if !value.is_finite() || value < 0.0 {
                errors.push("execution_profile.fee_bps 必须是有限数且大于等于 0".to_string());
            }
        }
        if let Some(value) = profile.slippage_bps {
            if !value.is_finite() || value < 0.0 {
                errors.push("execution_profile.slippage_bps 必须是有限数且大于等于 0".to_string());
            }
        }
    }
}
