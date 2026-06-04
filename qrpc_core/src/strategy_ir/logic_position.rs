use serde::{Deserialize, Serialize};

use super::KnownOrUnknown;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StrategyLogic {
    pub entry_rules: Vec<LogicRule>,
    #[serde(default)]
    pub exit_rules: Vec<LogicRule>,
    pub position_sizing: PositionSizing,
    pub rebalance_rule: Option<RebalanceRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogicRule {
    pub rule_id: String,
    pub condition: String,
    pub action: LogicAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogicAction {
    OpenLong,
    CloseLong,
    OpenShort,
    CloseShort,
    Rebalance,
    Hold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionSizing {
    pub method: PositionSizingMethod,
    pub value: KnownOrUnknown<f64>,
    pub unit: PositionSizingUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PositionSizingMethod {
    FixedRatio,
    VolatilityTarget,
    EqualWeight,
    RiskParity,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PositionSizingUnit {
    PortfolioRatio,
    Leverage,
    Quantity,
    Notional,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RebalanceRule {
    pub frequency: KnownOrUnknown<String>,
    pub condition: Option<String>,
}
