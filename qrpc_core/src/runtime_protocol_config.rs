use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{
    CoreStrategyIr, DataKind, Exchange, IntentKind, MarketType, RebalanceSchedule, SourceHealth,
    Symbol,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DataQualitySnapshot {
    #[serde(default)]
    pub source_health: SourceHealth,
    #[serde(default)]
    pub freshness_ms: u64,
    #[serde(default)]
    pub stale_after_ms: u64,
    #[serde(default)]
    pub gap_count: u64, // v2.1.x: u32→u64 防截断
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quality_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeEventType {
    DataUpdated,
    IntentEvaluated,
    IntentTriggered,
    AgentDecisionProduced,
    RiskDecisionProduced,
    ExecutionPlanned,
    ExecutionFilled,
    PortfolioUpdated,
    RuntimeWarning,
    RuntimeError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceConfig {
    pub data_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub market_type: MarketType,
    pub kind: DataKind,
    pub days: Option<u32>,
    pub interval: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub ping_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_interval_ms: Option<u64>,
    pub enabled: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UniverseAssetMetadataPoint {
    pub as_of_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_cap: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_24h: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listing_age_days: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UniverseAssetRecord {
    pub symbol: Symbol,
    pub exchange: Exchange,
    pub market_type: MarketType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_cap: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_24h: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listing_age_days: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listed_at_ms: Option<u64>,
    #[serde(default)]
    pub metadata_history: Vec<UniverseAssetMetadataPoint>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UniverseSnapshot {
    pub snapshot_id: String,
    pub as_of_ms: u64,
    #[serde(default)]
    pub assets: Vec<UniverseAssetRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentConfig {
    pub intent_id: String,
    pub name: String,
    pub kind: IntentKind,
    pub input_data_ids: Vec<String>,
    pub params: BTreeMap<String, f64>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub name: String,
    pub input_intent_ids: Vec<String>,
    #[serde(default)]
    pub rebalance_symbols: Vec<Symbol>,
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
    pub params: BTreeMap<String, f64>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub risk_id: String,
    pub name: String,
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
    pub enabled: bool,
}

pub const GLOBAL_RISK_PROFILE_ID: &str = "global";
pub const GLOBAL_RISK_PROFILE_DEFAULT_MAX_POSITION: f64 = 0.2;
pub const GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE: f64 = 3.0;
pub const GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE: f64 = 3.0;
pub const GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS: u64 = 100;
pub const PAPER_EXECUTION_PROFILE_ID: &str = "paper";
pub const PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS: f64 = 10.0;
pub const PAPER_EXECUTION_PROFILE_DEFAULT_SLIPPAGE_BPS: f64 = 5.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeProtocolCoreConfig {
    pub data_sources: Vec<DataSourceConfig>,
    pub intents: Vec<IntentConfig>,
    pub agents: Vec<AgentConfig>,
    pub risks: Vec<RiskConfig>,
    pub initial_cash_balance: f64,
    pub taker_fee_bps: f64,
    pub default_slippage_bps: f64,
    pub total_cost_buffer_bps: f64,
}

impl Default for RuntimeProtocolCoreConfig {
    fn default() -> Self {
        Self {
            data_sources: Vec::new(),
            intents: Vec::new(),
            agents: Vec::new(),
            risks: Vec::new(),
            initial_cash_balance: 100_000.0,
            taker_fee_bps: PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS,
            default_slippage_bps: 5.0,
            total_cost_buffer_bps: 20.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledRuntimeProtocol {
    pub protocol_name: String,
    pub config_hash: String,
    pub config: RuntimeProtocolCoreConfig,
    pub core_ir: CoreStrategyIr,
}
