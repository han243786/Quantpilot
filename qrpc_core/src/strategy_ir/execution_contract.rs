use serde::{Deserialize, Serialize};

use super::KnownOrUnknown;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StrategyExecution {
    pub venue_type: KnownOrUnknown<String>,
    pub order_type: KnownOrUnknown<String>,
    pub time_in_force: Option<KnownOrUnknown<String>>,
    pub slippage_model: KnownOrUnknown<String>,
    pub latency_assumption_ms: Option<KnownOrUnknown<u32>>,
    pub capital_base: Option<KnownOrUnknown<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyExecutionProfileRef {
    pub profile_id: String,
    #[serde(default)]
    pub fee_bps: Option<f64>,
    #[serde(default)]
    pub slippage_bps: Option<f64>,
}
