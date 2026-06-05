use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataBinding {
    pub data_id: String,
    pub kind: DataBindingKind,
    #[serde(default)]
    pub source_hints: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataBindingKind {
    KlineSeries,
    Quote,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndicatorNode {
    pub indicator_id: String,
    pub kind: CoreIndicatorKind,
    #[serde(default)]
    pub inputs: Vec<SeriesExpr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spread_spec: Option<SpreadSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_expr: Option<CustomExprSpec>,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CoreIndicatorKind {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomExprSpec {
    pub schema_version: String,
    pub signal_kind: SignalKind,
    pub predicate: CustomPredicateExpr,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strength: Option<CustomValueExpr>,
    #[serde(default = "default_custom_confidence")]
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomPredicateExpr {
    pub left: CustomValueExpr,
    pub op: ComparisonOp,
    pub right: CustomValueExpr,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CustomValueExpr {
    Number {
        value: f64,
    },
    Input {
        data_id: String,
        field: SeriesField,
    },
    WindowAgg {
        data_id: String,
        field: SeriesField,
        window_size: usize,
        agg: SeriesAggregation,
    },
    Binary {
        left: Box<CustomValueExpr>,
        op: ArithmeticOp,
        right: Box<CustomValueExpr>,
    },
    Unary {
        op: ArithmeticUnaryOp,
        value: Box<CustomValueExpr>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArithmeticUnaryOp {
    Abs,
    Negate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SeriesExpr {
    DataRef {
        data_id: String,
    },
    DataField {
        data_id: String,
        field: SeriesField,
    },
    Resample {
        input: Box<SeriesExpr>,
        period_ms: u64,
        agg: SeriesAggregation,
    },
    WindowAgg {
        input: Box<SeriesExpr>,
        window_size: usize,
        agg: SeriesAggregation,
    },
    IndicatorRef {
        indicator_id: String,
    },
    RawText {
        source: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SeriesField {
    MidOrClose,
    BidOrClose,
    AskOrClose,
    Close,
    Open,
    High,
    Low,
    Volume,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SeriesAggregation {
    Last,
    Mean,
    Sum,
    Min,
    Max,
    StdDev,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlignDirection {
    Backward,
    Forward,
    Nearest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlignAsofSpec {
    pub direction: AlignDirection,
    pub tolerance_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpreadValueKind {
    Ratio,
    Bps,
    Absolute,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpreadSpec {
    pub left: SeriesExpr,
    pub right: SeriesExpr,
    pub align: AlignAsofSpec,
    pub output: SpreadValueKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    Long,
    Short,
    Observe,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScalarExpr {
    Number {
        value: f64,
    },
    Bool {
        value: bool,
    },
    Series {
        expr: SeriesExpr,
    },
    Ref {
        name: String,
    },
    Compare {
        left: Box<ScalarExpr>,
        op: ComparisonOp,
        right: Box<ScalarExpr>,
    },
    RawText {
        source: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOp {
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
}

pub fn close_series_expr(data_id: impl Into<String>) -> SeriesExpr {
    SeriesExpr::DataField {
        data_id: data_id.into(),
        field: SeriesField::Close,
    }
}

pub fn moving_average_series_expr(data_id: impl Into<String>, period: usize) -> Option<SeriesExpr> {
    if period == 0 {
        return None;
    }

    Some(SeriesExpr::WindowAgg {
        input: Box::new(close_series_expr(data_id)),
        window_size: period,
        agg: SeriesAggregation::Mean,
    })
}

pub fn moving_average_compare_expr(
    data_id: impl Into<String>,
    left_period: usize,
    op: ComparisonOp,
    right_period: usize,
) -> Option<ScalarExpr> {
    let data_id = data_id.into();
    let left = moving_average_series_expr(data_id.clone(), left_period)?;
    let right = moving_average_series_expr(data_id, right_period)?;
    Some(ScalarExpr::Compare {
        left: Box::new(ScalarExpr::Series { expr: left }),
        op,
        right: Box::new(ScalarExpr::Series { expr: right }),
    })
}

pub fn indicator_threshold_compare_expr(
    indicator_id: impl Into<String>,
    op: ComparisonOp,
    threshold: f64,
) -> Option<ScalarExpr> {
    if !threshold.is_finite() {
        return None;
    }

    Some(ScalarExpr::Compare {
        left: Box::new(ScalarExpr::Ref {
            name: indicator_id.into(),
        }),
        op,
        right: Box::new(ScalarExpr::Number { value: threshold }),
    })
}

fn default_custom_confidence() -> f64 {
    0.8
}
