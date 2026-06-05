mod data_indicator_expression_contract;
mod policy_execution_contract;
mod root_graph_contract;

pub use data_indicator_expression_contract::*;
pub use policy_execution_contract::*;
pub use root_graph_contract::*;

#[cfg(test)]
use std::collections::BTreeMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_ir_round_trips() {
        let mut core_ir = CoreStrategyIr::new(
            CoreMetadata {
                strategy_id: "sample".into(),
                name: "Sample".into(),
                source_kind: CoreSourceKind::StrategyIr,
            },
            ExecutionRule {
                execution_id: "exec".into(),
                venue_kind: "paper".into(),
                sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
                slippage_bps: 5.0,
                taker_fee_bps: 10.0,
                total_cost_buffer_bps: 20.0,
                time_in_force: CoreTimeInForce::Gtc,
                params: BTreeMap::new(),
            },
        );
        core_ir.data_bindings.push(DataBinding {
            data_id: "btc_1d".into(),
            kind: DataBindingKind::KlineSeries,
            source_hints: BTreeMap::new(),
        });
        core_ir.indicators.push(IndicatorNode {
            indicator_id: "rsi_1".into(),
            kind: CoreIndicatorKind::Rsi,
            inputs: vec![SeriesExpr::DataRef {
                data_id: "btc_1d".into(),
            }],
            spread_spec: None,
            custom_expr: None,
            params: BTreeMap::new(),
        });
        core_ir.signal_rules.push(SignalRule {
            signal_id: "rule_1".into(),
            indicator_id: "rsi_1".into(),
            signal_kind: SignalKind::Long,
            condition: moving_average_compare_expr("btc_1d", 20, ComparisonOp::Gt, 100).unwrap(),
        });
        core_ir.agent_policies.push(AgentPolicy {
            agent_id: "agent_1".into(),
            name: "Weighted Agent".into(),
            kind: AgentPolicyKind::WeightedSignals,
            input_signal_ids: vec!["rsi_1".into()],
            rebalance_symbols: vec![],
            rebalance_schedule: None,
            rebalance_allocation_kind: None,
            rebalance_rank_method: None,
            rebalance_score_normalize: None,
            rebalance_target_weights: vec![],
            decision_threshold: Some(0.05),
            max_quantity_ratio: 0.2,
            spread_trigger_bps: None,
            enabled: true,
        });
        core_ir.risk_policies.push(RiskPolicy {
            policy_id: "risk_1".into(),
            name: "Global Risk".into(),
            observed_agent_ids: vec!["agent_1".into()],
            max_position_ratio: 0.2,
            max_single_weight: None,
            max_concentration_ratio: None,
            max_symbol_net_exposure_ratio: None,
            max_portfolio_net_exposure_ratio: None,
            max_turnover: None,
            min_trade_weight: None,
            max_new_positions_per_rebalance: None,
            max_total_leverage: 3.0,
            max_exchange_leverage: 3.0,
            min_action_interval_ms: 1000,
            enabled: true,
            max_cross_symbol_leverage: None,
        });

        let encoded = serde_json::to_string(&core_ir).unwrap();
        let decoded: CoreStrategyIr = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.ir_version, CORE_IR_V1_VERSION);
        assert_eq!(decoded, core_ir);
    }
}
