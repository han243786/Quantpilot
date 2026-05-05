mod plugin;
mod strategy_ir;

pub use plugin::*;
pub use qrpc_core_ir::{CoreStrategyIr, CORE_IR_V1_VERSION};
pub use strategy_ir::*;

use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

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
            return Err(D::Error::custom("symbol cannot be empty"));
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DataQualitySnapshot {
    #[serde(default)]
    pub source_health: SourceHealth,
    #[serde(default)]
    pub freshness_ms: u64,
    #[serde(default)]
    pub stale_after_ms: u64,
    #[serde(default)]
    pub gap_count: u32,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactDigestAlgorithm {
    Sha256CanonicalJson,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactDigest {
    pub algorithm: ArtifactDigestAlgorithm,
    pub value: String,
}

pub fn canonical_json_sha256_digest<T: Serialize + ?Sized>(
    value: &T,
) -> serde_json::Result<ArtifactDigest> {
    let canonical = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(&canonical);
    Ok(ArtifactDigest {
        algorithm: ArtifactDigestAlgorithm::Sha256CanonicalJson,
        value: format!("{:x}", hasher.finalize()),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunModeSpec {
    Paper,
    Backtest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BacktestReplaySource {
    HistoricalReplay,
    DeterministicMock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StrategyArtifactSourceKind {
    FrontendGraph,
    RuntimeProtocol,
    StrategyIr,
    FormalQuantScript,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetSpec {
    pub dataset_id: String,
    pub data_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub market_type: MarketType,
    pub kind: DataKind,
    pub interval: Option<String>,
    pub lookback_days: Option<u32>,
    pub enabled: bool,
}

impl From<&DataSourceConfig> for DatasetSpec {
    fn from(value: &DataSourceConfig) -> Self {
        Self {
            dataset_id: value.data_id.clone(),
            data_id: value.data_id.clone(),
            exchange: value.exchange.clone(),
            symbol: value.symbol.clone(),
            market_type: value.market_type.clone(),
            kind: value.kind.clone(),
            interval: value.interval.clone(),
            lookback_days: value.days,
            enabled: value.enabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionAssumptionSpec {
    pub initial_cash_balance: f64,
    pub taker_fee_bps: f64,
    pub default_slippage_bps: f64,
    pub total_cost_buffer_bps: f64,
    pub time_in_force: TimeInForce,
    pub allow_partial_fills: bool,
    pub latency_assumption_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionAssumptionValueSource {
    BackendFallback,
    ProfileDefault,
    RequestOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionAssumptionSourceSummary {
    pub fee_bps: ExecutionAssumptionValueSource,
    pub slippage_bps: ExecutionAssumptionValueSource,
    pub latency_ms: ExecutionAssumptionValueSource,
}

impl From<&RuntimeProtocolCoreConfig> for ExecutionAssumptionSpec {
    fn from(value: &RuntimeProtocolCoreConfig) -> Self {
        Self {
            initial_cash_balance: value.initial_cash_balance,
            taker_fee_bps: value.taker_fee_bps,
            default_slippage_bps: value.default_slippage_bps,
            total_cost_buffer_bps: value.total_cost_buffer_bps,
            time_in_force: TimeInForce::Gtc,
            allow_partial_fills: true,
            latency_assumption_ms: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketDataSnapshotSpec {
    pub snapshot_id: String,
    pub replay_source: BacktestReplaySource,
    pub captured_at_ms: u64,
    #[serde(default)]
    pub datasets: Vec<DatasetSpec>,
}

impl MarketDataSnapshotSpec {
    pub fn from_runtime_protocol(
        snapshot_id: impl Into<String>,
        replay_source: BacktestReplaySource,
        captured_at_ms: u64,
        config: &RuntimeProtocolCoreConfig,
    ) -> Self {
        Self {
            snapshot_id: snapshot_id.into(),
            replay_source,
            captured_at_ms,
            datasets: config.data_sources.iter().map(DatasetSpec::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunSpec {
    pub schema_version: String,
    pub run_mode: RunModeSpec,
    pub graph_id: String,
    pub compile_id: String,
    pub runtime_mode: String,
    pub protocol_name: String,
    pub config_hash: String,
    #[serde(default)]
    pub datasets: Vec<DatasetSpec>,
    pub execution_assumptions: ExecutionAssumptionSpec,
    #[serde(default)]
    pub execution_assumption_sources: Option<ExecutionAssumptionSourceSummary>,
    pub core_ir_digest: ArtifactDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunSpecRuntimeProtocolInput {
    pub graph_id: String,
    pub compile_id: String,
    pub run_mode: RunModeSpec,
    pub runtime_mode: String,
    pub protocol_name: String,
    pub config_hash: String,
    pub core_ir_digest: ArtifactDigest,
}

impl RunSpec {
    pub fn from_runtime_protocol(
        input: RunSpecRuntimeProtocolInput,
        config: &RuntimeProtocolCoreConfig,
    ) -> Self {
        Self {
            schema_version: RUN_SPEC_V1_VERSION.to_string(),
            run_mode: input.run_mode,
            graph_id: input.graph_id,
            compile_id: input.compile_id,
            runtime_mode: input.runtime_mode,
            protocol_name: input.protocol_name,
            config_hash: input.config_hash,
            datasets: config.data_sources.iter().map(DatasetSpec::from).collect(),
            execution_assumptions: ExecutionAssumptionSpec::from(config),
            execution_assumption_sources: None,
            core_ir_digest: input.core_ir_digest,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BacktestSpec {
    pub schema_version: String,
    pub backtest_id: String,
    pub replay_source: BacktestReplaySource,
    pub requested_at_ms: u64,
    pub run_spec: RunSpec,
    pub market_data_snapshot: MarketDataSnapshotSpec,
}

impl BacktestSpec {
    pub fn new(
        backtest_id: impl Into<String>,
        replay_source: BacktestReplaySource,
        requested_at_ms: u64,
        run_spec: RunSpec,
        market_data_snapshot: MarketDataSnapshotSpec,
    ) -> Self {
        Self {
            schema_version: BACKTEST_SPEC_V1_VERSION.to_string(),
            backtest_id: backtest_id.into(),
            replay_source,
            requested_at_ms,
            run_spec,
            market_data_snapshot,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyArtifact {
    pub schema_version: String,
    pub artifact_id: String,
    pub graph_id: String,
    pub compile_id: String,
    pub strategy_id: String,
    pub name: String,
    pub source_kind: StrategyArtifactSourceKind,
    pub source_ref: String,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
    pub digest: ArtifactDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreIrArtifact {
    pub schema_version: String,
    pub artifact_id: String,
    pub graph_id: String,
    pub compile_id: String,
    pub ir_version: String,
    pub digest: ArtifactDigest,
    pub core_ir: CoreStrategyIr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileArtifact {
    pub schema_version: String,
    pub artifact_id: String,
    pub graph_id: String,
    pub compile_id: String,
    pub protocol_name: String,
    pub config_hash: String,
    pub strategy_artifact_id: String,
    pub core_ir_artifact_id: String,
    pub digest: ArtifactDigest,
    pub runtime_config: RuntimeProtocolCoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileArtifactBundle {
    pub strategy: StrategyArtifact,
    pub compile: CompileArtifact,
    pub core_ir: CoreIrArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawKline {
    pub open_time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub close_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawQuote {
    pub best_bid: f64,
    pub best_ask: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedKline {
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub market_type: MarketType,
    pub interval: String,
    pub open_time_ms: u64,
    pub close_time_ms: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineSeriesSnapshot {
    pub data_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub market_type: MarketType,
    pub interval: String,
    pub bars: Vec<NormalizedKline>,
    pub window_len: usize,
    pub ts_ms: u64,
    pub source_latency_ms: u64,
    pub source_status: SourceStatus,
    #[serde(default)]
    pub data_quality: DataQualitySnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSnapshot {
    pub data_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub market_type: MarketType,
    pub best_bid: f64,
    pub best_ask: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub mid_price: f64,
    pub ts_ms: u64,
    pub source_latency_ms: u64,
    pub source_status: SourceStatus,
    #[serde(default)]
    pub data_quality: DataQualitySnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizedMarketData {
    KlineSeries(KlineSeriesSnapshot),
    Quote(QuoteSnapshot),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSignal {
    pub signal_id: String,
    pub intent_id: String,
    pub kind: IntentKind,
    pub exchange_scope: Vec<Exchange>,
    pub symbol_scope: Vec<Symbol>,
    pub side: SignalSide,
    pub strength: f64,
    pub confidence: f64,
    pub reference_price: Option<f64>,
    pub derived_metrics: BTreeMap<String, f64>,
    pub reason: String,
    pub triggered_at_ms: u64,
    pub ttl_ms: u64,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedAction {
    /// Fraction of current portfolio equity to allocate as trade notional.
    pub exchange: Exchange,
    pub side: OrderSide,
    pub quantity_ratio: f64,
    pub reference_price: f64,
    pub strategy_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TargetWeight {
    pub exchange: Exchange,
    pub symbol: Symbol,
    /// Target portfolio weight for this basket member, expressed as a 0..1 ratio of equity.
    pub target_weight: f64,
    /// Observed current weight at decision time, expressed as a 0..1 ratio of equity.
    pub current_weight: f64,
    pub reference_price: f64,
    #[serde(default)]
    pub signal_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortfolioTarget {
    pub allocation_kind: String,
    #[serde(default)]
    pub target_weights: Vec<TargetWeight>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortfolioTargetDecision {
    pub target_id: String,
    pub target: PortfolioTarget,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDecision {
    pub decision_id: String,
    pub agent_id: String,
    pub symbol: Symbol,
    pub exchange_targets: Vec<Exchange>,
    pub net_side: SignalSide,
    pub net_strength: f64,
    #[serde(default)]
    pub portfolio_target_decision: Option<PortfolioTargetDecision>,
    pub proposed_actions: Vec<ProposedAction>,
    pub reason: String,
    pub produced_at_ms: u64,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDecision {
    pub risk_decision_id: String,
    pub risk_id: String,
    pub agent_decision_id: String,
    pub symbol: Symbol,
    pub status: DecisionStatus,
    #[serde(default)]
    pub mode: RiskDecisionMode,
    #[serde(default)]
    pub adjusted_portfolio_target_decision: Option<PortfolioTargetDecision>,
    pub adjusted_actions: Vec<ProposedAction>,
    pub reason_codes: Vec<RiskReasonCode>,
    pub reason_text: String,
    pub produced_at_ms: u64,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimOrder {
    pub order_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub limit_price: Option<f64>,
    pub time_in_force: TimeInForce,
    pub allow_partial: bool,
    pub reference_price: f64,
    pub slippage_bps: f64,
    pub fee_bps: f64,
    pub strategy_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub plan_id: String,
    pub source_risk_decision_id: String,
    pub orders: Vec<SimOrder>,
    pub created_at_ms: u64,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillReport {
    pub fill_id: String,
    pub plan_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub side: OrderSide,
    pub filled_qty: f64,
    pub filled_price: f64,
    pub fee_paid: f64,
    pub filled_at_ms: u64,
    pub status: ExecutionStatus,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenOrder {
    pub order_id: String,
    pub plan_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub remaining_qty: f64,
    pub reserved_cash: f64,
    pub reserved_qty: f64,
    pub limit_price: Option<f64>,
    pub reference_price: f64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillResult {
    pub plan_id: String,
    pub status: ExecutionStatus,
    pub fills: Vec<FillReport>,
    pub open_orders: Vec<OpenOrder>,
    pub events: Vec<RuntimeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub net_qty: f64,
    pub frozen_qty: f64,
    pub avg_entry_price: f64,
    pub mark_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeExposure {
    pub exchange: Exchange,
    pub gross_notional: f64,
    pub net_notional: f64,
    pub leverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioState {
    pub cash_balance: f64,
    pub available_cash_balance: f64,
    pub frozen_cash_balance: f64,
    pub open_orders: Vec<OpenOrder>,
    pub positions: Vec<Position>,
    pub exchange_exposures: Vec<ExchangeExposure>,
    pub total_gross_notional: f64,
    pub total_net_notional: f64,
    pub total_leverage: f64,
    pub updated_at_ms: u64,
}

impl PortfolioState {
    pub fn new(initial_cash_balance: f64, ts_ms: u64) -> Self {
        Self {
            cash_balance: initial_cash_balance,
            available_cash_balance: initial_cash_balance,
            frozen_cash_balance: 0.0,
            open_orders: Vec::new(),
            positions: Vec::new(),
            exchange_exposures: Vec::new(),
            total_gross_notional: 0.0,
            total_net_notional: 0.0,
            total_leverage: 0.0,
            updated_at_ms: ts_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub event_id: String,
    pub event_type: RuntimeEventType,
    pub trace_id: String,
    pub source_id: String,
    pub ts_ms: u64,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCycleOutput {
    pub cycle_name: String,
    pub trace_id: String,
    pub normalized_data: Vec<NormalizedMarketData>,
    pub intent_signals: Vec<IntentSignal>,
    pub agent_decisions: Vec<AgentDecision>,
    pub risk_decisions: Vec<RiskDecision>,
    pub execution_plans: Vec<ExecutionPlan>,
    pub fill_reports: Vec<FillReport>,
    pub portfolio_state: PortfolioState,
    pub runtime_events: Vec<RuntimeEvent>,
    pub data_fetch_counts: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOutput {
    pub slow_cycle: RuntimeCycleOutput,
    pub fast_cycle: RuntimeCycleOutput,
    pub final_portfolio: PortfolioState,
    pub data_fetch_counts: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestEquityPoint {
    pub ts_ms: u64,
    pub equity: f64,
    pub cash_balance: f64,
    pub net_notional: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestSummary {
    pub step_count: usize,
    pub trade_count: usize,
    pub total_return_ratio: f64,
    pub max_drawdown_ratio: f64,
    pub final_equity: f64,
    #[serde(default)]
    pub net_profit: f64,
    #[serde(default)]
    pub turnover_ratio: f64,
    #[serde(default)]
    pub average_trade_notional: f64,
    #[serde(default)]
    pub fee_drag_ratio: f64,
    #[serde(default)]
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestOutput {
    pub mode: String,
    pub started_at_ms: u64,
    pub ended_at_ms: u64,
    pub sessions: Vec<SessionOutput>,
    pub equity_curve: Vec<BacktestEquityPoint>,
    pub summary: BacktestSummary,
    pub final_portfolio: PortfolioState,
    #[serde(default)]
    pub debug_values: Option<Vec<std::collections::BTreeMap<String, f64>>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_runtime_protocol() -> RuntimeProtocolCoreConfig {
        RuntimeProtocolCoreConfig {
            data_sources: vec![DataSourceConfig {
                data_id: "binance_btc_1d".into(),
                exchange: Exchange::Binance,
                symbol: Symbol::BtcUsdt,
                market_type: MarketType::Spot,
                kind: DataKind::KlineSeries,
                days: Some(200),
                interval: Some("1d".into()),
                ping_enabled: false,
                request_interval_ms: None,
                enabled: true,
            }],
            intents: vec![IntentConfig {
                intent_id: "intent_rsi".into(),
                name: "RSI".into(),
                kind: IntentKind::Rsi,
                input_data_ids: vec!["binance_btc_1d".into()],
                params: BTreeMap::new(),
                enabled: true,
            }],
            agents: vec![AgentConfig {
                agent_id: "agent_main".into(),
                name: "Main Agent".into(),
                input_intent_ids: vec!["intent_rsi".into()],
                rebalance_symbols: vec![],
                rebalance_schedule: None,
                rebalance_allocation_kind: None,
                rebalance_rank_method: None,
                rebalance_score_normalize: None,
                rebalance_target_weights: vec![],
                params: BTreeMap::new(),
                enabled: true,
            }],
            risks: vec![RiskConfig {
                risk_id: "risk_main".into(),
                name: "Main Risk".into(),
                observed_agent_ids: vec!["agent_main".into()],
                max_position_ratio: 0.2,
                max_single_weight: None,
                max_concentration_ratio: None,
                max_symbol_net_exposure_ratio: None,
                max_portfolio_net_exposure_ratio: None,
                max_turnover: None,
                min_trade_weight: None,
                max_new_positions_per_rebalance: None,
                max_total_leverage: 3.0,
                max_exchange_leverage: 3.0,
                min_action_interval_ms: 100,
                enabled: true,
            }],
            initial_cash_balance: 100_000.0,
            taker_fee_bps: 10.0,
            default_slippage_bps: 5.0,
            total_cost_buffer_bps: 20.0,
        }
    }

    #[test]
    fn canonical_digest_is_stable_for_equivalent_payloads() {
        let left = serde_json::json!({
            "graph_id": "graph_test",
            "compile_id": "compile_test",
            "mode": "paper"
        });
        let right = serde_json::json!({
            "compile_id": "compile_test",
            "mode": "paper",
            "graph_id": "graph_test"
        });

        let left_digest = canonical_json_sha256_digest(&left).unwrap();
        let right_digest = canonical_json_sha256_digest(&right).unwrap();

        assert_eq!(left_digest, right_digest);
        assert_eq!(
            left_digest.algorithm,
            ArtifactDigestAlgorithm::Sha256CanonicalJson
        );
    }

    #[test]
    fn run_and_backtest_specs_capture_protocol_boundary() {
        let config = sample_runtime_protocol();
        let core_ir_digest = canonical_json_sha256_digest(&serde_json::json!({
            "ir_version": "quantpilot/core-ir/v1"
        }))
        .unwrap();

        let run_spec = RunSpec::from_runtime_protocol(
            RunSpecRuntimeProtocolInput {
                graph_id: "graph_test".to_string(),
                compile_id: "compile_test".to_string(),
                run_mode: RunModeSpec::Backtest,
                runtime_mode: "paper".to_string(),
                protocol_name: "quantpilot/minimal-sim/v1".to_string(),
                config_hash: "runtime-spec-hash".to_string(),
                core_ir_digest: core_ir_digest.clone(),
            },
            &config,
        );
        let snapshot = MarketDataSnapshotSpec::from_runtime_protocol(
            "snapshot_test",
            BacktestReplaySource::DeterministicMock,
            1_700_000_000_000,
            &config,
        );
        let backtest_spec = BacktestSpec::new(
            "backtest_test",
            BacktestReplaySource::DeterministicMock,
            1_700_000_000_000,
            run_spec.clone(),
            snapshot.clone(),
        );

        assert_eq!(run_spec.schema_version, RUN_SPEC_V1_VERSION);
        assert_eq!(run_spec.datasets.len(), 1);
        assert_eq!(
            run_spec.execution_assumptions.time_in_force,
            TimeInForce::Gtc
        );
        assert_eq!(snapshot.datasets[0].data_id, "binance_btc_1d");
        assert_eq!(backtest_spec.schema_version, BACKTEST_SPEC_V1_VERSION);
        assert_eq!(backtest_spec.run_spec.core_ir_digest, core_ir_digest);
        assert_eq!(backtest_spec.market_data_snapshot, snapshot);
    }
}
