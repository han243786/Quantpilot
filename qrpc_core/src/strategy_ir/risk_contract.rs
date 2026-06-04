use serde::{Deserialize, Serialize};

use super::KnownOrUnknown;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StrategyRiskRules {
    pub max_position_ratio: KnownOrUnknown<f64>,
    pub stop_loss_ratio: KnownOrUnknown<f64>,
    pub take_profit_ratio: Option<KnownOrUnknown<f64>>,
    pub max_drawdown_ratio: Option<KnownOrUnknown<f64>>,
    pub max_trades_per_day: Option<KnownOrUnknown<u32>>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyRiskProfileRef {
    pub profile_id: String,
    #[serde(default)]
    pub max_position: Option<f64>,
    #[serde(default)]
    pub max_total_leverage: Option<f64>,
    #[serde(default)]
    pub max_exchange_leverage: Option<f64>,
    #[serde(default)]
    pub min_action_interval_ms: Option<u64>,
}
