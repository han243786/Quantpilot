use super::{validate_unknownable, validate_unknownable_opt, StrategyIr};

pub(super) fn validate_risk(ir: &StrategyIr, errors: &mut Vec<String>) {
    validate_unknownable(
        &ir.risk_rules.max_position_ratio,
        "risk_rules.max_position_ratio",
        errors,
    );
    validate_unknownable(
        &ir.risk_rules.stop_loss_ratio,
        "risk_rules.stop_loss_ratio",
        errors,
    );
    validate_unknownable_opt(
        ir.risk_rules.take_profit_ratio.as_ref(),
        "risk_rules.take_profit_ratio",
        errors,
    );
    validate_unknownable_opt(
        ir.risk_rules.max_drawdown_ratio.as_ref(),
        "risk_rules.max_drawdown_ratio",
        errors,
    );
    validate_unknownable_opt(
        ir.risk_rules.max_trades_per_day.as_ref(),
        "risk_rules.max_trades_per_day",
        errors,
    );

    if let Some(profile) = &ir.risk_profile {
        if profile.profile_id.trim() != "global" {
            errors.push("risk_profile.profile_id 在当前运行时中必须为 \"global\"".to_string());
        }
        if let Some(value) = profile.max_position {
            if !value.is_finite() || value <= 0.0 {
                errors.push("risk_profile.max_position 必须大于 0".to_string());
            }
        }
        if let Some(value) = profile.max_total_leverage {
            if value < 1.0 {
                errors.push("risk_profile.max_total_leverage 必须大于等于 1".to_string());
            }
        }
        if let Some(value) = profile.max_exchange_leverage {
            if value < 1.0 {
                errors.push("risk_profile.max_exchange_leverage 必须大于等于 1".to_string());
            }
        }
    }
}
