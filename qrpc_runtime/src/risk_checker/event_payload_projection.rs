use super::*;
use serde_json::{json, Value};

pub(super) fn build_risk_event_payload(
    provider_key: &str,
    decision: &AgentDecision,
    risk_decision: &RiskDecision,
    portfolio: &PortfolioState,
) -> Value {
    let reason_codes = risk_decision
        .reason_codes
        .iter()
        .map(reason_code_name)
        .collect::<Vec<_>>();
    let limit_triggered = risk_decision
        .reason_codes
        .iter()
        .find(|code| !matches!(code, RiskReasonCode::WithinLimit))
        .map(reason_code_name);

    let payload = if let Some(original_target) = &decision.portfolio_target_decision {
        let adjusted_target = risk_decision
            .adjusted_portfolio_target_decision
            .as_ref()
            .map(|item| &item.target.target_weights[..])
            .unwrap_or(&[]);
        json!({
            "sizing_mode": "portfolio_target",
            "pre_risk": target_weight_stats(&original_target.target.target_weights),
            "post_risk": target_weight_stats(adjusted_target),
        })
    } else {
        let equity = portfolio_equity(portfolio).abs().max(1.0);
        json!({
            "sizing_mode": "action_list",
            "pre_risk": action_stats(&decision.proposed_actions, portfolio, &decision.symbol, equity),
            "post_risk": action_stats(&risk_decision.adjusted_actions, portfolio, &decision.symbol, equity),
        })
    };

    let mut object = json!({
        "provider_key": provider_key,
        "status": format!("{:?}", risk_decision.status),
        "reasons": reason_codes,
        "reason_text": risk_decision.reason_text,
        "limit_triggered": limit_triggered,
        "explanation_summary": risk_explanation_summary(risk_decision),
        "agent_decision_id": decision.decision_id,
        "risk_id": risk_decision.risk_id,
    });

    if let (Value::Object(object_map), Value::Object(payload_map)) = (&mut object, payload) {
        object_map.extend(payload_map);
    }

    object
}

fn risk_explanation_summary(risk_decision: &RiskDecision) -> String {
    let limit = risk_decision
        .reason_codes
        .iter()
        .find(|code| !matches!(code, RiskReasonCode::WithinLimit))
        .map(reason_code_name);
    match (&risk_decision.status, limit) {
        (DecisionStatus::Approve, _) => "Risk approved without clamp.".to_string(),
        (DecisionStatus::Clamp, Some(limit)) => {
            format!("Risk clamped sizing after triggering {limit}.")
        }
        (DecisionStatus::Reject, Some(limit)) => {
            format!("Risk rejected the decision because {limit} triggered.")
        }
        (DecisionStatus::Clamp, None) => "Risk clamped sizing before execution.".to_string(),
        (DecisionStatus::Reject, None) => "Risk rejected the decision.".to_string(),
    }
}

fn reason_code_name(reason_code: &RiskReasonCode) -> &'static str {
    match reason_code {
        RiskReasonCode::WithinLimit => "within_limit",
        RiskReasonCode::ExceedTotalLeverage => "max_total_leverage",
        RiskReasonCode::ExceedExchangeLeverage => "max_exchange_leverage",
        RiskReasonCode::ExceedSingleWeight => "max_single_weight",
        RiskReasonCode::ExceedConcentration => "max_concentration_ratio",
        RiskReasonCode::ExceedSymbolNetExposure => "max_symbol_net_exposure_ratio",
        RiskReasonCode::ExceedPortfolioNetExposure => "max_portfolio_net_exposure_ratio",
        RiskReasonCode::ExceedTurnover => "max_turnover",
        RiskReasonCode::TradeBelowMinimum => "min_trade_weight",
        RiskReasonCode::ExceedNewPositionsLimit => "max_new_positions_per_rebalance",
        RiskReasonCode::ActionTooFrequent => "min_action_interval_ms",
        RiskReasonCode::DirectionConflict => "direction_conflict",
        RiskReasonCode::InsufficientCash => "available_cash_balance",
        RiskReasonCode::InsufficientInventory => "available_inventory",
        RiskReasonCode::CostNotCovered => "cost_not_covered",
        RiskReasonCode::InvalidAction => "invalid_action",
        RiskReasonCode::ExceedDailyLoss => "max_daily_loss",
        RiskReasonCode::ExceedDrawdown => "max_drawdown",
    }
}

fn action_stats(
    actions: &[qrpc_core::ProposedAction],
    portfolio: &PortfolioState,
    symbol: &Symbol,
    equity: f64,
) -> Value {
    let total_quantity_ratio = actions.iter().map(|item| item.quantity_ratio).sum::<f64>();
    let buy_quantity_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Buy))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    let sell_quantity_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Sell))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    let portfolio_net_exposure_ratio =
        portfolio_net_exposure_ratio(portfolio, equity) + buy_quantity_ratio - sell_quantity_ratio;
    let symbol_net_exposure_ratio = symbol_net_exposure_ratio(portfolio, symbol, equity)
        + buy_quantity_ratio
        - sell_quantity_ratio;

    json!({
        "action_count": actions.len(),
        "total_quantity_ratio": total_quantity_ratio,
        "buy_quantity_ratio": buy_quantity_ratio,
        "sell_quantity_ratio": sell_quantity_ratio,
        "portfolio_net_exposure_ratio": portfolio_net_exposure_ratio.max(0.0),
        "symbol_net_exposure_ratio": symbol_net_exposure_ratio.max(0.0),
    })
}

fn target_weight_stats(target_weights: &[qrpc_core::TargetWeight]) -> Value {
    let turnover_ratio = target_weights
        .iter()
        .map(|item| (item.target_weight - item.current_weight).abs())
        .sum::<f64>();
    let max_target_weight = target_weights
        .iter()
        .map(|item| item.target_weight.abs())
        .fold(0.0_f64, f64::max);
    let portfolio_net_exposure_ratio = target_weights
        .iter()
        .map(|item| item.target_weight.abs())
        .sum::<f64>();
    let new_positions = target_weights
        .iter()
        .filter(|item| item.current_weight <= 1e-9 && item.target_weight > 1e-9)
        .count();

    json!({
        "basket_members": target_weights.len(),
        "turnover_ratio": turnover_ratio,
        "max_target_weight": max_target_weight,
        "concentration_ratio": max_target_weight,
        "max_symbol_net_exposure_ratio": max_target_weight,
        "portfolio_net_exposure_ratio": portfolio_net_exposure_ratio,
        "new_positions": new_positions,
    })
}
