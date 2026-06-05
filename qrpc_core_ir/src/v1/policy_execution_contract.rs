use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{ScalarExpr, SignalKind};

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
