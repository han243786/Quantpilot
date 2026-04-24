use qrpc_core::{RebalanceSchedule, Symbol, UniverseSnapshot};

#[derive(Debug, Clone, Default)]
pub struct LoweringContext {
    pub universe_snapshot: Option<UniverseSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstrumentPoolSourceSpec {
    ExplicitSymbols,
    Universe {
        exchange: Option<String>,
        market: Option<String>,
        quote: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstrumentPoolValue {
    String(String),
    Number(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentPoolEligibilityRule {
    pub field: String,
    pub op: String,
    pub value: InstrumentPoolValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentPoolFeatureDef {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstrumentPoolSelectionKey {
    Symbol,
    MetadataField(String),
    Feature(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentPoolSelectionRule {
    pub kind: String,
    pub key: Option<InstrumentPoolSelectionKey>,
    pub order: Option<String>,
    pub count: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentPoolWeightingRule {
    pub kind: String,
    pub method: Option<String>,
    pub normalize: Option<String>,
    pub target_weights: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentPoolRebalanceRule {
    pub every: Option<RebalanceSchedule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentPoolSpec {
    pub source: InstrumentPoolSourceSpec,
    pub eligibility_rules: Vec<InstrumentPoolEligibilityRule>,
    pub feature_defs: Vec<InstrumentPoolFeatureDef>,
    pub selection_rule: Option<InstrumentPoolSelectionRule>,
    pub weighting_rule: Option<InstrumentPoolWeightingRule>,
    pub rebalance_rule: Option<InstrumentPoolRebalanceRule>,
}

#[derive(Debug, Clone)]
pub(crate) struct PortfolioRebalanceDirective {
    pub(crate) symbols: Vec<Symbol>,
    pub(crate) schedule: Option<RebalanceSchedule>,
    pub(crate) allocation_kind: String,
    pub(crate) rank_method: Option<String>,
    pub(crate) score_normalize: Option<String>,
    pub(crate) target_weights: Vec<f64>,
    #[allow(dead_code)]
    pub(crate) instrument_pool: InstrumentPoolSpec,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct UniverseAssetMetrics {
    pub(crate) market_cap: Option<f64>,
    pub(crate) volume_24h: Option<f64>,
    pub(crate) listing_age_days: Option<f64>,
}
