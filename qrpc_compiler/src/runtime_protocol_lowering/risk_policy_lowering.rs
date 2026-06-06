use qrpc_core::RiskConfig;
use qrpc_core_ir::RiskPolicy;

pub(super) fn lower_runtime_risk_to_policy(risk: &RiskConfig) -> RiskPolicy {
    RiskPolicy {
        policy_id: risk.risk_id.clone(),
        name: risk.name.clone(),
        observed_agent_ids: risk.observed_agent_ids.clone(),
        max_position_ratio: risk.max_position_ratio,
        max_single_weight: risk.max_single_weight,
        max_concentration_ratio: risk.max_concentration_ratio,
        max_symbol_net_exposure_ratio: risk.max_symbol_net_exposure_ratio,
        max_portfolio_net_exposure_ratio: risk.max_portfolio_net_exposure_ratio,
        max_turnover: risk.max_turnover,
        min_trade_weight: risk.min_trade_weight,
        max_new_positions_per_rebalance: risk.max_new_positions_per_rebalance,
        max_total_leverage: risk.max_total_leverage,
        max_exchange_leverage: risk.max_exchange_leverage,
        min_action_interval_ms: risk.min_action_interval_ms,
        enabled: risk.enabled,
        max_cross_symbol_leverage: None,
    }
}
