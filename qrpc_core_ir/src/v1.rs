mod data_indicator_expression_contract;
mod root_graph_contract;

pub use data_indicator_expression_contract::*;
pub use root_graph_contract::*;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalRule {
    pub signal_id: String,
    pub indicator_id: String,
    pub signal_kind: SignalKind,
    pub condition: ScalarExpr,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentPolicy {
    pub agent_id: String,
    pub name: String,
    pub kind: AgentPolicyKind,
    #[serde(default)]
    pub input_signal_ids: Vec<String>,
    #[serde(default)]
    pub rebalance_symbols: Vec<String>,
    #[serde(default)]
    pub rebalance_schedule: Option<RebalanceSchedule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebalance_allocation_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebalance_rank_method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebalance_score_normalize: Option<String>,
    #[serde(default)]
    pub rebalance_target_weights: Vec<f64>,
    #[serde(default)]
    pub decision_threshold: Option<f64>,
    pub max_quantity_ratio: f64,
    #[serde(default)]
    pub spread_trigger_bps: Option<f64>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentPolicyKind {
    WeightedSignals,
    CrossVenueArbitrage,
    PortfolioRebalance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RebalanceSchedule {
    EverySlow,
    Every1d,
    Weekly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskPolicy {
    pub policy_id: String,
    pub name: String,
    #[serde(default)]
    pub observed_agent_ids: Vec<String>,
    pub max_position_ratio: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_single_weight: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_concentration_ratio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_symbol_net_exposure_ratio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_portfolio_net_exposure_ratio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_turnover: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_trade_weight: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_new_positions_per_rebalance: Option<u32>,
    pub max_total_leverage: f64,
    pub max_exchange_leverage: f64,
    pub min_action_interval_ms: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// v1.1.0: 所有标的合计杠杆上限（跨标的联合约束 Phase 2）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cross_symbol_leverage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionSizingKind {
    EquityNotionalRatio,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoreTimeInForce {
    Gtc,
    Ioc,
    Fok,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionRule {
    pub execution_id: String,
    pub venue_kind: String,
    #[serde(default = "default_execution_sizing_kind")]
    pub sizing_kind: ExecutionSizingKind,
    #[serde(default)]
    pub slippage_bps: f64,
    #[serde(default)]
    pub taker_fee_bps: f64,
    #[serde(default)]
    pub total_cost_buffer_bps: f64,
    #[serde(default = "default_time_in_force")]
    pub time_in_force: CoreTimeInForce,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
}

fn default_true() -> bool {
    true
}

fn default_execution_sizing_kind() -> ExecutionSizingKind {
    ExecutionSizingKind::EquityNotionalRatio
}

fn default_time_in_force() -> CoreTimeInForce {
    CoreTimeInForce::Gtc
}

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
