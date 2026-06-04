use serde::{Deserialize, Serialize};

use crate::Symbol;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketScope {
    Spot,
    Margin,
    Perpetual,
    Futures,
    Options,
    Index,
    Composite,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrimaryDataType {
    FactPrice,
    KlineRange,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    SpotTrade,
    SpotTicker,
    PerpetualTrade,
    PerpetualMark,
    PerpetualIndex,
    FuturesTrade,
    FuturesMark,
    FuturesIndex,
    IndexPrice,
    Aggregated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Timeframe {
    Tick,
    Ms100,
    Sec1,
    Sec5,
    Min1,
    Min3,
    Min5,
    Min15,
    Min30,
    Hour1,
    Hour4,
    Day1,
    Week1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeRange {
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundingMode {
    Floor,
    Ceil,
    Round,
    Truncate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrecisionPolicy {
    pub price_scale: u8,
    pub quantity_scale: u8,
    pub rounding_mode: RoundingMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsageTag {
    LiveExecution,
    IntentComputation,
    FactSimulation,
    HistoricalBacktest,
    Diagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataRequest {
    pub request_id: String,
    pub instrument: Symbol,
    pub market_scope: MarketScope,
    pub primary_data_type: PrimaryDataType,
    pub source_type: SourceType,
    pub timeframe: Option<Timeframe>,
    pub lookback_count: Option<u32>,
    pub time_range: Option<TimeRange>,
    pub precision_policy: PrecisionPolicy,
    pub usage_tag: UsageTag,
    pub priority: u8,
    pub is_realtime: bool,
    pub requested_at_ms: u64,
}
