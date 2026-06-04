use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};

pub const RUN_SPEC_V1_VERSION: &str = "quantpilot/run-spec/v1";
pub const BACKTEST_SPEC_V1_VERSION: &str = "quantpilot/backtest-spec/v1";
pub const STRATEGY_ARTIFACT_V1_VERSION: &str = "quantpilot/strategy-artifact/v1";
pub const COMPILE_ARTIFACT_V1_VERSION: &str = "quantpilot/compile-artifact/v1";
pub const CORE_IR_ARTIFACT_V1_VERSION: &str = "quantpilot/core-ir-artifact/v1";
pub const EVENT_ENVELOPE_PROTO_VERSION: &str = "quantpilot/events/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Exchange {
    Binance,
    Okx,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol {
    BtcUsdt,
    Other(String),
}

impl Symbol {
    pub fn parse(input: &str) -> Self {
        match input.trim().to_ascii_uppercase().as_str() {
            "BTCUSDT" => Self::BtcUsdt,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::BtcUsdt => "BTCUSDT",
            Self::Other(value) => value.as_str(),
        }
    }
}

impl Serialize for Symbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.trim().is_empty() {
            return Err(D::Error::custom("交易品种符号不能为空"));
        }
        Ok(Symbol::parse(&value))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MarketType {
    Spot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RebalanceSchedule {
    EverySlow,
    Every1d,
    Weekly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataKind {
    KlineSeries,
    Quote,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntentKind {
    LongTermBuy,
    LongTermSell,
    Rsi,
    Macd,
    Momentum,
    ZScore,
    QuoteObserve,
    SmaCrossover,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalSide {
    Long,
    Short,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionStatus {
    Approve,
    Clamp,
    Reject,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskDecisionMode {
    Normal,
    FreezeOpen,
    ReduceOnly,
    ReconcileOnly,
    EmergencyHalt,
}

impl Default for RiskDecisionMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| format!("{:?}", self))
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    Accepted,
    Open,
    PartiallyFilled,
    Planned,
    Filled,
    Cancelled,
    Rejected,
    Expired,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskReasonCode {
    WithinLimit,
    ExceedTotalLeverage,
    ExceedExchangeLeverage,
    ExceedSingleWeight,
    ExceedConcentration,
    ExceedSymbolNetExposure,
    ExceedPortfolioNetExposure,
    ExceedTurnover,
    TradeBelowMinimum,
    ExceedNewPositionsLimit,
    ActionTooFrequent,
    DirectionConflict,
    InsufficientCash,
    InsufficientInventory,
    CostNotCovered,
    InvalidAction,
    /// v1.2.0: 当日累计亏损超过限制（RiskMonitor 触发）
    ExceedDailyLoss,
    /// v1.2.0: 实时回撤超过限制（RiskMonitor 触发）
    ExceedDrawdown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceStatus {
    Healthy,
    Stale,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceHealth {
    Healthy,
    Delayed,
    Stale,
    Missing,
    Error,
}

impl Default for SourceHealth {
    fn default() -> Self {
        Self::Healthy
    }
}
