use serde::{Deserialize, Serialize};

use super::KnownOrUnknown;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DataRequirement {
    pub data_id: String,
    pub venue: KnownOrUnknown<String>,
    pub symbol: KnownOrUnknown<String>,
    pub data_type: DataRequirementType,
    pub granularity: KnownOrUnknown<String>,
    pub lookback: KnownOrUnknown<u32>,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataRequirementType {
    Kline,
    Quote,
    Tick,
    OrderBook,
    Fundamental,
    Event,
}
