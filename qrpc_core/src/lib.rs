pub mod error;
mod plugin;
mod protocol_primitives;
mod strategy_ir;

pub use plugin::*;
pub use protocol_primitives::*;
pub use qrpc_core_ir::{CoreStrategyIr, CORE_IR_V1_VERSION};
pub use strategy_ir::*;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DataQualitySnapshot {
    #[serde(default)]
    pub source_health: SourceHealth,
    #[serde(default)]
    pub freshness_ms: u64,
    #[serde(default)]
    pub stale_after_ms: u64,
    #[serde(default)]
    pub gap_count: u64, // v2.1.x: u32→u64 防截断
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

/// 计算规范 JSON 的 SHA-256 摘要。依赖默认 BTreeMap 键排序保证确定性。
///
/// **禁止**在依赖链中启用 `serde_json/features = ["preserve_order"]`，
/// 该 feature 会破坏快照签名和跨版本回测的可重现性。
/// 验证: `cargo tree -p serde_json -e features | grep preserve_order` 应返回空。
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
    #[serde(default)]
    pub slippage_bps: f64,
    #[serde(default)]
    pub fee_bps: f64,
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
#[serde(deny_unknown_fields)]
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

    /// v2.5.0: debug 模式下验证跨字段一致性
    pub fn debug_assert_invariants(&self) {
        if cfg!(debug_assertions) {
            // 可用现金 + 冻结现金 = 总现金
            debug_assert!(
                (self.available_cash_balance + self.frozen_cash_balance - self.cash_balance).abs()
                    < 0.01,
                "PortfolioState: available({}) + frozen({}) != cash({})",
                self.available_cash_balance,
                self.frozen_cash_balance,
                self.cash_balance
            );
            // 杠杆为非负
            debug_assert!(self.total_leverage >= 0.0, "total_leverage 不能为负");
            // 余额非负
            debug_assert!(self.cash_balance >= 0.0, "cash_balance 不能为负");
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

/// v1.1.0: 风险调整收益指标组
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacktestRiskAdjusted {
    #[serde(default)]
    pub sharpe_ratio: f64,
    #[serde(default)]
    pub sortino_ratio: f64,
    #[serde(default)]
    pub calmar_ratio: f64,
    /// v1.1.0 P2: 95% VaR (日收益率, 历史模拟法)
    #[serde(default)]
    pub var_95: f64,
    /// v1.1.0 P2: 95% CVaR / Expected Shortfall
    #[serde(default)]
    pub cvar_95: f64,
}

/// v1.1.0: 交易分析指标组
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacktestTradeAnalysis {
    #[serde(default)]
    pub profit_factor: f64,
    #[serde(default)]
    pub avg_win: f64,
    #[serde(default)]
    pub avg_loss: f64,
    #[serde(default)]
    pub max_consecutive_wins: u32,
    #[serde(default)]
    pub max_consecutive_losses: u32,
}

/// v1.1.0: 回撤分析指标组
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacktestDrawdownAnalysis {
    #[serde(default)]
    pub max_drawdown_ratio: f64,
    #[serde(default)]
    pub max_drawdown_duration_days: f64,
    #[serde(default)]
    pub avg_drawdown_duration_days: f64,
}

/// v1.1.0: 基准比较指标组 — None 表示基准未启用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestBenchmarkComparison {
    #[serde(default)]
    pub benchmark_total_return: f64,
    #[serde(default)]
    pub alpha: f64,
    #[serde(default)]
    pub beta: f64,
    #[serde(default)]
    pub information_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestSummary {
    pub step_count: usize,
    pub trade_count: usize,
    pub total_return_ratio: f64,
    pub final_equity: f64,
    #[serde(default)]
    pub net_profit: f64,
    #[serde(default)]
    pub win_rate: f64,
    #[serde(default)]
    pub annualized_return: f64,
    #[serde(default)]
    pub annualized_volatility: f64,
    #[serde(default)]
    pub risk_adjusted: BacktestRiskAdjusted,
    #[serde(default)]
    pub trade_analysis: BacktestTradeAnalysis,
    #[serde(default)]
    pub drawdown_analysis: BacktestDrawdownAnalysis,
    #[serde(default)]
    pub benchmark_comparison: Option<BacktestBenchmarkComparison>,
    /// v1.1.0 P2: 日收益率偏度 (正偏 = 多小亏少大赚)
    #[serde(default)]
    pub skewness: f64,
    /// v1.1.0 P2: 日收益率峰度 (高值 = 肥尾风险)
    #[serde(default)]
    pub kurtosis: f64,
}

/// v1.1.0 P2: 月度/季度收益率分解
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodReturn {
    pub period: String, // e.g. "2025-01", "2025-Q1", "2025"
    pub return_ratio: f64,
    pub trade_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestOutput {
    pub mode: String,
    pub started_at_ms: u64,
    pub ended_at_ms: u64,
    /// v1.3.4: 回测实际墙钟耗时（毫秒）
    #[serde(default)]
    pub elapsed_ms: Option<u64>,
    pub sessions: Vec<SessionOutput>,
    pub equity_curve: Vec<BacktestEquityPoint>,
    /// v1.1.0: 买入持有基准权益曲线（等权重持有全部交易标的）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub benchmark_equity_curve: Vec<BacktestEquityPoint>,
    /// v1.1.0 P2: 月度/季度/年度收益率分解
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub period_returns: Vec<PeriodReturn>,
    pub summary: BacktestSummary,
    pub final_portfolio: PortfolioState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v4_artifact: Option<qrpc_core_ir::v4::V4BacktestArtifact>,
    #[serde(default)]
    pub debug_values: Option<Vec<std::collections::BTreeMap<String, f64>>>,
}

// ── v1.0.0 RFC-001 数据请求协议 ──────────────────────

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

// ── v1.0.0 RFC-010 分配协议 ──────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AllocationMethod {
    EqualWeight,
    FixedWeight,
    RankWeight,
    ScoreWeight,
    RiskParity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Allocation {
    pub allocation_id: String,
    pub method: AllocationMethod,
    pub weights: BTreeMap<Symbol, f64>,
    pub total_budget: f64,
    pub min_weight: Option<f64>,
    pub max_weight: Option<f64>,
    pub constraint_source: Option<String>,
    pub created_at_ms: u64,
}

impl Allocation {
    /// v1.0.0: 对一组目标仓位按权重分配资金
    pub fn apply_to_targets(&self, targets: &[PortfolioTarget]) -> BTreeMap<Symbol, f64> {
        let mut allocated: BTreeMap<Symbol, f64> = BTreeMap::new();
        for target in targets {
            for tw in &target.target_weights {
                let weight = self
                    .weights
                    .get(&tw.symbol)
                    .copied()
                    .unwrap_or(tw.target_weight);
                let amount = self.total_budget * weight;
                let clamped = match (self.min_weight, self.max_weight) {
                    (Some(min), Some(max)) => amount
                        .max(min * self.total_budget)
                        .min(max * self.total_budget),
                    (Some(min), None) => amount.max(min * self.total_budget),
                    (None, Some(max)) => amount.min(max * self.total_budget),
                    (None, None) => amount,
                };
                allocated.insert(tw.symbol.clone(), clamped);
            }
        }
        allocated
    }
}

// ── v1.0.0 RFC-012 订单协议 ──────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Created,
    Submitted,
    Accepted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

impl OrderStatus {
    /// v1.0.1: 校验状态转移合法性。终态不可再转移。
    pub fn can_transition_to(&self, next: &OrderStatus) -> bool {
        // 终态不可转移
        if matches!(
            self,
            Self::Filled | Self::Cancelled | Self::Rejected | Self::Expired
        ) {
            return false;
        }
        matches!(
            (self, next),
            (Self::Created, Self::Submitted)
                | (Self::Created, Self::Cancelled)
                | (Self::Created, Self::Rejected)
                | (Self::Submitted, Self::Accepted)
                | (Self::Submitted, Self::Rejected)
                | (Self::Accepted, Self::PartiallyFilled)
                | (Self::Accepted, Self::Cancelled)
                | (Self::Accepted, Self::Rejected)
                | (Self::Accepted, Self::Expired)
                | (Self::PartiallyFilled, Self::PartiallyFilled)
                | (Self::PartiallyFilled, Self::Filled)
                | (Self::PartiallyFilled, Self::Cancelled)
                | (Self::PartiallyFilled, Self::Rejected)
                | (Self::PartiallyFilled, Self::Expired)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub exchange: Exchange,
    pub instrument: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Option<f64>,
    pub quantity: f64,
    pub executed_qty: f64,
    pub time_in_force: TimeInForce,
    pub status: OrderStatus,
    pub source_intent_id: Option<String>,
    pub source_agent_id: Option<String>,
    pub venue_order_id: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

// ── v1.0.0 RFC-013 执行反馈协议 ──────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeedbackKind {
    OrderSubmitted,
    OrderRejected,
    OrderPartiallyFilled,
    OrderFilled,
    OrderCancelled,
    OrderExpired,
    VenueError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionFeedback {
    pub feedback_id: String,
    pub order_id: String,
    pub kind: FeedbackKind,
    pub fill_qty: Option<f64>,
    pub fill_price: Option<f64>,
    pub remaining_qty: Option<f64>,
    pub reject_reason: Option<String>,
    pub venue_message: Option<String>,
    pub occurred_at_ms: u64,
    pub ingested_at_ms: u64,
}

// ── v1.0.0 热接管协议 (RFC 暂未独立编号, Phase 3) ──

/// 策略 A → Sandbox → 策略 B 的状态快照
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HandoffSnapshot {
    pub snapshot_id: String,
    pub source_strategy_id: String,
    pub positions: BTreeMap<Symbol, f64>,
    pub open_orders: Vec<Order>,
    pub cash_balance: f64,
    pub available_cash_balance: f64,
    pub frozen_cash_balance: f64,
    pub current_cycle: u64,
    pub snapshot_at_ms: u64,
}

impl HandoffSnapshot {
    /// 校验快照完整性 — 未结订单必须携带 order_id
    pub fn validate_completeness(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.snapshot_id.is_empty() {
            errors.push("snapshot_id 不能为空".to_string());
        }
        if self.source_strategy_id.is_empty() {
            errors.push("source_strategy_id 不能为空".to_string());
        }
        for order in &self.open_orders {
            if order.order_id.is_empty() {
                errors.push(format!(
                    "未结订单缺少 order_id (symbol={:?})",
                    order.instrument
                ));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
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
