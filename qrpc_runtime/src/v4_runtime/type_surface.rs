use super::{
    MachineRuntimeState, V4ExecutionRuntimeState, V4RiskPlaneRuntimeState,
    V4SimulatedExecutionRuntimeState,
};
use qrpc_core_ir::v4::{
    CapabilitySupportSource, ComplexityMetrics, ExecutionCapabilityKind,
    RuntimeSettlementAuthority, RuntimeTradingMode, V4MachineGraphContract,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, VecDeque};

pub const V4_DEFAULT_MARKET_DATA_SOURCE: &str = "market.data";
pub const EVENT_EXECUTION_ORDER_ACKNOWLEDGED: &str = "execution_order_acknowledged";
pub const EVENT_EXECUTION_ORDER_REJECTED: &str = "execution_order_rejected";
pub const EVENT_EXECUTION_ORDER_CANCELED: &str = "execution_order_canceled";
pub const EVENT_EXECUTION_ORDER_EXPIRED: &str = "execution_order_expired";
pub const EVENT_EXECUTION_ORDER_AMENDED: &str = "execution_order_amended";
pub const EVENT_EXECUTION_ORDER_PARTIALLY_FILLED: &str = "execution_order_partially_filled";
pub const EVENT_EXECUTION_ORDER_FILLED: &str = "execution_order_filled";
pub const EVENT_EXECUTION_FEE_CHARGED: &str = "execution_fee_charged";
pub const EVENT_EXECUTION_PORTFOLIO_CHANGED: &str = "execution_portfolio_changed";
pub const EVENT_EXECUTION_CONDITIONAL_ORDER_TRIGGERED: &str =
    "execution_conditional_order_triggered";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct V4RuntimeInputEvent {
    pub event_type: String,
    pub source: String,
    #[serde(default)]
    pub payload: Value,
    pub ts_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4BacktestBarInput {
    pub venue_id: String,
    pub symbol: String,
    pub close: f64,
    pub ts_ms: u64,
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4BacktestTickInput {
    pub venue_id: String,
    pub symbol: String,
    pub price: f64,
    pub size: f64,
    pub ts_ms: u64,
    #[serde(default)]
    pub sequence: u64,
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4RuntimeEventEnvelope {
    pub sequence: u64,
    pub event_type: String,
    pub source: String,
    #[serde(default)]
    pub origin: V4RuntimeEventOrigin,
    pub ts_ms: u64,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub replayable: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4RuntimeEventOrigin {
    ExternalInput,
    MachineEmit,
    RuntimeControl,
}

impl Default for V4RuntimeEventOrigin {
    fn default() -> Self {
        Self::ExternalInput
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4PaperSimulatedRunOutput {
    pub runtime_mode: RuntimeTradingMode,
    #[serde(default)]
    pub events: Vec<V4RuntimeEventEnvelope>,
    pub memory_snapshot: V4RuntimeMemorySnapshot,
    pub provider_order_submission_attached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4RuntimeMemorySnapshot {
    pub graph_id: String,
    pub runtime_mode: RuntimeTradingMode,
    pub ts_ms: u64,
    #[serde(default)]
    pub machines: Vec<V4MachineRuntimeSnapshot>,
    pub risk_plane: V4RiskPlaneRuntimeSnapshot,
    pub execution: V4ExecutionRuntimeSnapshot,
    pub simulated_execution: V4SimulatedExecutionSnapshot,
    pub venue_adapter_boundary: V4VenueAdapterRuntimeBoundary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complexity_metrics: Option<ComplexityMetrics>,
    pub event_sequence: u64,
    pub provider_order_submission_attached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4MachineRuntimeSnapshot {
    pub machine_id: String,
    pub template: qrpc_core_ir::v4::MachineTemplateKind,
    pub state_id: String,
    pub status: V4MachineRuntimeStatus,
    #[serde(default)]
    pub memory: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_output: Option<V4CachedMachineOutput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_pulled_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<V4MachineRuntimeSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4CachedMachineOutput {
    pub machine_id: String,
    pub state_id: String,
    pub event_type: String,
    #[serde(default)]
    pub emitted_events: Vec<String>,
    #[serde(default)]
    pub payload: Value,
    pub updated_at_ms: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4MachineRuntimeStatus {
    Active,
    SoftSilent,
    Recovering,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4RiskPlaneRuntimeSnapshot {
    pub required: bool,
    #[serde(default)]
    pub machine_ids: Vec<String>,
    pub min_priority: i32,
    pub approved_event_count: u64,
    pub rejected_event_count: u64,
    pub real_order_path_unlocked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_decision: Option<V4RiskPlaneRuntimeDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4RiskPlaneRuntimeDecision {
    pub decision_id: String,
    pub target_machine_id: String,
    pub source_machine_id: String,
    pub event_type: String,
    pub approved: bool,
    pub reason: String,
    pub ts_ms: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4ExecutionRuntimeSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub venue_id: Option<String>,
    #[serde(default)]
    pub required_capabilities: Vec<ExecutionCapabilityKind>,
    pub accepted_count: u64,
    pub rejected_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_decision: Option<V4ExecutionRuntimeDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4ExecutionRuntimeDecision {
    pub decision_id: String,
    pub target_machine_id: String,
    pub venue_id: String,
    pub runtime_mode: RuntimeTradingMode,
    pub accepted: bool,
    pub reason: String,
    #[serde(default)]
    pub entries: Vec<V4ExecutionCapabilityRuntimeEntry>,
    pub provider_order_submission_attached: bool,
    pub ts_ms: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4ExecutionCapabilityRuntimeEntry {
    pub capability: ExecutionCapabilityKind,
    pub source: CapabilitySupportSource,
    pub status: V4ExecutionCapabilityRuntimeStatus,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4ExecutionCapabilityRuntimeStatus {
    Accepted,
    Unsupported,
    NotDeclared,
    ModeRejected,
    PolicyMissing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4SimulatedExecutionConfig {
    pub starting_cash: f64,
    pub quote_asset: String,
    pub default_venue_id: String,
    pub default_symbol: String,
    pub default_quantity: f64,
    pub default_price: f64,
    pub default_fee_bps: f64,
    pub default_slippage_bps: f64,
    #[serde(default)]
    pub allow_partial_fill: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_fill_quantity: Option<f64>,
}

impl Default for V4SimulatedExecutionConfig {
    fn default() -> Self {
        Self {
            starting_cash: 100_000.0,
            quote_asset: "USD".to_string(),
            default_venue_id: "paper-local".to_string(),
            default_symbol: "BTCUSDT".to_string(),
            default_quantity: 1.0,
            default_price: 100.0,
            default_fee_bps: 10.0,
            default_slippage_bps: 0.0,
            allow_partial_fill: true,
            max_fill_quantity: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4SimulatedOrderRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    pub venue_id: String,
    pub symbol: String,
    pub action: V4SimulatedPositionAction,
    pub order_type: V4SimulatedOrderType,
    pub quantity: f64,
    pub reference_price: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub take_profit_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_loss_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trailing_offset_bps: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expire_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<V4SimulatedTimeInForce>,
    #[serde(default)]
    pub post_only: bool,
    #[serde(default)]
    pub reduce_only: bool,
    #[serde(default)]
    pub close_only: bool,
    #[serde(default)]
    pub allow_partial_fill: bool,
    pub fee_bps: f64,
    pub slippage_bps: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_fill_quantity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4SimulatedOrder {
    pub order_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    pub venue_id: String,
    pub symbol: String,
    pub action: V4SimulatedPositionAction,
    pub side: V4SimulatedOrderSide,
    pub order_type: V4SimulatedOrderType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<V4SimulatedTimeInForce>,
    pub requested_quantity: f64,
    pub filled_quantity: f64,
    pub remaining_quantity: f64,
    pub reference_price: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub take_profit_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_loss_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trailing_offset_bps: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expire_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oco_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trailing_peak_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trailing_trough_price: Option<f64>,
    #[serde(default)]
    pub amend_revision: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_price: Option<f64>,
    pub status: V4SimulatedOrderStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
    pub fee_bps: f64,
    pub slippage_bps: f64,
    pub ts_ms: u64,
    pub source_event_sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4SimulatedFill {
    pub fill_id: String,
    pub order_id: String,
    pub venue_id: String,
    pub symbol: String,
    pub side: V4SimulatedOrderSide,
    pub action: V4SimulatedPositionAction,
    pub quantity: f64,
    pub price: f64,
    pub notional: f64,
    pub fee: f64,
    pub fee_asset: String,
    pub ts_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4SimulatedPosition {
    pub venue_id: String,
    pub symbol: String,
    pub net_quantity: f64,
    pub average_price: f64,
    pub market_price: f64,
    pub market_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4SimulatedAssetPoint {
    pub ts_ms: u64,
    pub cash_balance: f64,
    pub position_market_value: f64,
    pub portfolio_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4SimulatedExecutionSnapshot {
    pub enabled: bool,
    pub quote_asset: String,
    pub cash_balance: f64,
    pub realized_fees: f64,
    pub position_market_value: f64,
    pub portfolio_value: f64,
    pub order_count: u64,
    pub open_order_count: u64,
    pub rejected_order_count: u64,
    pub fill_count: u64,
    #[serde(default)]
    pub positions: Vec<V4SimulatedPosition>,
    #[serde(default)]
    pub asset_curve: Vec<V4SimulatedAssetPoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_order: Option<V4SimulatedOrder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_fill: Option<V4SimulatedFill>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4VenueAdapterRuntimeBoundary {
    pub provider_order_submission_attached: bool,
    pub provider_order_submission_allowed: bool,
    pub settlement_authority: RuntimeSettlementAuthority,
    pub live_actual_submission_allowed: bool,
    pub rejection_before_provider_submit: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum V4SimulatedPositionAction {
    Buy,
    Sell,
    OpenLong,
    CloseLong,
    OpenShort,
    CloseShort,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4SimulatedOrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum V4SimulatedOrderType {
    Market,
    Limit,
    StopMarket,
    StopLimit,
    TakeProfitMarket,
    TakeProfitLimit,
    OcoBracket,
    TrailingStop,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum V4SimulatedTimeInForce {
    Gtc,
    Ioc,
    Fok,
    Day,
    Gtd,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4SimulatedOrderStatus {
    Accepted,
    Rejected,
    PartiallyFilled,
    Filled,
    Canceled,
    Expired,
}

#[derive(Debug, Clone)]
pub struct V4PaperSimulatedRuntime {
    pub(super) graph: V4MachineGraphContract,
    pub(super) runtime_mode: RuntimeTradingMode,
    pub(super) machines: BTreeMap<String, MachineRuntimeState>,
    pub(super) risk_plane: V4RiskPlaneRuntimeState,
    pub(super) execution: V4ExecutionRuntimeState,
    pub(super) simulated_execution: V4SimulatedExecutionRuntimeState,
    pub(super) event_queue: VecDeque<V4RuntimeEventEnvelope>,
    pub(super) event_log: Vec<V4RuntimeEventEnvelope>,
    pub(super) sequence: u64,
    pub(super) provider_order_submission_attached: bool,
}

pub type V4Runtime = V4PaperSimulatedRuntime;
