use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SignalDefinition {
    pub signal_id: String,
    pub name: String,
    pub indicator: IndicatorDefinition,
    #[serde(default)]
    pub transforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IndicatorDefinition {
    pub kind: IndicatorKind,
    pub inputs: Vec<String>,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IndicatorKind {
    MaCross,
    Rsi,
    Macd,
    Momentum,
    Spread,
    ZScore,
    Custom,
    QuoteObserve,
    Atr,
    BollingerBands,
    Obv,
    Cmf,
    Adx,
    Stochastic,
    Cci,
    ParabolicSar,
    KeltnerChannel,
    DonchianChannel,
}

const DECLARED_INDICATOR_KINDS: [IndicatorKind; 18] = [
    IndicatorKind::MaCross,
    IndicatorKind::Rsi,
    IndicatorKind::Macd,
    IndicatorKind::Momentum,
    IndicatorKind::Spread,
    IndicatorKind::ZScore,
    IndicatorKind::Custom,
    IndicatorKind::QuoteObserve,
    IndicatorKind::Atr,
    IndicatorKind::BollingerBands,
    IndicatorKind::Obv,
    IndicatorKind::Cmf,
    IndicatorKind::Adx,
    IndicatorKind::Stochastic,
    IndicatorKind::Cci,
    IndicatorKind::ParabolicSar,
    IndicatorKind::KeltnerChannel,
    IndicatorKind::DonchianChannel,
];

const SUPPORTED_INDICATOR_KINDS: [IndicatorKind; 18] = [
    IndicatorKind::MaCross,
    IndicatorKind::Rsi,
    IndicatorKind::Macd,
    IndicatorKind::Momentum,
    IndicatorKind::Spread,
    IndicatorKind::ZScore,
    IndicatorKind::Custom,
    IndicatorKind::QuoteObserve,
    IndicatorKind::Atr,
    IndicatorKind::BollingerBands,
    IndicatorKind::Obv,
    IndicatorKind::Cmf,
    IndicatorKind::Adx,
    IndicatorKind::Stochastic,
    IndicatorKind::Cci,
    IndicatorKind::ParabolicSar,
    IndicatorKind::KeltnerChannel,
    IndicatorKind::DonchianChannel,
];

pub fn declared_indicator_kinds() -> &'static [IndicatorKind] {
    &DECLARED_INDICATOR_KINDS
}

pub fn supported_indicator_kinds() -> &'static [IndicatorKind] {
    &SUPPORTED_INDICATOR_KINDS
}
