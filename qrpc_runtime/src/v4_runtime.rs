use crate::backtest_metrics::{
    compute_microstructure_metrics, MicrostructureFillSample, MicrostructureOrderSample,
};
use anyhow::{anyhow, Result};
use qrpc_core_ir::v4::{
    default_v4_runtime_mode_contract, CapabilitySupportSource, ComplexityMetrics,
    EventFreshnessRequirement, ExecutionCapabilityKind, MachineCachePolicy,
    MachineEventPayloadField, MachineEventSourceKind, MachineGraphEdge, MachineRecoveryPolicy,
    MachineSilencePolicy, MachineTemplateKind, MachineTransition, RuntimeSettlementAuthority,
    RuntimeTradingMode, V4BacktestArtifact, V4BacktestExecutionCapabilitySourceRecord,
    V4BacktestMachineTrajectoryPoint, V4BacktestRiskPlaneDecisionRecord, V4MachineContract,
    V4MachineGraphContract, VenueCapabilityMatrix, V4_BACKTEST_ARTIFACT_VERSION,
    V4_RUNTIME_EVENT_REJECTED_EVENT,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const V4_RUNTIME_MAX_EVENT_STEPS: usize = 1_024;
const EVENT_DOWNSTREAM_PULL: &str = "downstream_pull";
const EVENT_SILENCE_ENTERED: &str = "silence_entered";
const EVENT_SILENCE_EXITED: &str = "silence_exited";
const EVENT_CACHE_RETURNED: &str = "cache_returned";
const EVENT_RECOVERY_STARTED: &str = "recovery_started";
const EVENT_RECOVERY_COMPLETED: &str = "recovery_completed";
const EVENT_TRANSITION_APPLIED: &str = "machine_transition_applied";
const EVENT_RISK_PLANE_APPROVED: &str = "risk_plane_approved";
const EVENT_RISK_PLANE_REJECTED: &str = "risk_plane_rejected";
const EVENT_EXECUTION_CAPABILITY_ACCEPTED: &str = "execution_capability_accepted";
const EVENT_EXECUTION_CAPABILITY_REJECTED: &str = "execution_capability_rejected";
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
    pub template: MachineTemplateKind,
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
struct MachineRuntimeState {
    state_id: String,
    status: V4MachineRuntimeStatus,
    memory: BTreeMap<String, Value>,
    cached_output: Option<V4CachedMachineOutput>,
    last_pulled_at_ms: Option<u64>,
    last_event_at_ms: Option<u64>,
    initialized_at_ms: u64,
}

#[derive(Debug, Clone)]
struct RuntimeTransitionCandidate {
    priority: i32,
    sort_id: String,
    machine_id: String,
    transition: MachineTransition,
}

fn initialize_machine_family_state(
    machine: &V4MachineContract,
    machines: &mut BTreeMap<String, MachineRuntimeState>,
) -> Result<()> {
    let initial_state = machine
        .states
        .iter()
        .find(|state| state.initial)
        .ok_or_else(|| anyhow!("machine `{}` 缺少初始状态", machine.machine_id))?;
    let memory = machine
        .memory
        .iter()
        .map(|field| {
            (
                field.name.clone(),
                field.default_value.clone().unwrap_or(Value::Null),
            )
        })
        .collect();
    machines.insert(
        machine.machine_id.clone(),
        MachineRuntimeState {
            state_id: initial_state.state_id.clone(),
            status: V4MachineRuntimeStatus::Active,
            memory,
            cached_output: None,
            last_pulled_at_ms: None,
            last_event_at_ms: None,
            initialized_at_ms: 0,
        },
    );

    for state in &machine.states {
        if let Some(child_machine) = state.child_machine.as_deref() {
            initialize_machine_family_state(child_machine, machines)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct V4RiskPlaneRuntimeState {
    required: bool,
    machine_ids: BTreeSet<String>,
    min_priority: i32,
    approved_event_count: u64,
    rejected_event_count: u64,
    last_decision: Option<V4RiskPlaneRuntimeDecision>,
}

#[derive(Debug, Clone)]
struct V4ExecutionCapabilityRuntimePolicy {
    venue_matrix: VenueCapabilityMatrix,
    required_capabilities: Vec<ExecutionCapabilityKind>,
}

#[derive(Debug, Clone)]
struct V4ExecutionRuntimeState {
    capability_policy: Option<V4ExecutionCapabilityRuntimePolicy>,
    accepted_count: u64,
    rejected_count: u64,
    last_decision: Option<V4ExecutionRuntimeDecision>,
}

#[derive(Debug, Clone)]
struct V4SimulatedExecutionRuntimeState {
    config: V4SimulatedExecutionConfig,
    cash_balance: f64,
    realized_fees: f64,
    order_sequence: u64,
    rejected_order_count: u64,
    positions: BTreeMap<(String, String), V4SimulatedPosition>,
    orders: Vec<V4SimulatedOrder>,
    fills: Vec<V4SimulatedFill>,
    asset_curve: Vec<V4SimulatedAssetPoint>,
    market_prices: BTreeMap<(String, String), f64>,
}

#[derive(Debug, Clone)]
struct V4SimulatedExecutionOutcome {
    events: Vec<(&'static str, Value)>,
}

pub fn expand_v4_graph_for_symbols(
    graph: &V4MachineGraphContract,
    symbols: &[String],
) -> Result<V4MachineGraphContract> {
    let normalized_symbols = normalize_v4_backtest_symbols(symbols);
    if normalized_symbols.len() <= 1 {
        let mut single = graph.clone();
        single.metadata.insert(
            "symbols".to_string(),
            json!(normalized_symbols
                .first()
                .cloned()
                .into_iter()
                .collect::<Vec<_>>()),
        );
        return Ok(single);
    }

    let machine_ids = collect_v4_machine_family_ids(&graph.machines);
    let mut expanded = graph.clone();
    expanded.graph_id = format!(
        "{}::universe_{}",
        graph.graph_id,
        normalized_symbols
            .iter()
            .map(|symbol| sanitize_v4_symbol_for_id(symbol))
            .collect::<Vec<_>>()
            .join("_")
    );
    expanded
        .metadata
        .insert("symbols".to_string(), json!(normalized_symbols.clone()));
    expanded
        .metadata
        .insert("universe_expanded".to_string(), Value::Bool(true));

    expanded.machines.clear();
    for symbol in &normalized_symbols {
        for machine in &graph.machines {
            let mut cloned = machine.clone();
            let original_machine_id = machine.machine_id.clone();
            prefix_v4_machine_family_for_symbol(&mut cloned, symbol, &machine_ids);
            cloned
                .metadata
                .insert("symbol".to_string(), Value::String(symbol.clone()));
            cloned.metadata.insert(
                "base_machine_id".to_string(),
                Value::String(original_machine_id.clone()),
            );
            expanded.machines.push(cloned);
        }
    }

    expanded.edges.clear();
    for symbol in &normalized_symbols {
        for edge in &graph.edges {
            expanded.edges.push(MachineGraphEdge {
                edge_id: expanded_v4_machine_id(symbol, &edge.edge_id),
                source_machine_id: expanded_v4_machine_id(symbol, &edge.source_machine_id),
                target_machine_id: expanded_v4_machine_id(symbol, &edge.target_machine_id),
                event_type: edge.event_type.clone(),
                activation: edge.activation.clone(),
                required: edge.required,
                metadata: {
                    let mut metadata = edge.metadata.clone();
                    metadata.insert("symbol".to_string(), Value::String(symbol.clone()));
                    metadata
                },
            });
        }
    }

    if let Some(catalog) = expanded.event_catalog.as_mut() {
        for event in &mut catalog.events {
            event.allowed_emitters = expand_v4_event_party_list(
                &event.allowed_emitters,
                &machine_ids,
                &normalized_symbols,
            );
            event.allowed_consumers = expand_v4_event_party_list(
                &event.allowed_consumers,
                &machine_ids,
                &normalized_symbols,
            );
        }
        catalog.metadata.insert(
            "universe_symbols".to_string(),
            json!(normalized_symbols.clone()),
        );
    }

    if let Some(risk_plane) = expanded.risk_plane.as_mut() {
        risk_plane.machine_ids = graph
            .risk_plane
            .as_ref()
            .map(|plane| {
                normalized_symbols
                    .iter()
                    .flat_map(|symbol| {
                        plane
                            .machine_ids
                            .iter()
                            .map(move |machine_id| expanded_v4_machine_id(symbol, machine_id))
                    })
                    .collect()
            })
            .unwrap_or_default();
    }

    expanded.validate_static_contract().map_err(|errors| {
        anyhow!(
            "expanded v4 multi-symbol graph failed static validation: {:?}",
            errors
        )
    })?;
    Ok(expanded)
}

pub fn normalize_v4_backtest_symbols(symbols: &[String]) -> Vec<String> {
    let mut normalized = symbols
        .iter()
        .map(|symbol| symbol.trim().to_ascii_uppercase())
        .filter(|symbol| !symbol.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        normalized.push("BTCUSDT".to_string());
    }
    normalized
}

fn expanded_v4_machine_id(symbol: &str, base: &str) -> String {
    format!("{}::{}", sanitize_v4_symbol_for_id(symbol), base)
}

fn collect_v4_machine_family_ids(machines: &[V4MachineContract]) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for machine in machines {
        collect_v4_machine_family_ids_inner(machine, &mut ids);
    }
    ids
}

fn collect_v4_machine_family_ids_inner(machine: &V4MachineContract, ids: &mut BTreeSet<String>) {
    ids.insert(machine.machine_id.clone());
    for state in &machine.states {
        if let Some(child_machine) = state.child_machine.as_deref() {
            collect_v4_machine_family_ids_inner(child_machine, ids);
        }
    }
}

fn prefix_v4_machine_family_for_symbol(
    machine: &mut V4MachineContract,
    symbol: &str,
    machine_ids: &BTreeSet<String>,
) {
    machine.machine_id = expanded_v4_machine_id(symbol, &machine.machine_id);
    for transition in &mut machine.transitions {
        transition.transition_id = expanded_v4_machine_id(symbol, &transition.transition_id);
        if let Some(source) = transition.event.source.as_mut() {
            if machine_ids.contains(source.as_str()) {
                *source = expanded_v4_machine_id(symbol, source);
            }
        }
    }
    for state in &mut machine.states {
        if let Some(child_machine) = state.child_machine.as_deref_mut() {
            prefix_v4_machine_family_for_symbol(child_machine, symbol, machine_ids);
        }
    }
}

fn sanitize_v4_symbol_for_id(symbol: &str) -> String {
    symbol
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn expand_v4_event_party_list(
    parties: &[String],
    machine_ids: &BTreeSet<String>,
    symbols: &[String],
) -> Vec<String> {
    if parties.is_empty() {
        return Vec::new();
    }
    let mut expanded = BTreeSet::new();
    for party in parties {
        if machine_ids.contains(party.as_str()) {
            for symbol in symbols {
                expanded.insert(expanded_v4_machine_id(symbol, party));
            }
        } else {
            expanded.insert(party.clone());
        }
    }
    expanded.into_iter().collect()
}

fn symbol_for_machine_id(machine_id: &str) -> Option<String> {
    machine_id
        .split_once("::")
        .map(|(symbol, _)| symbol.to_ascii_uppercase())
}

fn v4_machine_status_label(status: V4MachineRuntimeStatus) -> &'static str {
    match status {
        V4MachineRuntimeStatus::Active => "active",
        V4MachineRuntimeStatus::SoftSilent => "soft_silent",
        V4MachineRuntimeStatus::Recovering => "recovering",
    }
}

fn v4_execution_capability_status_label(
    status: V4ExecutionCapabilityRuntimeStatus,
) -> &'static str {
    match status {
        V4ExecutionCapabilityRuntimeStatus::Accepted => "accepted",
        V4ExecutionCapabilityRuntimeStatus::Unsupported => "unsupported",
        V4ExecutionCapabilityRuntimeStatus::NotDeclared => "not_declared",
        V4ExecutionCapabilityRuntimeStatus::ModeRejected => "mode_rejected",
        V4ExecutionCapabilityRuntimeStatus::PolicyMissing => "policy_missing",
    }
}

#[derive(Debug, Clone)]
pub struct V4PaperSimulatedRuntime {
    graph: V4MachineGraphContract,
    runtime_mode: RuntimeTradingMode,
    machines: BTreeMap<String, MachineRuntimeState>,
    risk_plane: V4RiskPlaneRuntimeState,
    execution: V4ExecutionRuntimeState,
    simulated_execution: V4SimulatedExecutionRuntimeState,
    event_queue: VecDeque<V4RuntimeEventEnvelope>,
    event_log: Vec<V4RuntimeEventEnvelope>,
    sequence: u64,
    provider_order_submission_attached: bool,
}

pub type V4Runtime = V4PaperSimulatedRuntime;

impl V4PaperSimulatedRuntime {
    pub fn new(graph: V4MachineGraphContract) -> Result<Self> {
        Self::new_for_mode(graph, RuntimeTradingMode::PaperSimulated)
    }

    pub fn new_for_mode(
        graph: V4MachineGraphContract,
        runtime_mode: RuntimeTradingMode,
    ) -> Result<Self> {
        if !matches!(
            runtime_mode,
            RuntimeTradingMode::PaperSimulated | RuntimeTradingMode::LiveActual
        ) {
            return Err(anyhow!(
                "v4 Phase 5 runtime 只允许 PaperSimulated 模式，实际收到 {:?}",
                runtime_mode
            ));
        }
        if runtime_mode == RuntimeTradingMode::LiveActual
            && !graph
                .risk_plane
                .as_ref()
                .map(|plane| plane.required)
                .unwrap_or(false)
        {
            return Err(anyhow!(
                "v4 LiveActual runtime requires an explicit required Risk Plane"
            ));
        }
        graph.validate_static_contract().map_err(|errors| {
            anyhow!(
                "v4 machine graph 在进入 PaperSimulated runtime 前未通过静态契约: {:?}",
                errors
            )
        })?;

        let mut machines = BTreeMap::new();
        for machine in &graph.machines {
            initialize_machine_family_state(machine, &mut machines)?;
        }
        let risk_plane = graph
            .risk_plane
            .as_ref()
            .map(|risk_plane| V4RiskPlaneRuntimeState {
                required: risk_plane.required,
                machine_ids: risk_plane.machine_ids.iter().cloned().collect(),
                min_priority: risk_plane.min_priority,
                approved_event_count: 0,
                rejected_event_count: 0,
                last_decision: None,
            })
            .unwrap_or_else(|| V4RiskPlaneRuntimeState {
                required: false,
                machine_ids: BTreeSet::new(),
                min_priority: 0,
                approved_event_count: 0,
                rejected_event_count: 0,
                last_decision: None,
            });

        Ok(Self {
            graph,
            runtime_mode,
            machines,
            risk_plane,
            execution: V4ExecutionRuntimeState {
                capability_policy: None,
                accepted_count: 0,
                rejected_count: 0,
                last_decision: None,
            },
            simulated_execution: V4SimulatedExecutionRuntimeState::new(
                V4SimulatedExecutionConfig::default(),
                0,
            ),
            event_queue: VecDeque::new(),
            event_log: Vec::new(),
            sequence: 0,
            provider_order_submission_attached: false,
        })
    }

    pub fn new_with_execution_capabilities(
        graph: V4MachineGraphContract,
        venue_matrix: VenueCapabilityMatrix,
        required_capabilities: Vec<ExecutionCapabilityKind>,
    ) -> Result<Self> {
        Self::new(graph)?.with_execution_capabilities(venue_matrix, required_capabilities)
    }

    pub fn new_for_mode_with_execution_capabilities(
        graph: V4MachineGraphContract,
        runtime_mode: RuntimeTradingMode,
        venue_matrix: VenueCapabilityMatrix,
        required_capabilities: Vec<ExecutionCapabilityKind>,
    ) -> Result<Self> {
        Self::new_for_mode(graph, runtime_mode)?
            .with_execution_capabilities(venue_matrix, required_capabilities)
    }

    pub fn new_for_backtest(
        graph: V4MachineGraphContract,
        venue_matrix: VenueCapabilityMatrix,
        required_capabilities: Vec<ExecutionCapabilityKind>,
    ) -> Result<Self> {
        Self::new_with_execution_capabilities(graph, venue_matrix, required_capabilities)
    }

    pub fn with_execution_capabilities(
        mut self,
        venue_matrix: VenueCapabilityMatrix,
        required_capabilities: Vec<ExecutionCapabilityKind>,
    ) -> Result<Self> {
        venue_matrix
            .validate_required_capability_sources(&required_capabilities)
            .map_err(|errors| {
                anyhow!(
                    "v4 execution capability policy 未通过静态契约: {:?}",
                    errors
                )
            })?;
        self.execution.capability_policy = Some(V4ExecutionCapabilityRuntimePolicy {
            venue_matrix,
            required_capabilities,
        });
        Ok(self)
    }

    pub fn with_simulated_execution_config(
        mut self,
        config: V4SimulatedExecutionConfig,
    ) -> Result<Self> {
        validate_simulated_execution_config(&config)?;
        self.simulated_execution = V4SimulatedExecutionRuntimeState::new(config, self.sequence);
        Ok(self)
    }

    pub fn run_backtest_bars(&mut self, bars: &[V4BacktestBarInput]) -> Result<V4BacktestArtifact> {
        let started_at_ms = bars.first().map(|bar| bar.ts_ms).unwrap_or(0);
        let mut ended_at_ms = started_at_ms;
        let mut trajectory = Vec::new();
        let mut risk_plane_decisions = Vec::new();
        let mut execution_capability_sources = Vec::new();
        let mut seen_risk_decisions = BTreeSet::new();
        let mut seen_execution_entries = BTreeSet::new();
        let mut symbols = BTreeSet::new();
        let mut sorted_bars = bars.to_vec();
        sorted_bars.sort_by(|left, right| {
            left.ts_ms
                .cmp(&right.ts_ms)
                .then_with(|| left.symbol.cmp(&right.symbol))
                .then_with(|| left.venue_id.cmp(&right.venue_id))
        });

        for bar in &sorted_bars {
            symbols.insert(bar.symbol.clone());
            ended_at_ms = ended_at_ms.max(bar.ts_ms);
            self.submit_market_bar_closed(
                &bar.venue_id,
                &bar.symbol,
                bar.close,
                bar.ts_ms,
                &bar.event_type,
            )?;
            self.advance_time(bar.ts_ms);
            let snapshot = self.memory_snapshot(bar.ts_ms);

            for machine in flatten_machine_snapshots(&snapshot.machines) {
                trajectory.push(V4BacktestMachineTrajectoryPoint {
                    ts_ms: snapshot.ts_ms,
                    event_sequence: snapshot.event_sequence,
                    machine_id: machine.machine_id.clone(),
                    template: machine.template.clone(),
                    state_id: machine.state_id.clone(),
                    status: v4_machine_status_label(machine.status).to_string(),
                    symbol: symbol_for_machine_id(&machine.machine_id),
                });
            }

            if let Some(decision) = &snapshot.risk_plane.last_decision {
                if seen_risk_decisions.insert(decision.decision_id.clone()) {
                    risk_plane_decisions.push(V4BacktestRiskPlaneDecisionRecord {
                        decision_id: decision.decision_id.clone(),
                        target_machine_id: decision.target_machine_id.clone(),
                        source_machine_id: decision.source_machine_id.clone(),
                        event_type: decision.event_type.clone(),
                        approved: decision.approved,
                        reason: decision.reason.clone(),
                        ts_ms: decision.ts_ms,
                        sequence: decision.sequence,
                        symbol: symbol_for_machine_id(&decision.target_machine_id),
                    });
                }
            }

            if let Some(decision) = &snapshot.execution.last_decision {
                for entry in &decision.entries {
                    let key = format!(
                        "{}:{}:{:?}:{:?}",
                        decision.decision_id,
                        decision.target_machine_id,
                        entry.capability,
                        entry.status
                    );
                    if seen_execution_entries.insert(key) {
                        execution_capability_sources.push(
                            V4BacktestExecutionCapabilitySourceRecord {
                                decision_id: decision.decision_id.clone(),
                                target_machine_id: decision.target_machine_id.clone(),
                                venue_id: decision.venue_id.clone(),
                                runtime_mode: decision.runtime_mode,
                                accepted: decision.accepted,
                                reason: decision.reason.clone(),
                                capability: entry.capability,
                                source: entry.source,
                                status: v4_execution_capability_status_label(entry.status)
                                    .to_string(),
                                ts_ms: decision.ts_ms,
                                sequence: decision.sequence,
                                symbol: symbol_for_machine_id(&decision.target_machine_id),
                            },
                        );
                    }
                }
            }
        }

        let final_snapshot = serde_json::to_value(self.memory_snapshot(ended_at_ms))
            .map_err(|error| anyhow!("序列化 v4 回测最终快照失败: {error}"))?;

        Ok(V4BacktestArtifact {
            schema_version: V4_BACKTEST_ARTIFACT_VERSION.to_string(),
            graph_id: self.graph.graph_id.clone(),
            started_at_ms,
            ended_at_ms,
            replay_mode: "deterministic_bar_replay".to_string(),
            input_bar_count: sorted_bars.len(),
            input_tick_count: None,
            symbols: symbols.into_iter().collect(),
            machine_trajectory: trajectory,
            risk_plane_decisions,
            execution_capability_sources,
            microstructure_metrics: Some(self.simulated_execution.microstructure_metrics()),
            final_snapshot: Some(final_snapshot),
        })
    }

    pub fn run_backtest_ticks(
        &mut self,
        ticks: &[V4BacktestTickInput],
    ) -> Result<V4BacktestArtifact> {
        let started_at_ms = ticks.first().map(|tick| tick.ts_ms).unwrap_or(0);
        let mut ended_at_ms = started_at_ms;
        let mut trajectory = Vec::new();
        let mut risk_plane_decisions = Vec::new();
        let mut execution_capability_sources = Vec::new();
        let mut seen_risk_decisions = BTreeSet::new();
        let mut seen_execution_entries = BTreeSet::new();
        let mut symbols = BTreeSet::new();
        let mut sorted_ticks = ticks.to_vec();
        crate::sandbox::replay::sort_v4_replay_ticks_deterministically(&mut sorted_ticks);

        for tick in &sorted_ticks {
            if !tick.price.is_finite() || tick.price <= 0.0 {
                return Err(anyhow!("v4 tick replay requires finite positive prices"));
            }
            if !tick.size.is_finite() || tick.size < 0.0 {
                return Err(anyhow!("v4 tick replay requires finite non-negative sizes"));
            }
            symbols.insert(tick.symbol.clone());
            ended_at_ms = ended_at_ms.max(tick.ts_ms);
            self.submit_market_price_tick(
                &tick.venue_id,
                &tick.symbol,
                tick.price,
                tick.ts_ms,
                &tick.event_type,
            )?;
            self.advance_time(tick.ts_ms);
            let snapshot = self.memory_snapshot(tick.ts_ms);

            for machine in flatten_machine_snapshots(&snapshot.machines) {
                trajectory.push(V4BacktestMachineTrajectoryPoint {
                    ts_ms: snapshot.ts_ms,
                    event_sequence: snapshot.event_sequence,
                    machine_id: machine.machine_id.clone(),
                    template: machine.template.clone(),
                    state_id: machine.state_id.clone(),
                    status: v4_machine_status_label(machine.status).to_string(),
                    symbol: symbol_for_machine_id(&machine.machine_id),
                });
            }

            if let Some(decision) = &snapshot.risk_plane.last_decision {
                if seen_risk_decisions.insert(decision.decision_id.clone()) {
                    risk_plane_decisions.push(V4BacktestRiskPlaneDecisionRecord {
                        decision_id: decision.decision_id.clone(),
                        target_machine_id: decision.target_machine_id.clone(),
                        source_machine_id: decision.source_machine_id.clone(),
                        event_type: decision.event_type.clone(),
                        approved: decision.approved,
                        reason: decision.reason.clone(),
                        ts_ms: decision.ts_ms,
                        sequence: decision.sequence,
                        symbol: symbol_for_machine_id(&decision.target_machine_id),
                    });
                }
            }

            if let Some(decision) = &snapshot.execution.last_decision {
                for entry in &decision.entries {
                    let key = format!(
                        "{}:{}:{:?}:{:?}",
                        decision.decision_id,
                        decision.target_machine_id,
                        entry.capability,
                        entry.status
                    );
                    if seen_execution_entries.insert(key) {
                        execution_capability_sources.push(
                            V4BacktestExecutionCapabilitySourceRecord {
                                decision_id: decision.decision_id.clone(),
                                target_machine_id: decision.target_machine_id.clone(),
                                venue_id: decision.venue_id.clone(),
                                runtime_mode: decision.runtime_mode,
                                accepted: decision.accepted,
                                reason: decision.reason.clone(),
                                capability: entry.capability,
                                source: entry.source,
                                status: v4_execution_capability_status_label(entry.status)
                                    .to_string(),
                                ts_ms: decision.ts_ms,
                                sequence: decision.sequence,
                                symbol: symbol_for_machine_id(&decision.target_machine_id),
                            },
                        );
                    }
                }
            }
        }

        let final_snapshot = serde_json::to_value(self.memory_snapshot(ended_at_ms))
            .map_err(|error| anyhow!("搴忓垪鍖?v4 tick 鍥炴祴鏈€缁堝揩鐓уけ璐? {error}"))?;

        Ok(V4BacktestArtifact {
            schema_version: V4_BACKTEST_ARTIFACT_VERSION.to_string(),
            graph_id: self.graph.graph_id.clone(),
            started_at_ms,
            ended_at_ms,
            replay_mode: "tick_replay".to_string(),
            input_bar_count: 0,
            input_tick_count: Some(sorted_ticks.len()),
            symbols: symbols.into_iter().collect(),
            machine_trajectory: trajectory,
            risk_plane_decisions,
            execution_capability_sources,
            microstructure_metrics: Some(self.simulated_execution.microstructure_metrics()),
            final_snapshot: Some(final_snapshot),
        })
    }

    pub fn submit_event(
        &mut self,
        event: V4RuntimeInputEvent,
    ) -> Result<V4PaperSimulatedRunOutput> {
        let start_index = self.event_log.len();
        self.enqueue_graph_event(
            event.event_type,
            event.source,
            event.payload,
            event.ts_ms,
            true,
            V4RuntimeEventOrigin::ExternalInput,
        );
        self.run_until_idle()?;
        Ok(self.output_since(start_index, event.ts_ms))
    }

    pub fn submit_market_price_tick(
        &mut self,
        venue_id: &str,
        symbol: &str,
        price: f64,
        ts_ms: u64,
        event_type: &str,
    ) -> Result<V4PaperSimulatedRunOutput> {
        if !price.is_finite() || price <= 0.0 {
            return Err(anyhow!(
                "v4 market price_tick requires a finite positive price"
            ));
        }
        let start_index = self.event_log.len();
        let outcome = self
            .simulated_execution
            .update_market_price(venue_id, symbol, price, ts_ms);
        self.record_simulated_execution_events(outcome, ts_ms);
        self.enqueue_graph_event(
            event_type,
            "market.okx",
            json!({
                "venue_id": venue_id,
                "symbol": symbol,
                "price": price,
                "last_price": price,
                "ts_ms": ts_ms,
            }),
            ts_ms,
            true,
            V4RuntimeEventOrigin::ExternalInput,
        );
        self.run_until_idle()?;
        Ok(self.output_since(start_index, ts_ms))
    }

    pub fn submit_market_bar_closed(
        &mut self,
        venue_id: &str,
        symbol: &str,
        close: f64,
        ts_ms: u64,
        event_type: &str,
    ) -> Result<V4PaperSimulatedRunOutput> {
        if !close.is_finite() || close <= 0.0 {
            return Err(anyhow!(
                "v4 market bar_closed requires a finite positive close price"
            ));
        }
        let start_index = self.event_log.len();
        let outcome = self
            .simulated_execution
            .update_market_price(venue_id, symbol, close, ts_ms);
        self.record_simulated_execution_events(outcome, ts_ms);
        self.enqueue_graph_event(
            event_type,
            "market.okx",
            json!({
                "venue_id": venue_id,
                "symbol": symbol,
                "close": close,
                "price": close,
                "ts_ms": ts_ms,
            }),
            ts_ms,
            true,
            V4RuntimeEventOrigin::ExternalInput,
        );
        self.run_until_idle()?;
        Ok(self.output_since(start_index, ts_ms))
    }

    pub fn advance_time(&mut self, now_ms: u64) -> Vec<V4RuntimeEventEnvelope> {
        let start_index = self.event_log.len();
        let machine_ids = self
            .graph
            .machines
            .iter()
            .map(|machine| machine.machine_id.clone())
            .collect::<Vec<_>>();

        for machine_id in machine_ids {
            let Some(machine) = self.machine_spec(&machine_id) else {
                continue;
            };
            let MachineSilencePolicy::SoftDormantAfter { ttl_ms } = machine.silence_policy else {
                continue;
            };
            let Some(state) = self.machines.get_mut(&machine_id) else {
                continue;
            };
            if state.status != V4MachineRuntimeStatus::Active {
                continue;
            }
            let last_observed = state
                .last_pulled_at_ms
                .or(state.last_event_at_ms)
                .unwrap_or(state.initialized_at_ms);
            if now_ms.saturating_sub(last_observed) >= ttl_ms {
                state.status = V4MachineRuntimeStatus::SoftSilent;
                self.record_control_event(
                    EVENT_SILENCE_ENTERED,
                    "runtime",
                    json!({
                        "machine_id": machine_id,
                        "ttl_ms": ttl_ms,
                        "last_observed_at_ms": last_observed
                    }),
                    now_ms,
                );
            }
        }

        let expiration_outcome = self.simulated_execution.expire_orders(now_ms);
        self.record_simulated_execution_events(expiration_outcome, now_ms);

        self.event_log[start_index..].to_vec()
    }

    pub fn pull_machine(
        &mut self,
        machine_id: &str,
        now_ms: u64,
    ) -> Result<Vec<V4RuntimeEventEnvelope>> {
        let start_index = self.event_log.len();
        let cache_policy = self
            .machine_spec(machine_id)
            .ok_or_else(|| anyhow!("未知 machine `{machine_id}`"))?
            .cache_policy
            .clone();
        let mut cached_to_return = None;
        let mut recovery_started = false;
        {
            let state = self
                .machines
                .get_mut(machine_id)
                .ok_or_else(|| anyhow!("未知 machine `{machine_id}`"))?;
            state.last_pulled_at_ms = Some(now_ms);
            if state.status == V4MachineRuntimeStatus::SoftSilent {
                if matches!(cache_policy, MachineCachePolicy::ReturnLastThenRecover) {
                    cached_to_return = state.cached_output.clone();
                }
                state.status = V4MachineRuntimeStatus::Recovering;
                recovery_started = true;
            }
        }

        self.record_control_event(
            EVENT_DOWNSTREAM_PULL,
            "runtime",
            json!({ "machine_id": machine_id }),
            now_ms,
        );

        if let Some(cached) = cached_to_return {
            self.record_control_event(
                EVENT_CACHE_RETURNED,
                machine_id,
                json!({ "machine_id": machine_id, "cached_output": cached }),
                now_ms,
            );
        }
        if recovery_started {
            self.record_control_event(
                EVENT_RECOVERY_STARTED,
                "runtime",
                json!({ "machine_id": machine_id }),
                now_ms,
            );
        }

        Ok(self.event_log[start_index..].to_vec())
    }

    pub fn complete_recovery(
        &mut self,
        machine_id: &str,
        now_ms: u64,
    ) -> Result<Vec<V4RuntimeEventEnvelope>> {
        let start_index = self.event_log.len();
        let should_complete = {
            let state = self
                .machines
                .get_mut(machine_id)
                .ok_or_else(|| anyhow!("未知 machine `{machine_id}`"))?;
            let should_complete = state.status == V4MachineRuntimeStatus::Recovering;
            if should_complete {
                state.status = V4MachineRuntimeStatus::Active;
                state.last_event_at_ms = Some(now_ms);
            }
            should_complete
        };

        if should_complete {
            self.record_control_event(
                EVENT_RECOVERY_COMPLETED,
                "runtime",
                json!({ "machine_id": machine_id }),
                now_ms,
            );
            self.record_control_event(
                EVENT_SILENCE_EXITED,
                "runtime",
                json!({ "machine_id": machine_id }),
                now_ms,
            );
        }

        Ok(self.event_log[start_index..].to_vec())
    }

    pub fn update_simulated_market_price(
        &mut self,
        venue_id: &str,
        symbol: &str,
        price: f64,
        now_ms: u64,
    ) -> Result<Vec<V4RuntimeEventEnvelope>> {
        if !price.is_finite() || price <= 0.0 {
            return Err(anyhow!("模拟行情价格必须是有限数且大于 0"));
        }

        let start_index = self.event_log.len();
        let outcome = self
            .simulated_execution
            .update_market_price(venue_id, symbol, price, now_ms);
        self.record_simulated_execution_events(outcome, now_ms);
        Ok(self.event_log[start_index..].to_vec())
    }

    pub fn memory_snapshot(&self, now_ms: u64) -> V4RuntimeMemorySnapshot {
        V4RuntimeMemorySnapshot {
            graph_id: self.graph.graph_id.clone(),
            runtime_mode: self.runtime_mode,
            ts_ms: now_ms,
            machines: self
                .graph
                .machines
                .iter()
                .filter_map(|machine| self.machine_snapshot(machine))
                .collect(),
            risk_plane: self.risk_plane_snapshot(),
            execution: self.execution_snapshot(),
            simulated_execution: self.simulated_execution_snapshot(),
            venue_adapter_boundary: self.venue_adapter_boundary(),
            complexity_metrics: Some(ComplexityMetrics::from_machine_graph(
                &self.graph,
                default_v4_runtime_mode_contract().modes.len() as u32,
                0,
            )),
            event_sequence: self.sequence,
            provider_order_submission_attached: self.provider_order_submission_attached,
        }
    }

    fn machine_snapshot(&self, machine: &V4MachineContract) -> Option<V4MachineRuntimeSnapshot> {
        let state = self.machines.get(&machine.machine_id)?;
        let children = machine
            .states
            .iter()
            .find(|candidate| candidate.state_id == state.state_id)
            .and_then(|active_state| active_state.child_machine.as_deref())
            .and_then(|child_machine| self.machine_snapshot(child_machine))
            .into_iter()
            .collect();
        Some(V4MachineRuntimeSnapshot {
            machine_id: machine.machine_id.clone(),
            template: machine.template.clone(),
            state_id: state.state_id.clone(),
            status: state.status,
            memory: state.memory.clone(),
            cached_output: state.cached_output.clone(),
            last_pulled_at_ms: state.last_pulled_at_ms,
            last_event_at_ms: state.last_event_at_ms,
            children,
        })
    }

    pub fn machine_status(&self, machine_id: &str) -> Option<V4MachineRuntimeStatus> {
        self.machines.get(machine_id).map(|state| state.status)
    }

    pub fn machine_state_id(&self, machine_id: &str) -> Option<&str> {
        self.machines
            .get(machine_id)
            .map(|state| state.state_id.as_str())
    }

    pub fn event_log(&self) -> &[V4RuntimeEventEnvelope] {
        &self.event_log
    }

    pub fn risk_plane_snapshot(&self) -> V4RiskPlaneRuntimeSnapshot {
        V4RiskPlaneRuntimeSnapshot {
            required: self.risk_plane.required,
            machine_ids: self.risk_plane.machine_ids.iter().cloned().collect(),
            min_priority: self.risk_plane.min_priority,
            approved_event_count: self.risk_plane.approved_event_count,
            rejected_event_count: self.risk_plane.rejected_event_count,
            real_order_path_unlocked: self.risk_plane.approved_event_count > 0
                && self.risk_plane.rejected_event_count == 0,
            last_decision: self.risk_plane.last_decision.clone(),
        }
    }

    pub fn execution_snapshot(&self) -> V4ExecutionRuntimeSnapshot {
        V4ExecutionRuntimeSnapshot {
            venue_id: self
                .execution
                .capability_policy
                .as_ref()
                .map(|policy| policy.venue_matrix.venue_id.clone()),
            required_capabilities: self
                .execution
                .capability_policy
                .as_ref()
                .map(|policy| policy.required_capabilities.clone())
                .unwrap_or_default(),
            accepted_count: self.execution.accepted_count,
            rejected_count: self.execution.rejected_count,
            last_decision: self.execution.last_decision.clone(),
        }
    }

    pub fn simulated_execution_snapshot(&self) -> V4SimulatedExecutionSnapshot {
        self.simulated_execution.snapshot()
    }

    pub fn venue_adapter_boundary(&self) -> V4VenueAdapterRuntimeBoundary {
        let runtime_mode_contract = default_v4_runtime_mode_contract();
        let mode_spec = runtime_mode_contract
            .mode_spec(self.runtime_mode)
            .expect("default v4 runtime mode contract declares all runtime modes");
        V4VenueAdapterRuntimeBoundary {
            provider_order_submission_attached: self.provider_order_submission_attached,
            provider_order_submission_allowed: mode_spec.provider_order_submission_allowed,
            settlement_authority: mode_spec.settlement_authority,
            live_actual_submission_allowed: false,
            rejection_before_provider_submit: !self.provider_order_submission_attached,
            reason: if self.provider_order_submission_attached {
                "运行时配置已接入 provider 下单提交".to_string()
            } else {
                "v4 PaperSimulated runtime 保持 VenueAdapter 提交断开；provider_native 下单必须在提交 provider 前拒绝".to_string()
            },
        }
    }

    fn output_since(&self, start_index: usize, now_ms: u64) -> V4PaperSimulatedRunOutput {
        V4PaperSimulatedRunOutput {
            runtime_mode: self.runtime_mode,
            events: self.event_log[start_index..].to_vec(),
            memory_snapshot: self.memory_snapshot(now_ms),
            provider_order_submission_attached: self.provider_order_submission_attached,
        }
    }

    fn run_until_idle(&mut self) -> Result<()> {
        let mut steps = 0usize;
        while let Some(event) = self.event_queue.pop_front() {
            steps += 1;
            if steps > V4_RUNTIME_MAX_EVENT_STEPS {
                return Err(anyhow!(
                    "v4 runtime 超过最大事件步数 {}",
                    V4_RUNTIME_MAX_EVENT_STEPS
                ));
            }
            self.process_event(event)?;
        }
        Ok(())
    }

    fn process_event(&mut self, event: V4RuntimeEventEnvelope) -> Result<()> {
        if let Err(reason) = self.validate_event_payload(&event) {
            self.record_event_rejected(&event, reason);
            return Ok(());
        }

        if self.machines.contains_key(event.source.as_str()) {
            if let Some(source_state) = self.machines.get_mut(event.source.as_str()) {
                source_state.last_pulled_at_ms = Some(event.ts_ms);
            }
        }

        let mut candidates = self
            .graph
            .machines
            .iter()
            .filter_map(|machine| self.transition_candidate_for_machine(machine, &event))
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.sort_id.cmp(&right.sort_id))
        });

        for candidate in candidates {
            let machine_id = candidate.machine_id;
            let transition = candidate.transition;
            let Some(machine) = self.machine_spec(&machine_id).cloned() else {
                continue;
            };
            if let Some(guard) = transition
                .guard
                .as_ref()
                .filter(|guard| !guard.trim().is_empty())
            {
                self.record_event_rejected(
                    &event,
                    format!(
                        "transition `{}` declares unsupported guard `{}`; v4 runtime fails closed",
                        transition.transition_id, guard
                    ),
                );
                continue;
            }
            if let Some(action) = &transition.action {
                let declared_memory = machine
                    .memory
                    .iter()
                    .map(|field| field.name.as_str())
                    .collect::<BTreeSet<_>>();
                if let Some(memory_name) = action
                    .memory_writes
                    .iter()
                    .find(|name| !declared_memory.contains(name.as_str()))
                {
                    self.record_event_rejected(
                        &event,
                        format!(
                            "transition `{}` writes undeclared memory field `{}`",
                            transition.transition_id, memory_name
                        ),
                    );
                    continue;
                }
            }
            if matches!(machine.template, MachineTemplateKind::Execution) {
                let decision = self.evaluate_risk_plane_for_execution(&machine_id, &event);
                let approved = decision.approved;
                self.record_risk_plane_decision(decision, event.ts_ms);
                if !approved {
                    continue;
                }

                let execution_decision =
                    self.evaluate_execution_capabilities_for_execution(&machine_id, event.ts_ms);
                let execution_accepted = execution_decision.accepted;
                self.record_execution_decision(execution_decision, event.ts_ms);
                if !execution_accepted {
                    continue;
                }
            }
            let emitted_events = transition
                .action
                .as_ref()
                .map(|action| action.emits.clone())
                .unwrap_or_default();
            let mut silence_exited = false;
            let mut recovery_completed = false;

            {
                let Some(runtime_state) = self.machines.get_mut(machine_id.as_str()) else {
                    continue;
                };
                if runtime_state.state_id != transition.from_state {
                    continue;
                }
                if runtime_state.status == V4MachineRuntimeStatus::SoftSilent {
                    runtime_state.status = V4MachineRuntimeStatus::Active;
                    silence_exited = true;
                }
                if runtime_state.status == V4MachineRuntimeStatus::Recovering {
                    runtime_state.status = V4MachineRuntimeStatus::Active;
                    recovery_completed = true;
                }

                runtime_state.state_id = transition.to_state.clone();
                runtime_state.last_event_at_ms = Some(event.ts_ms);

                if let Some(action) = &transition.action {
                    for memory_name in &action.memory_writes {
                        if let Some(value) = event.payload.get(memory_name).cloned() {
                            runtime_state.memory.insert(memory_name.clone(), value);
                        }
                    }
                }

                if matches!(
                    machine.cache_policy,
                    MachineCachePolicy::ReturnLastThenRecover
                ) {
                    runtime_state.cached_output = Some(V4CachedMachineOutput {
                        machine_id: machine_id.clone(),
                        state_id: runtime_state.state_id.clone(),
                        event_type: event.event_type.clone(),
                        emitted_events: emitted_events.clone(),
                        payload: event.payload.clone(),
                        updated_at_ms: event.ts_ms,
                        sequence: self.sequence,
                    });
                }
            }

            if silence_exited {
                self.record_control_event(
                    EVENT_SILENCE_EXITED,
                    "runtime",
                    json!({ "machine_id": machine_id, "reason": "event_arrived" }),
                    event.ts_ms,
                );
            }
            if recovery_completed {
                self.record_control_event(
                    EVENT_RECOVERY_COMPLETED,
                    "runtime",
                    json!({ "machine_id": machine_id, "reason": "event_arrived" }),
                    event.ts_ms,
                );
            }

            self.record_control_event(
                EVENT_TRANSITION_APPLIED,
                machine_id.as_str(),
                json!({
                    "machine_id": machine_id,
                    "transition_id": transition.transition_id,
                    "from_state": transition.from_state,
                    "to_state": transition.to_state,
                    "input_event_type": event.event_type,
                }),
                event.ts_ms,
            );

            if matches!(machine.template, MachineTemplateKind::Execution) {
                let outcome = self.apply_runtime_simulated_execution_for_transition(
                    machine_id.as_str(),
                    &event,
                    event.ts_ms,
                )?;
                self.record_simulated_execution_events(outcome, event.ts_ms);
            }

            for emitted_event in emitted_events {
                let payload = self.payload_for_emitted_event(
                    emitted_event.as_str(),
                    machine_id.as_str(),
                    &event,
                );
                self.enqueue_graph_event(
                    emitted_event,
                    machine_id.clone(),
                    payload,
                    event.ts_ms,
                    true,
                    V4RuntimeEventOrigin::MachineEmit,
                );
            }
        }

        Ok(())
    }

    fn validate_event_payload(&self, event: &V4RuntimeEventEnvelope) -> Result<(), String> {
        let Some(catalog) = &self.graph.event_catalog else {
            return Ok(());
        };
        let Some(spec) = catalog
            .events
            .iter()
            .find(|candidate| candidate.event_type == event.event_type)
        else {
            return Err(format!(
                "event `{}` is not declared in MachineEventCatalog",
                event.event_type
            ));
        };
        let Some(payload) = event.payload.as_object() else {
            return Err(format!(
                "event `{}` payload must be a JSON object",
                event.event_type
            ));
        };

        for field in &spec.payload_fields {
            let value = payload.get(field.name.as_str());
            match value {
                None if field.required => {
                    return Err(format!(
                        "event `{}` payload missing required field `{}`",
                        event.event_type, field.name
                    ));
                }
                None => continue,
                Some(Value::Null) if field.nullable => continue,
                Some(Value::Null) => {
                    return Err(format!(
                        "event `{}` payload field `{}` is null but not nullable",
                        event.event_type, field.name
                    ));
                }
                Some(value) => validate_payload_field_type(field, value).map_err(|reason| {
                    format!(
                        "event `{}` payload field `{}` type mismatch: {}",
                        event.event_type, field.name, reason
                    )
                })?,
            }
        }

        Ok(())
    }

    fn record_event_rejected(&mut self, event: &V4RuntimeEventEnvelope, reason: String) {
        self.record_control_event(
            V4_RUNTIME_EVENT_REJECTED_EVENT,
            "runtime.validation",
            json!({
                "rejected_event_sequence": event.sequence,
                "rejected_event_type": event.event_type,
                "rejected_event_source": event.source,
                "reason": reason,
                "payload": event.payload,
            }),
            event.ts_ms,
        );
    }

    fn evaluate_risk_plane_for_execution(
        &self,
        target_machine_id: &str,
        event: &V4RuntimeEventEnvelope,
    ) -> V4RiskPlaneRuntimeDecision {
        let reject = |reason: String| V4RiskPlaneRuntimeDecision {
            decision_id: format!("risk-decision-{}", self.sequence + 1),
            target_machine_id: target_machine_id.to_string(),
            source_machine_id: event.source.clone(),
            event_type: event.event_type.clone(),
            approved: false,
            reason,
            ts_ms: event.ts_ms,
            sequence: self.sequence + 1,
        };

        if !self.risk_plane.required {
            return reject(
                "execution transition requires a runtime Risk Plane, but none is required"
                    .to_string(),
            );
        }
        if !self.risk_plane.machine_ids.contains(event.source.as_str()) {
            return reject(format!(
                "execution event source `{}` is not a runtime Risk Plane machine",
                event.source
            ));
        }
        if event.origin != V4RuntimeEventOrigin::MachineEmit {
            return reject(
                "execution event must be emitted by a Risk Plane machine transition".to_string(),
            );
        }
        if self.event_source_kind(&event.event_type) != Some(MachineEventSourceKind::RiskPlane) {
            return reject(format!(
                "execution event `{}` is not declared as a Risk Plane event",
                event.event_type
            ));
        }
        if event.payload.get("risk_plane_approved") != Some(&Value::Bool(true)) {
            return reject("Risk Plane event payload does not carry explicit approval".to_string());
        }

        let Some(source_machine) = self.machine_spec(event.source.as_str()) else {
            return reject(format!(
                "runtime Risk Plane source `{}` is not a declared machine",
                event.source
            ));
        };
        if !matches!(source_machine.template, MachineTemplateKind::Decision) {
            return reject(format!(
                "runtime Risk Plane source `{}` is not a Decision machine",
                event.source
            ));
        }
        if source_machine.priority < self.risk_plane.min_priority {
            return reject(format!(
                "runtime Risk Plane source `{}` priority {} is below min_priority {}",
                event.source, source_machine.priority, self.risk_plane.min_priority
            ));
        }
        match self.machines.get(event.source.as_str()) {
            Some(state) if state.status == V4MachineRuntimeStatus::Active => {}
            Some(state) => {
                return reject(format!(
                    "runtime Risk Plane source `{}` is not active: {:?}",
                    event.source, state.status
                ));
            }
            None => {
                return reject(format!(
                    "runtime Risk Plane source `{}` has no runtime state",
                    event.source
                ));
            }
        }

        V4RiskPlaneRuntimeDecision {
            decision_id: format!("risk-decision-{}", self.sequence + 1),
            target_machine_id: target_machine_id.to_string(),
            source_machine_id: event.source.clone(),
            event_type: event.event_type.clone(),
            approved: true,
            reason: "Risk Plane approved execution transition".to_string(),
            ts_ms: event.ts_ms,
            sequence: self.sequence + 1,
        }
    }

    fn record_risk_plane_decision(&mut self, decision: V4RiskPlaneRuntimeDecision, ts_ms: u64) {
        if decision.approved {
            self.risk_plane.approved_event_count += 1;
        } else {
            self.risk_plane.rejected_event_count += 1;
        }
        self.risk_plane.last_decision = Some(decision.clone());

        self.record_control_event(
            if decision.approved {
                EVENT_RISK_PLANE_APPROVED
            } else {
                EVENT_RISK_PLANE_REJECTED
            },
            "runtime.risk_plane",
            json!({ "decision": decision }),
            ts_ms,
        );
    }

    fn evaluate_execution_capabilities_for_execution(
        &self,
        target_machine_id: &str,
        ts_ms: u64,
    ) -> V4ExecutionRuntimeDecision {
        let decision_id = format!("execution-capability-decision-{}", self.sequence + 1);

        let Some(policy) = &self.execution.capability_policy else {
            return V4ExecutionRuntimeDecision {
                decision_id,
                target_machine_id: target_machine_id.to_string(),
                venue_id: "<missing>".to_string(),
                runtime_mode: self.runtime_mode,
                accepted: false,
                reason: "缺少 execution capability policy".to_string(),
                entries: vec![V4ExecutionCapabilityRuntimeEntry {
                    capability: ExecutionCapabilityKind::Market,
                    source: CapabilitySupportSource::Unsupported,
                    status: V4ExecutionCapabilityRuntimeStatus::PolicyMissing,
                    reason: "缺少 execution capability policy".to_string(),
                }],
                provider_order_submission_attached: self.provider_order_submission_attached,
                ts_ms,
                sequence: self.sequence + 1,
            };
        };

        if policy.required_capabilities.is_empty() {
            return V4ExecutionRuntimeDecision {
                decision_id,
                target_machine_id: target_machine_id.to_string(),
                venue_id: policy.venue_matrix.venue_id.clone(),
                runtime_mode: self.runtime_mode,
                accepted: false,
                reason: "ExecutionMachine 至少需要声明一个 execution capability".to_string(),
                entries: Vec::new(),
                provider_order_submission_attached: self.provider_order_submission_attached,
                ts_ms,
                sequence: self.sequence + 1,
            };
        }

        let runtime_mode_contract = default_v4_runtime_mode_contract();
        let mut entries = Vec::new();
        let mut errors = Vec::new();

        for capability in &policy.required_capabilities {
            let entry = match policy.venue_matrix.capability_entry(capability) {
                Some(entry) => entry,
                None => {
                    let reason = format!(
                        "execution capability `{:?}` 未在 venue `{}` 中声明",
                        capability, policy.venue_matrix.venue_id
                    );
                    errors.push(reason.clone());
                    entries.push(V4ExecutionCapabilityRuntimeEntry {
                        capability: *capability,
                        source: CapabilitySupportSource::Unsupported,
                        status: V4ExecutionCapabilityRuntimeStatus::NotDeclared,
                        reason,
                    });
                    continue;
                }
            };

            if matches!(entry.source, CapabilitySupportSource::Unsupported) {
                let reason = format!(
                    "execution capability `{:?}` 在 venue `{}` 中不受支持",
                    capability, policy.venue_matrix.venue_id
                );
                errors.push(reason.clone());
                entries.push(V4ExecutionCapabilityRuntimeEntry {
                    capability: *capability,
                    source: entry.source,
                    status: V4ExecutionCapabilityRuntimeStatus::Unsupported,
                    reason,
                });
                continue;
            }

            match policy.venue_matrix.require_supported_for_mode(
                capability,
                self.runtime_mode,
                &runtime_mode_contract,
            ) {
                Ok(source) => entries.push(V4ExecutionCapabilityRuntimeEntry {
                    capability: *capability,
                    source,
                    status: V4ExecutionCapabilityRuntimeStatus::Accepted,
                    reason: format!(
                        "execution capability `{:?}` 在 runtime mode `{:?}` 下以 `{:?}` 来源通过",
                        capability, self.runtime_mode, source
                    ),
                }),
                Err(reason) => {
                    errors.push(reason.clone());
                    entries.push(V4ExecutionCapabilityRuntimeEntry {
                        capability: *capability,
                        source: entry.source,
                        status: V4ExecutionCapabilityRuntimeStatus::ModeRejected,
                        reason,
                    });
                }
            }
        }

        V4ExecutionRuntimeDecision {
            decision_id,
            target_machine_id: target_machine_id.to_string(),
            venue_id: policy.venue_matrix.venue_id.clone(),
            runtime_mode: self.runtime_mode,
            accepted: errors.is_empty(),
            reason: if errors.is_empty() {
                "execution capabilities 已通过当前 runtime mode 校验".to_string()
            } else {
                errors.join("; ")
            },
            entries,
            provider_order_submission_attached: self.provider_order_submission_attached,
            ts_ms,
            sequence: self.sequence + 1,
        }
    }

    fn record_execution_decision(&mut self, decision: V4ExecutionRuntimeDecision, ts_ms: u64) {
        if decision.accepted {
            self.execution.accepted_count += 1;
        } else {
            self.execution.rejected_count += 1;
        }
        self.execution.last_decision = Some(decision.clone());

        self.record_control_event(
            if decision.accepted {
                EVENT_EXECUTION_CAPABILITY_ACCEPTED
            } else {
                EVENT_EXECUTION_CAPABILITY_REJECTED
            },
            "runtime.execution_capability",
            json!({ "decision": decision }),
            ts_ms,
        );
    }

    fn apply_runtime_simulated_execution_for_transition(
        &mut self,
        machine_id: &str,
        event: &V4RuntimeEventEnvelope,
        ts_ms: u64,
    ) -> Result<V4SimulatedExecutionOutcome> {
        let runtime_mode_contract = default_v4_runtime_mode_contract();
        let mode_spec = runtime_mode_contract
            .mode_spec(self.runtime_mode)
            .ok_or_else(|| anyhow!("未声明 runtime mode `{:?}`", self.runtime_mode))?;

        if mode_spec.settlement_authority != RuntimeSettlementAuthority::LocalSimulated {
            let request = self.build_simulated_order_request(machine_id, event);
            return Ok(self.simulated_execution.reject_order(
                request,
                event.sequence,
                ts_ms,
                "runtime mode 不是 local_simulated；provider submission 已断开".to_string(),
            ));
        }

        if let Some(amend_order_id) =
            payload_string(&event.payload, &["amend_order_id", "replace_order_id"])
        {
            if let Err(reason) = self
                .validate_single_execution_capability(ExecutionCapabilityKind::CancelReplaceAmend)
            {
                let request = self.build_simulated_order_request(machine_id, event);
                return Ok(self.simulated_execution.reject_order(
                    request,
                    event.sequence,
                    ts_ms,
                    reason,
                ));
            }
            return Ok(self.simulated_execution.amend_order(
                &amend_order_id,
                payload_f64(
                    &event.payload,
                    &["new_reference_price", "reference_price", "price"],
                ),
                payload_f64(&event.payload, &["new_limit_price", "limit_price"]),
                payload_f64(&event.payload, &["new_trigger_price", "trigger_price"]),
                payload_f64(&event.payload, &["new_quantity", "quantity", "qty"]),
                ts_ms,
            ));
        }

        let request = self.build_simulated_order_request(machine_id, event);
        if let Err(reason) = self.validate_simulated_order_capabilities(&request) {
            return Ok(self.simulated_execution.reject_order(
                request,
                event.sequence,
                ts_ms,
                reason,
            ));
        }

        Ok(self
            .simulated_execution
            .submit_order(request, event.sequence, ts_ms))
    }

    fn build_simulated_order_request(
        &self,
        machine_id: &str,
        event: &V4RuntimeEventEnvelope,
    ) -> V4SimulatedOrderRequest {
        let config = &self.simulated_execution.config;
        let machine_metadata = self
            .machine_spec(machine_id)
            .map(|machine| &machine.metadata)
            .unwrap_or(&self.graph.metadata);
        let venue_id = payload_string(&event.payload, &["venue_id", "venue", "exchange"])
            .or_else(|| {
                self.execution
                    .capability_policy
                    .as_ref()
                    .map(|policy| policy.venue_matrix.venue_id.clone())
            })
            .or_else(|| metadata_string(machine_metadata, "core_venue_kind"))
            .unwrap_or_else(|| config.default_venue_id.clone());
        let symbol = payload_string(&event.payload, &["symbol", "instrument"])
            .or_else(|| metadata_string(&self.graph.metadata, "default_symbol"))
            .unwrap_or_else(|| config.default_symbol.clone());
        let action = payload_string(
            &event.payload,
            &["action", "position_action", "order_action", "side"],
        )
        .and_then(|raw| parse_position_action(raw.as_str()))
        .unwrap_or(V4SimulatedPositionAction::Buy);
        let order_type = payload_string(&event.payload, &["order_type", "type"])
            .and_then(|raw| parse_order_type(raw.as_str()))
            .unwrap_or(V4SimulatedOrderType::Market);
        let reference_price =
            payload_f64(&event.payload, &["reference_price", "price", "last_price"])
                .or_else(|| self.latest_market_price(&venue_id, &symbol))
                .unwrap_or(config.default_price);

        V4SimulatedOrderRequest {
            order_id: payload_string(&event.payload, &["order_id"]),
            client_order_id: payload_string(&event.payload, &["client_order_id"]),
            venue_id,
            symbol,
            action,
            order_type,
            quantity: payload_f64(&event.payload, &["quantity", "qty"])
                .unwrap_or(config.default_quantity),
            reference_price,
            limit_price: payload_f64(&event.payload, &["limit_price"]),
            trigger_price: payload_f64(&event.payload, &["trigger_price", "stop_price"]),
            take_profit_price: payload_f64(&event.payload, &["take_profit_price", "tp_price"]),
            stop_loss_price: payload_f64(&event.payload, &["stop_loss_price", "sl_price"]),
            trailing_offset_bps: payload_f64(
                &event.payload,
                &["trailing_offset_bps", "trail_offset_bps"],
            ),
            expire_at_ms: payload_u64(&event.payload, &["expire_at_ms", "expires_at_ms"]),
            time_in_force: payload_string(&event.payload, &["time_in_force", "tif"])
                .or_else(|| metadata_string(machine_metadata, "core_time_in_force"))
                .and_then(|raw| parse_time_in_force(raw.as_str())),
            post_only: payload_bool(&event.payload, &["post_only"]).unwrap_or(false),
            reduce_only: payload_bool(&event.payload, &["reduce_only"]).unwrap_or(false),
            close_only: payload_bool(&event.payload, &["close_only"]).unwrap_or(false),
            allow_partial_fill: payload_bool(&event.payload, &["allow_partial_fill"])
                .unwrap_or(config.allow_partial_fill),
            fee_bps: payload_f64(&event.payload, &["fee_bps"]).unwrap_or(config.default_fee_bps),
            slippage_bps: payload_f64(&event.payload, &["slippage_bps"])
                .unwrap_or(config.default_slippage_bps),
            max_fill_quantity: payload_f64(&event.payload, &["max_fill_quantity"])
                .or(config.max_fill_quantity),
        }
    }

    fn latest_market_price(&self, venue_id: &str, symbol: &str) -> Option<f64> {
        self.simulated_execution
            .market_prices
            .get(&(venue_id.to_string(), symbol.to_string()))
            .copied()
    }

    fn validate_simulated_order_capabilities(
        &self,
        request: &V4SimulatedOrderRequest,
    ) -> Result<(), String> {
        let Some(policy) = &self.execution.capability_policy else {
            return Err("缺少 execution capability policy".to_string());
        };
        let runtime_mode_contract = default_v4_runtime_mode_contract();
        let mut errors = Vec::new();
        for capability in simulated_order_required_capabilities(request) {
            if let Err(reason) = policy.venue_matrix.require_supported_for_mode(
                &capability,
                self.runtime_mode,
                &runtime_mode_contract,
            ) {
                errors.push(reason);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    fn validate_single_execution_capability(
        &self,
        capability: ExecutionCapabilityKind,
    ) -> Result<(), String> {
        let Some(policy) = &self.execution.capability_policy else {
            return Err("缂哄皯 execution capability policy".to_string());
        };
        let runtime_mode_contract = default_v4_runtime_mode_contract();
        policy
            .venue_matrix
            .require_supported_for_mode(&capability, self.runtime_mode, &runtime_mode_contract)
            .map(|_| ())
    }

    fn record_simulated_execution_events(
        &mut self,
        outcome: V4SimulatedExecutionOutcome,
        ts_ms: u64,
    ) {
        for (event_type, payload) in outcome.events {
            self.record_control_event(event_type, "runtime.execution_simulator", payload, ts_ms);
        }
    }

    fn payload_for_emitted_event(
        &self,
        event_type: &str,
        machine_id: &str,
        input_event: &V4RuntimeEventEnvelope,
    ) -> Value {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "emitted_by".to_string(),
            Value::String(machine_id.to_string()),
        );
        payload.insert(
            "input_event_type".to_string(),
            Value::String(input_event.event_type.clone()),
        );

        if let Some(spec) = self.graph.event_catalog.as_ref().and_then(|catalog| {
            catalog
                .events
                .iter()
                .find(|candidate| candidate.event_type == event_type)
        }) {
            if let Some(state) = self.machines.get(machine_id) {
                for field in &spec.payload_fields {
                    if let Some(value) = state.memory.get(field.name.as_str()) {
                        payload.insert(field.name.clone(), value.clone());
                    }
                }
            }
            for field in &spec.payload_fields {
                if payload.contains_key(field.name.as_str()) {
                    continue;
                }
                if let Some(value) = self.graph.metadata.get(field.name.as_str()) {
                    payload.insert(field.name.clone(), value.clone());
                } else if field.name == "execution_id" {
                    payload.insert(
                        field.name.clone(),
                        self.graph
                            .machines
                            .iter()
                            .find(|machine| {
                                matches!(machine.template, MachineTemplateKind::Execution)
                            })
                            .and_then(|machine| machine.metadata.get("core_execution_id"))
                            .cloned()
                            .unwrap_or(Value::Null),
                    );
                }
            }
            if spec.source_kind == MachineEventSourceKind::RiskPlane {
                payload.insert("risk_plane_approved".to_string(), Value::Bool(true));
                payload.insert(
                    "risk_plane_machine_id".to_string(),
                    Value::String(machine_id.to_string()),
                );
                payload.insert(
                    "risk_plane_decision".to_string(),
                    Value::String("approved".to_string()),
                );
            }
        }

        Value::Object(payload)
    }

    fn event_source_kind(&self, event_type: &str) -> Option<MachineEventSourceKind> {
        self.graph
            .event_catalog
            .as_ref()?
            .events
            .iter()
            .find(|candidate| candidate.event_type == event_type)
            .map(|event| event.source_kind.clone())
    }

    fn enqueue_graph_event(
        &mut self,
        event_type: impl Into<String>,
        source: impl Into<String>,
        payload: Value,
        ts_ms: u64,
        replayable: bool,
        origin: V4RuntimeEventOrigin,
    ) {
        self.sequence += 1;
        let event = V4RuntimeEventEnvelope {
            sequence: self.sequence,
            event_type: event_type.into(),
            source: source.into(),
            origin,
            ts_ms,
            payload,
            replayable,
        };
        self.event_log.push(event.clone());
        self.event_queue.push_back(event);
    }

    fn record_control_event(
        &mut self,
        event_type: impl Into<String>,
        source: impl Into<String>,
        payload: Value,
        ts_ms: u64,
    ) {
        self.sequence += 1;
        self.event_log.push(V4RuntimeEventEnvelope {
            sequence: self.sequence,
            event_type: event_type.into(),
            source: source.into(),
            origin: V4RuntimeEventOrigin::RuntimeControl,
            ts_ms,
            payload,
            replayable: true,
        });
    }

    fn transition_candidate_for_machine(
        &self,
        machine: &V4MachineContract,
        event: &V4RuntimeEventEnvelope,
    ) -> Option<RuntimeTransitionCandidate> {
        let runtime_state = self.machines.get(machine.machine_id.as_str())?;
        if let Some(transition) = matching_transition(machine, runtime_state, event) {
            return Some(RuntimeTransitionCandidate {
                priority: machine.priority,
                sort_id: machine.machine_id.clone(),
                machine_id: machine.machine_id.clone(),
                transition: transition.clone(),
            });
        }

        let child_machine = machine
            .states
            .iter()
            .find(|state| state.state_id == runtime_state.state_id)
            .and_then(|state| state.child_machine.as_deref())?;
        let child_state = self.machines.get(child_machine.machine_id.as_str())?;
        let transition = matching_transition(child_machine, child_state, event)?;
        Some(RuntimeTransitionCandidate {
            priority: machine.priority.saturating_sub(1),
            sort_id: format!("{}::{}", machine.machine_id, child_machine.machine_id),
            machine_id: child_machine.machine_id.clone(),
            transition: transition.clone(),
        })
    }

    fn machine_spec(&self, machine_id: &str) -> Option<&V4MachineContract> {
        self.graph
            .machines
            .iter()
            .find_map(|machine| find_machine_spec(machine, machine_id))
    }
}

fn matching_transition<'a>(
    machine: &'a V4MachineContract,
    runtime_state: &MachineRuntimeState,
    event: &V4RuntimeEventEnvelope,
) -> Option<&'a MachineTransition> {
    machine.transitions.iter().find(|transition| {
        transition.from_state == runtime_state.state_id
            && transition.event.event_type == event.event_type
            && transition_source_matches(transition.event.source.as_deref(), event)
            && transition_freshness_matches(transition.event.freshness.clone(), event)
    })
}

fn find_machine_spec<'a>(
    machine: &'a V4MachineContract,
    machine_id: &str,
) -> Option<&'a V4MachineContract> {
    if machine.machine_id == machine_id {
        return Some(machine);
    }
    machine
        .states
        .iter()
        .filter_map(|state| state.child_machine.as_deref())
        .find_map(|child_machine| find_machine_spec(child_machine, machine_id))
}

fn flatten_machine_snapshots<'a>(
    machines: &'a [V4MachineRuntimeSnapshot],
) -> Vec<&'a V4MachineRuntimeSnapshot> {
    let mut flattened = Vec::new();
    for machine in machines {
        flattened.push(machine);
        flattened.extend(flatten_machine_snapshots(&machine.children));
    }
    flattened
}

impl V4SimulatedExecutionRuntimeState {
    fn new(config: V4SimulatedExecutionConfig, sequence: u64) -> Self {
        Self {
            cash_balance: config.starting_cash,
            config,
            realized_fees: 0.0,
            order_sequence: sequence,
            rejected_order_count: 0,
            positions: BTreeMap::new(),
            orders: Vec::new(),
            fills: Vec::new(),
            asset_curve: Vec::new(),
            market_prices: BTreeMap::new(),
        }
    }

    fn submit_order(
        &mut self,
        request: V4SimulatedOrderRequest,
        source_event_sequence: u64,
        ts_ms: u64,
    ) -> V4SimulatedExecutionOutcome {
        if let Err(reason) = validate_simulated_order_request(&request) {
            return self.reject_order(request, source_event_sequence, ts_ms, reason);
        }

        if let Err(reason) = self.validate_position_action(&request) {
            return self.reject_order(request, source_event_sequence, ts_ms, reason);
        }

        let side = request.action.side();
        if let Some(reason) = self.pre_execution_rejection_reason(&request, side) {
            return self.reject_order(request, source_event_sequence, ts_ms, reason);
        }
        if request.order_type == V4SimulatedOrderType::OcoBracket {
            return self.submit_oco_bracket(request, source_event_sequence, ts_ms);
        }
        if let Some(reason) = self.non_executable_resting_reason(&request, side) {
            let order = self.accepted_order(&request, source_event_sequence, ts_ms);
            self.orders.push(order.clone());
            return V4SimulatedExecutionOutcome {
                events: vec![
                    (
                        EVENT_EXECUTION_ORDER_ACKNOWLEDGED,
                        json!({ "order": order, "resting_reason": reason }),
                    ),
                    (
                        EVENT_EXECUTION_PORTFOLIO_CHANGED,
                        json!({ "snapshot": self.snapshot() }),
                    ),
                ],
            };
        }

        let requested_quantity = request.quantity;
        let max_fill_quantity = request
            .max_fill_quantity
            .unwrap_or(requested_quantity)
            .max(0.0);
        let fill_quantity = requested_quantity.min(max_fill_quantity);

        if fill_quantity <= 0.0 {
            return self.reject_order(
                request,
                source_event_sequence,
                ts_ms,
                "本地模拟流动性为 0".to_string(),
            );
        }
        if fill_quantity + f64::EPSILON < requested_quantity {
            if matches!(request.time_in_force, Some(V4SimulatedTimeInForce::Fok)) {
                return self.reject_order(
                    request,
                    source_event_sequence,
                    ts_ms,
                    "FOK 订单无法被本地模拟流动性完全成交".to_string(),
                );
            }
            if !request.allow_partial_fill {
                return self.reject_order(
                    request,
                    source_event_sequence,
                    ts_ms,
                    "当前本地模拟订单未启用部分成交".to_string(),
                );
            }
        }

        let mut order = self.accepted_order(&request, source_event_sequence, ts_ms);
        let acknowledged_order = order.clone();
        let fill_price = compute_simulated_fill_price(&request, side);
        let notional = fill_quantity * fill_price;
        let fee = notional * request.fee_bps.max(0.0) / 10_000.0;
        order.filled_quantity = fill_quantity;
        order.remaining_quantity = (requested_quantity - fill_quantity).max(0.0);
        order.fill_price = Some(fill_price);
        order.status = if order.remaining_quantity > 1e-9 {
            V4SimulatedOrderStatus::PartiallyFilled
        } else {
            V4SimulatedOrderStatus::Filled
        };

        let fill = V4SimulatedFill {
            fill_id: format!("fill-{}-{}", order.order_id, self.fills.len() + 1),
            order_id: order.order_id.clone(),
            venue_id: order.venue_id.clone(),
            symbol: order.symbol.clone(),
            side,
            action: request.action,
            quantity: fill_quantity,
            price: fill_price,
            notional,
            fee,
            fee_asset: self.config.quote_asset.clone(),
            ts_ms,
        };

        self.apply_fill_to_ledger(&fill);
        self.orders.push(order.clone());
        self.fills.push(fill.clone());
        self.record_asset_point(ts_ms);

        let mut events = vec![(
            EVENT_EXECUTION_ORDER_ACKNOWLEDGED,
            json!({ "order": acknowledged_order }),
        )];
        if order.status == V4SimulatedOrderStatus::PartiallyFilled {
            events.push((
                EVENT_EXECUTION_ORDER_PARTIALLY_FILLED,
                json!({ "order": order.clone(), "fill": fill.clone() }),
            ));
        } else {
            events.push((
                EVENT_EXECUTION_ORDER_FILLED,
                json!({ "order": order.clone(), "fill": fill.clone() }),
            ));
        }
        events.push((
            EVENT_EXECUTION_FEE_CHARGED,
            json!({ "order_id": fill.order_id, "fee": fill.fee, "fee_asset": fill.fee_asset }),
        ));
        events.push((
            EVENT_EXECUTION_PORTFOLIO_CHANGED,
            json!({ "snapshot": self.snapshot() }),
        ));
        V4SimulatedExecutionOutcome { events }
    }

    fn submit_oco_bracket(
        &mut self,
        request: V4SimulatedOrderRequest,
        source_event_sequence: u64,
        ts_ms: u64,
    ) -> V4SimulatedExecutionOutcome {
        let (Some(take_profit_price), Some(stop_loss_price)) =
            (request.take_profit_price, request.stop_loss_price)
        else {
            return self.reject_order(
                request,
                source_event_sequence,
                ts_ms,
                "OCO bracket requires take_profit_price and stop_loss_price".to_string(),
            );
        };

        let parent = self.accepted_order(&request, source_event_sequence, ts_ms);
        let group_id = parent.order_id.clone();
        let mut take_profit = self.accepted_order(
            &V4SimulatedOrderRequest {
                order_id: Some(format!("{}-take-profit", group_id)),
                order_type: V4SimulatedOrderType::TakeProfitMarket,
                trigger_price: Some(take_profit_price),
                ..request.clone()
            },
            source_event_sequence,
            ts_ms,
        );
        take_profit.parent_order_id = Some(parent.order_id.clone());
        take_profit.oco_group_id = Some(group_id.clone());
        let mut stop_loss = self.accepted_order(
            &V4SimulatedOrderRequest {
                order_id: Some(format!("{}-stop-loss", group_id)),
                order_type: V4SimulatedOrderType::StopMarket,
                trigger_price: Some(stop_loss_price),
                ..request
            },
            source_event_sequence,
            ts_ms,
        );
        stop_loss.parent_order_id = Some(parent.order_id.clone());
        stop_loss.oco_group_id = Some(group_id);

        self.orders.push(parent.clone());
        self.orders.push(take_profit.clone());
        self.orders.push(stop_loss.clone());
        self.record_asset_point(ts_ms);

        V4SimulatedExecutionOutcome {
            events: vec![
                (
                    EVENT_EXECUTION_ORDER_ACKNOWLEDGED,
                    json!({
                        "order": parent,
                        "oco_legs": [take_profit, stop_loss],
                        "resting_reason": "OCO bracket registered as linked take-profit and stop-loss legs",
                    }),
                ),
                (
                    EVENT_EXECUTION_PORTFOLIO_CHANGED,
                    json!({ "snapshot": self.snapshot() }),
                ),
            ],
        }
    }

    fn amend_order(
        &mut self,
        order_id: &str,
        new_reference_price: Option<f64>,
        new_limit_price: Option<f64>,
        new_trigger_price: Option<f64>,
        new_quantity: Option<f64>,
        ts_ms: u64,
    ) -> V4SimulatedExecutionOutcome {
        let Some(index) = self
            .orders
            .iter()
            .position(|order| order.order_id == order_id)
        else {
            let request = V4SimulatedOrderRequest {
                order_id: Some(order_id.to_string()),
                client_order_id: None,
                venue_id: self.config.default_venue_id.clone(),
                symbol: self.config.default_symbol.clone(),
                action: V4SimulatedPositionAction::Buy,
                order_type: V4SimulatedOrderType::Limit,
                quantity: self.config.default_quantity,
                reference_price: self.config.default_price,
                limit_price: None,
                trigger_price: None,
                take_profit_price: None,
                stop_loss_price: None,
                trailing_offset_bps: None,
                expire_at_ms: None,
                time_in_force: None,
                post_only: false,
                reduce_only: false,
                close_only: false,
                allow_partial_fill: true,
                fee_bps: self.config.default_fee_bps,
                slippage_bps: self.config.default_slippage_bps,
                max_fill_quantity: None,
            };
            return self.reject_order(
                request,
                0,
                ts_ms,
                format!("amend target order `{order_id}` not found"),
            );
        };

        let mut order = self.orders[index].clone();
        if order.status != V4SimulatedOrderStatus::Accepted {
            return V4SimulatedExecutionOutcome {
                events: self.reject_existing_order(
                    index,
                    "cancel-replace-amend only supports open accepted orders".to_string(),
                    ts_ms,
                ),
            };
        }
        if let Some(value) = new_reference_price {
            if !value.is_finite() || value <= 0.0 {
                return V4SimulatedExecutionOutcome {
                    events: self.reject_existing_order(
                        index,
                        "new_reference_price must be finite and positive".to_string(),
                        ts_ms,
                    ),
                };
            }
            order.reference_price = value;
        }
        if let Some(value) = new_limit_price {
            if !value.is_finite() || value <= 0.0 {
                return V4SimulatedExecutionOutcome {
                    events: self.reject_existing_order(
                        index,
                        "new_limit_price must be finite and positive".to_string(),
                        ts_ms,
                    ),
                };
            }
            order.limit_price = Some(value);
        }
        if let Some(value) = new_trigger_price {
            if !value.is_finite() || value <= 0.0 {
                return V4SimulatedExecutionOutcome {
                    events: self.reject_existing_order(
                        index,
                        "new_trigger_price must be finite and positive".to_string(),
                        ts_ms,
                    ),
                };
            }
            order.trigger_price = Some(value);
        }
        if let Some(value) = new_quantity {
            if !value.is_finite() || value <= 0.0 || value + f64::EPSILON < order.filled_quantity {
                return V4SimulatedExecutionOutcome {
                    events: self.reject_existing_order(
                        index,
                        "new_quantity must be finite, positive, and not below filled quantity"
                            .to_string(),
                        ts_ms,
                    ),
                };
            }
            order.requested_quantity = value;
            order.remaining_quantity = (value - order.filled_quantity).max(0.0);
        }
        order.amend_revision = order.amend_revision.saturating_add(1);
        order.ts_ms = ts_ms;
        self.orders[index] = order.clone();
        self.record_asset_point(ts_ms);

        V4SimulatedExecutionOutcome {
            events: vec![
                (
                    EVENT_EXECUTION_ORDER_AMENDED,
                    json!({ "order": order, "reason": "cancel_replace_amend" }),
                ),
                (
                    EVENT_EXECUTION_PORTFOLIO_CHANGED,
                    json!({ "snapshot": self.snapshot() }),
                ),
            ],
        }
    }

    fn reject_order(
        &mut self,
        request: V4SimulatedOrderRequest,
        source_event_sequence: u64,
        ts_ms: u64,
        reason: String,
    ) -> V4SimulatedExecutionOutcome {
        let mut order = self.accepted_order(&request, source_event_sequence, ts_ms);
        order.status = V4SimulatedOrderStatus::Rejected;
        order.rejection_reason = Some(reason.clone());
        order.remaining_quantity = order.requested_quantity;
        self.rejected_order_count += 1;
        self.orders.push(order.clone());
        self.record_asset_point(ts_ms);

        V4SimulatedExecutionOutcome {
            events: vec![
                (
                    EVENT_EXECUTION_ORDER_REJECTED,
                    json!({ "order": order, "reason": reason }),
                ),
                (
                    EVENT_EXECUTION_PORTFOLIO_CHANGED,
                    json!({ "snapshot": self.snapshot() }),
                ),
            ],
        }
    }

    fn expire_orders(&mut self, now_ms: u64) -> V4SimulatedExecutionOutcome {
        let mut events = Vec::new();
        for index in 0..self.orders.len() {
            let Some(expire_at_ms) = self.orders[index].expire_at_ms else {
                continue;
            };
            if self.orders[index].status != V4SimulatedOrderStatus::Accepted
                || expire_at_ms > now_ms
            {
                continue;
            }
            let mut order = self.orders[index].clone();
            order.status = V4SimulatedOrderStatus::Expired;
            order.ts_ms = now_ms;
            self.orders[index] = order.clone();
            events.push((
                EVENT_EXECUTION_ORDER_EXPIRED,
                json!({ "order": order, "expire_at_ms": expire_at_ms }),
            ));
        }
        if !events.is_empty() {
            self.record_asset_point(now_ms);
            events.push((
                EVENT_EXECUTION_PORTFOLIO_CHANGED,
                json!({ "snapshot": self.snapshot() }),
            ));
        }
        V4SimulatedExecutionOutcome { events }
    }

    fn update_market_price(
        &mut self,
        venue_id: &str,
        symbol: &str,
        price: f64,
        ts_ms: u64,
    ) -> V4SimulatedExecutionOutcome {
        self.market_prices
            .insert((venue_id.to_string(), symbol.to_string()), price);
        if let Some(position) = self
            .positions
            .get_mut(&(venue_id.to_string(), symbol.to_string()))
        {
            position.market_price = price;
            position.market_value = position.net_quantity * price;
        }
        self.record_asset_point(ts_ms);
        let mut events = Vec::new();
        events.extend(self.expire_orders(ts_ms).events);
        events.extend(self.check_order_triggers(venue_id, symbol, price, ts_ms));
        events.push((
            EVENT_EXECUTION_PORTFOLIO_CHANGED,
            json!({
                "market_price": {
                    "venue_id": venue_id,
                    "symbol": symbol,
                    "price": price,
                },
                "snapshot": self.snapshot(),
            }),
        ));

        V4SimulatedExecutionOutcome { events }
    }

    fn check_order_triggers(
        &mut self,
        venue_id: &str,
        symbol: &str,
        price: f64,
        ts_ms: u64,
    ) -> Vec<(&'static str, Value)> {
        let mut events = Vec::new();
        for index in 0..self.orders.len() {
            let Some(order) = self.orders.get(index).cloned() else {
                continue;
            };
            if order.status != V4SimulatedOrderStatus::Accepted
                || order.venue_id != venue_id
                || order.symbol != symbol
            {
                continue;
            }

            if order.order_type == V4SimulatedOrderType::Limit {
                let request = self.request_from_order(&order, V4SimulatedOrderType::Limit, price);
                if limit_order_is_marketable(&request, order.side) {
                    events.extend(self.fill_existing_order(
                        index,
                        request,
                        "resting_limit_marketable",
                        ts_ms,
                    ));
                }
                continue;
            }

            if order.order_type == V4SimulatedOrderType::TrailingStop {
                events.extend(self.update_trailing_stop(index, price, ts_ms));
            }
            let Some(order) = self.orders.get(index).cloned() else {
                continue;
            };

            if !is_conditional_order_type(order.order_type) {
                continue;
            }
            let Some(trigger_price) = order.trigger_price else {
                events.extend(self.reject_existing_order(
                    index,
                    "条件单缺少 trigger_price".to_string(),
                    ts_ms,
                ));
                continue;
            };
            if !conditional_order_is_triggered(&order, price, trigger_price) {
                continue;
            }

            let converted_order_type = conditional_order_execution_type(order.order_type);
            let request = self.request_from_order(&order, converted_order_type, price);
            events.push((
                EVENT_EXECUTION_CONDITIONAL_ORDER_TRIGGERED,
                json!({
                    "order_id": order.order_id,
                    "order_type": order.order_type,
                    "converted_order_type": converted_order_type,
                    "trigger_price": trigger_price,
                    "market_price": price,
                }),
            ));

            if converted_order_type == V4SimulatedOrderType::Limit
                && !limit_order_is_marketable(&request, order.side)
            {
                let mut converted = order;
                converted.order_type = V4SimulatedOrderType::Limit;
                converted.reference_price = price;
                converted.ts_ms = ts_ms;
                self.orders[index] = converted.clone();
                events.push((
                    EVENT_EXECUTION_ORDER_ACKNOWLEDGED,
                    json!({
                        "order": converted,
                        "resting_reason": "条件单已触发并转换为限价挂单",
                    }),
                ));
                continue;
            }

            events.extend(self.fill_existing_order(index, request, "conditional_trigger", ts_ms));
        }
        events
    }

    fn update_trailing_stop(
        &mut self,
        index: usize,
        price: f64,
        ts_ms: u64,
    ) -> Vec<(&'static str, Value)> {
        let Some(mut order) = self.orders.get(index).cloned() else {
            return Vec::new();
        };
        if order.status != V4SimulatedOrderStatus::Accepted
            || order.order_type != V4SimulatedOrderType::TrailingStop
        {
            return Vec::new();
        }
        let Some(offset_bps) = order.trailing_offset_bps else {
            return Vec::new();
        };
        let offset = offset_bps / 10_000.0;
        let old_trigger = order.trigger_price;
        match order.side {
            V4SimulatedOrderSide::Sell => {
                let peak = order
                    .trailing_peak_price
                    .unwrap_or(order.reference_price)
                    .max(price);
                order.trailing_peak_price = Some(peak);
                let next_trigger = peak * (1.0 - offset);
                order.trigger_price = Some(
                    order
                        .trigger_price
                        .unwrap_or(next_trigger)
                        .max(next_trigger),
                );
            }
            V4SimulatedOrderSide::Buy => {
                let trough = order
                    .trailing_trough_price
                    .unwrap_or(order.reference_price)
                    .min(price);
                order.trailing_trough_price = Some(trough);
                let next_trigger = trough * (1.0 + offset);
                order.trigger_price = Some(
                    order
                        .trigger_price
                        .unwrap_or(next_trigger)
                        .min(next_trigger),
                );
            }
        }
        order.reference_price = price;
        order.ts_ms = ts_ms;
        if old_trigger == order.trigger_price {
            self.orders[index] = order;
            return Vec::new();
        }
        self.orders[index] = order.clone();
        vec![(
            EVENT_EXECUTION_ORDER_AMENDED,
            json!({
                "order": order,
                "reason": "trailing_stop_adjusted",
                "previous_trigger_price": old_trigger,
            }),
        )]
    }

    fn request_from_order(
        &self,
        order: &V4SimulatedOrder,
        order_type: V4SimulatedOrderType,
        reference_price: f64,
    ) -> V4SimulatedOrderRequest {
        V4SimulatedOrderRequest {
            order_id: Some(order.order_id.clone()),
            client_order_id: order.client_order_id.clone(),
            venue_id: order.venue_id.clone(),
            symbol: order.symbol.clone(),
            action: order.action,
            order_type,
            quantity: order.remaining_quantity.max(0.0),
            reference_price,
            limit_price: order.limit_price,
            trigger_price: order.trigger_price,
            take_profit_price: order.take_profit_price,
            stop_loss_price: order.stop_loss_price,
            trailing_offset_bps: order.trailing_offset_bps,
            expire_at_ms: order.expire_at_ms,
            time_in_force: order.time_in_force,
            post_only: false,
            reduce_only: order.action.is_reducing(),
            close_only: matches!(
                order.action,
                V4SimulatedPositionAction::CloseLong | V4SimulatedPositionAction::CloseShort
            ),
            allow_partial_fill: true,
            fee_bps: order.fee_bps,
            slippage_bps: order.slippage_bps,
            max_fill_quantity: None,
        }
    }

    fn fill_existing_order(
        &mut self,
        index: usize,
        request: V4SimulatedOrderRequest,
        trigger_reason: &'static str,
        ts_ms: u64,
    ) -> Vec<(&'static str, Value)> {
        if let Err(reason) = validate_simulated_order_request(&request) {
            return self.reject_existing_order(index, reason, ts_ms);
        }
        if let Err(reason) = self.validate_position_action(&request) {
            return self.reject_existing_order(index, reason, ts_ms);
        }
        let side = request.action.side();
        if let Some(reason) = self.pre_execution_rejection_reason(&request, side) {
            return self.reject_existing_order(index, reason, ts_ms);
        }

        let Some(mut order) = self.orders.get(index).cloned() else {
            return Vec::new();
        };
        let fill_quantity = request.quantity;
        if fill_quantity <= 0.0 {
            return self.reject_existing_order(index, "本地模拟流动性为 0".to_string(), ts_ms);
        }

        let fill_price = compute_simulated_fill_price(&request, side);
        let notional = fill_quantity * fill_price;
        let fee = notional * request.fee_bps.max(0.0) / 10_000.0;
        order.filled_quantity += fill_quantity;
        order.remaining_quantity = (order.remaining_quantity - fill_quantity).max(0.0);
        order.fill_price = Some(fill_price);
        order.status = if order.remaining_quantity > 1e-9 {
            V4SimulatedOrderStatus::PartiallyFilled
        } else {
            V4SimulatedOrderStatus::Filled
        };
        order.reference_price = request.reference_price;
        order.ts_ms = ts_ms;

        let fill = V4SimulatedFill {
            fill_id: format!("fill-{}-{}", order.order_id, self.fills.len() + 1),
            order_id: order.order_id.clone(),
            venue_id: order.venue_id.clone(),
            symbol: order.symbol.clone(),
            side,
            action: order.action,
            quantity: fill_quantity,
            price: fill_price,
            notional,
            fee,
            fee_asset: self.config.quote_asset.clone(),
            ts_ms,
        };

        self.apply_fill_to_ledger(&fill);
        self.orders[index] = order.clone();
        self.fills.push(fill.clone());
        self.record_asset_point(ts_ms);

        let mut events = Vec::new();
        if order.status == V4SimulatedOrderStatus::PartiallyFilled {
            events.push((
                EVENT_EXECUTION_ORDER_PARTIALLY_FILLED,
                json!({ "order": order.clone(), "fill": fill.clone(), "trigger_reason": trigger_reason }),
            ));
        } else {
            events.push((
                EVENT_EXECUTION_ORDER_FILLED,
                json!({ "order": order.clone(), "fill": fill.clone(), "trigger_reason": trigger_reason }),
            ));
        }
        events.push((
            EVENT_EXECUTION_FEE_CHARGED,
            json!({ "order_id": fill.order_id, "fee": fill.fee, "fee_asset": fill.fee_asset }),
        ));
        if order.status == V4SimulatedOrderStatus::Filled {
            events.extend(self.cancel_oco_siblings(&order, ts_ms));
        }
        events.push((
            EVENT_EXECUTION_PORTFOLIO_CHANGED,
            json!({ "snapshot": self.snapshot() }),
        ));
        events
    }

    fn cancel_oco_siblings(
        &mut self,
        filled_order: &V4SimulatedOrder,
        ts_ms: u64,
    ) -> Vec<(&'static str, Value)> {
        let Some(group_id) = filled_order
            .oco_group_id
            .clone()
            .or_else(|| filled_order.parent_order_id.clone())
        else {
            return Vec::new();
        };
        let mut events = Vec::new();
        for index in 0..self.orders.len() {
            let should_cancel = {
                let order = &self.orders[index];
                order.status == V4SimulatedOrderStatus::Accepted
                    && order.order_id != filled_order.order_id
                    && (order.order_id == group_id
                        || order.oco_group_id.as_deref() == Some(group_id.as_str())
                        || order.parent_order_id.as_deref() == Some(group_id.as_str()))
            };
            if !should_cancel {
                continue;
            }
            let mut order = self.orders[index].clone();
            order.status = V4SimulatedOrderStatus::Canceled;
            order.ts_ms = ts_ms;
            self.orders[index] = order.clone();
            events.push((
                EVENT_EXECUTION_ORDER_CANCELED,
                json!({
                    "order": order,
                    "reason": "oco_sibling_filled",
                    "filled_order_id": filled_order.order_id,
                    "oco_group_id": group_id.clone(),
                }),
            ));
        }
        events
    }

    fn reject_existing_order(
        &mut self,
        index: usize,
        reason: String,
        ts_ms: u64,
    ) -> Vec<(&'static str, Value)> {
        let Some(mut order) = self.orders.get(index).cloned() else {
            return Vec::new();
        };
        order.status = V4SimulatedOrderStatus::Rejected;
        order.rejection_reason = Some(reason.clone());
        order.ts_ms = ts_ms;
        self.orders[index] = order.clone();
        self.rejected_order_count += 1;
        self.record_asset_point(ts_ms);
        vec![
            (
                EVENT_EXECUTION_ORDER_REJECTED,
                json!({ "order": order, "reason": reason }),
            ),
            (
                EVENT_EXECUTION_PORTFOLIO_CHANGED,
                json!({ "snapshot": self.snapshot() }),
            ),
        ]
    }

    fn accepted_order(
        &mut self,
        request: &V4SimulatedOrderRequest,
        source_event_sequence: u64,
        ts_ms: u64,
    ) -> V4SimulatedOrder {
        let order_id = request.order_id.clone().unwrap_or_else(|| {
            self.order_sequence += 1;
            format!("v4-sim-order-{}", self.order_sequence)
        });
        V4SimulatedOrder {
            order_id,
            client_order_id: request.client_order_id.clone(),
            venue_id: request.venue_id.clone(),
            symbol: request.symbol.clone(),
            action: request.action,
            side: request.action.side(),
            order_type: request.order_type,
            time_in_force: request.time_in_force,
            requested_quantity: request.quantity,
            filled_quantity: 0.0,
            remaining_quantity: request.quantity,
            reference_price: request.reference_price,
            limit_price: request.limit_price,
            trigger_price: request.trigger_price,
            take_profit_price: request.take_profit_price,
            stop_loss_price: request.stop_loss_price,
            trailing_offset_bps: request.trailing_offset_bps,
            expire_at_ms: request.expire_at_ms,
            parent_order_id: None,
            oco_group_id: None,
            trailing_peak_price: if request.order_type == V4SimulatedOrderType::TrailingStop {
                Some(request.reference_price)
            } else {
                None
            },
            trailing_trough_price: if request.order_type == V4SimulatedOrderType::TrailingStop {
                Some(request.reference_price)
            } else {
                None
            },
            amend_revision: 0,
            fill_price: None,
            status: V4SimulatedOrderStatus::Accepted,
            rejection_reason: None,
            fee_bps: request.fee_bps,
            slippage_bps: request.slippage_bps,
            ts_ms,
            source_event_sequence,
        }
    }

    fn validate_position_action(&self, request: &V4SimulatedOrderRequest) -> Result<(), String> {
        let current_qty = self
            .positions
            .get(&(request.venue_id.clone(), request.symbol.clone()))
            .map(|position| position.net_quantity)
            .unwrap_or(0.0);

        match request.action {
            V4SimulatedPositionAction::CloseLong => {
                if current_qty <= 0.0 {
                    return Err("close_long 需要已有多头持仓".to_string());
                }
                if request.quantity > current_qty + f64::EPSILON && !request.allow_partial_fill {
                    return Err("close_long 数量超过已有多头持仓，且未启用部分成交".to_string());
                }
            }
            V4SimulatedPositionAction::CloseShort => {
                if current_qty >= 0.0 {
                    return Err("close_short 需要已有空头持仓".to_string());
                }
                if request.quantity > current_qty.abs() + f64::EPSILON
                    && !request.allow_partial_fill
                {
                    return Err("close_short 数量超过已有空头持仓，且未启用部分成交".to_string());
                }
            }
            V4SimulatedPositionAction::Sell => {
                if current_qty <= 0.0 && (request.reduce_only || request.close_only) {
                    return Err("卖出 reduce_only/close_only 需要已有多头持仓".to_string());
                }
            }
            V4SimulatedPositionAction::Buy => {
                if current_qty >= 0.0 && request.close_only {
                    return Err("buy close_only 需要已有空头持仓".to_string());
                }
            }
            V4SimulatedPositionAction::OpenLong | V4SimulatedPositionAction::OpenShort => {}
        }

        if request.reduce_only {
            match request.action {
                V4SimulatedPositionAction::OpenLong | V4SimulatedPositionAction::OpenShort => {
                    return Err("reduce_only 不能打开新持仓".to_string());
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn pre_execution_rejection_reason(
        &self,
        request: &V4SimulatedOrderRequest,
        side: V4SimulatedOrderSide,
    ) -> Option<String> {
        if request.post_only
            && request.order_type == V4SimulatedOrderType::Limit
            && limit_order_is_marketable(request, side)
        {
            return Some("post_only 限价单会在本地模拟盘口中主动吃单".to_string());
        }
        None
    }

    fn non_executable_resting_reason(
        &self,
        request: &V4SimulatedOrderRequest,
        side: V4SimulatedOrderSide,
    ) -> Option<String> {
        match request.order_type {
            V4SimulatedOrderType::Market => None,
            V4SimulatedOrderType::Limit => {
                if limit_order_is_marketable(request, side) {
                    None
                } else {
                    Some("限价单已登记为挂单；当前本地 runtime 路径尚未接入 open-order trigger engine".to_string())
                }
            }
            V4SimulatedOrderType::StopMarket
            | V4SimulatedOrderType::StopLimit
            | V4SimulatedOrderType::TakeProfitMarket
            | V4SimulatedOrderType::TakeProfitLimit
            | V4SimulatedOrderType::OcoBracket
            | V4SimulatedOrderType::TrailingStop => {
                if request.trigger_price.is_some() {
                    Some("条件单已登记；当前本地 runtime 路径尚未接入 trigger engine".to_string())
                } else {
                    Some("条件单在本地模拟成交前需要提供 trigger_price".to_string())
                }
            }
        }
    }

    fn apply_fill_to_ledger(&mut self, fill: &V4SimulatedFill) {
        match fill.side {
            V4SimulatedOrderSide::Buy => {
                self.cash_balance -= fill.notional + fill.fee;
            }
            V4SimulatedOrderSide::Sell => {
                self.cash_balance += fill.notional - fill.fee;
            }
        }
        self.realized_fees += fill.fee;

        let key = (fill.venue_id.clone(), fill.symbol.clone());
        let position = self
            .positions
            .entry(key)
            .or_insert_with(|| V4SimulatedPosition {
                venue_id: fill.venue_id.clone(),
                symbol: fill.symbol.clone(),
                net_quantity: 0.0,
                average_price: 0.0,
                market_price: fill.price,
                market_value: 0.0,
            });
        let old_qty = position.net_quantity;
        let signed_qty = match fill.action {
            V4SimulatedPositionAction::Buy
            | V4SimulatedPositionAction::OpenLong
            | V4SimulatedPositionAction::CloseShort => fill.quantity,
            V4SimulatedPositionAction::Sell
            | V4SimulatedPositionAction::OpenShort
            | V4SimulatedPositionAction::CloseLong => -fill.quantity,
        };
        let new_qty = old_qty + signed_qty;
        if old_qty.signum() == signed_qty.signum() || old_qty.abs() <= f64::EPSILON {
            let old_notional = old_qty.abs() * position.average_price;
            let added_notional = fill.quantity * fill.price;
            let total_qty = old_qty.abs() + fill.quantity;
            position.average_price = if total_qty > 0.0 {
                (old_notional + added_notional) / total_qty
            } else {
                0.0
            };
        } else if new_qty.abs() <= f64::EPSILON {
            position.average_price = 0.0;
        }
        position.net_quantity = if new_qty.abs() <= 1e-9 { 0.0 } else { new_qty };
        position.market_price = fill.price;
        position.market_value = position.net_quantity * fill.price;
    }

    fn record_asset_point(&mut self, ts_ms: u64) {
        let point = self.asset_point(ts_ms);
        self.asset_curve.push(point);
        if self.asset_curve.len() > 256 {
            let overflow = self.asset_curve.len() - 256;
            self.asset_curve.drain(0..overflow);
        }
    }

    fn asset_point(&self, ts_ms: u64) -> V4SimulatedAssetPoint {
        let position_market_value = self
            .positions
            .values()
            .map(|position| position.market_value)
            .sum::<f64>();
        V4SimulatedAssetPoint {
            ts_ms,
            cash_balance: round_money(self.cash_balance),
            position_market_value: round_money(position_market_value),
            portfolio_value: round_money(self.cash_balance + position_market_value),
        }
    }

    fn snapshot(&self) -> V4SimulatedExecutionSnapshot {
        let point = self.asset_point(
            self.asset_curve
                .last()
                .map(|item| item.ts_ms)
                .unwrap_or_default(),
        );
        V4SimulatedExecutionSnapshot {
            enabled: true,
            quote_asset: self.config.quote_asset.clone(),
            cash_balance: point.cash_balance,
            realized_fees: round_money(self.realized_fees),
            position_market_value: point.position_market_value,
            portfolio_value: point.portfolio_value,
            order_count: self.orders.len() as u64,
            open_order_count: self
                .orders
                .iter()
                .filter(|order| order.status == V4SimulatedOrderStatus::Accepted)
                .count() as u64,
            rejected_order_count: self.rejected_order_count,
            fill_count: self.fills.len() as u64,
            positions: self.positions.values().cloned().collect(),
            asset_curve: self.asset_curve.clone(),
            last_order: self.orders.last().cloned(),
            last_fill: self.fills.last().cloned(),
        }
    }

    fn microstructure_metrics(&self) -> qrpc_core_ir::v4::V4BacktestMicrostructureMetrics {
        let orders = self
            .orders
            .iter()
            .map(|order| MicrostructureOrderSample {
                requested_quantity: order.requested_quantity,
                filled_quantity: order.filled_quantity,
                reference_price: order.reference_price,
                is_open: order.status == V4SimulatedOrderStatus::Accepted,
            })
            .collect::<Vec<_>>();
        let fills = self
            .fills
            .iter()
            .filter_map(|fill| {
                let reference_price = self
                    .orders
                    .iter()
                    .find(|order| order.order_id == fill.order_id)
                    .map(|order| order.reference_price)
                    .unwrap_or(fill.price);
                (reference_price > 0.0).then_some(MicrostructureFillSample {
                    quantity: fill.quantity,
                    price: fill.price,
                    reference_price,
                })
            })
            .collect::<Vec<_>>();
        compute_microstructure_metrics(&orders, &fills)
    }
}

fn transition_source_matches(
    expected_source: Option<&str>,
    event: &V4RuntimeEventEnvelope,
) -> bool {
    expected_source
        .map(|source| source == event.source)
        .unwrap_or(true)
}

fn transition_freshness_matches(
    freshness: Option<EventFreshnessRequirement>,
    _event: &V4RuntimeEventEnvelope,
) -> bool {
    matches!(
        freshness,
        None | Some(EventFreshnessRequirement::FreshOnly)
            | Some(EventFreshnessRequirement::FreshOrStale)
            | Some(EventFreshnessRequirement::RecoveringAllowed)
    )
}

fn validate_payload_field_type(
    field: &MachineEventPayloadField,
    value: &Value,
) -> Result<(), String> {
    let type_name = field.type_name.trim().to_ascii_lowercase();
    let ok = match type_name.as_str() {
        "string" | "symbol" | "venue" | "account" | "side" | "position_side" | "order_type"
        | "time_in_force" | "freshness" | "runtime_mode" | "order_permission" => value.is_string(),
        "bool" | "boolean" => value.is_boolean(),
        "u64" | "uint" => value.as_u64().is_some(),
        "i64" | "int" | "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "f64" | "decimal" | "number" | "price" | "quantity" | "notional" | "percent" | "ratio"
        | "fee" | "slippage" | "leverage" => value.as_f64().is_some(),
        "object" | "map" => value.is_object(),
        "array" | "list" => value.is_array(),
        other => return Err(format!("unsupported catalog type `{}`", other)),
    };

    if ok {
        Ok(())
    } else {
        Err(format!(
            "expected `{}`, got {}",
            field.type_name,
            payload_type_label(value)
        ))
    }
}

fn payload_type_label(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(number) if number.is_i64() => "i64",
        Value::Number(number) if number.is_u64() => "u64",
        Value::Number(_) => "f64",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[allow(dead_code)]
fn recovery_policy_allows_async(policy: &MachineRecoveryPolicy) -> bool {
    matches!(policy, MachineRecoveryPolicy::AsyncRecover)
}

impl V4SimulatedPositionAction {
    fn side(self) -> V4SimulatedOrderSide {
        match self {
            V4SimulatedPositionAction::Buy
            | V4SimulatedPositionAction::OpenLong
            | V4SimulatedPositionAction::CloseShort => V4SimulatedOrderSide::Buy,
            V4SimulatedPositionAction::Sell
            | V4SimulatedPositionAction::OpenShort
            | V4SimulatedPositionAction::CloseLong => V4SimulatedOrderSide::Sell,
        }
    }
    fn is_reducing(self) -> bool {
        matches!(
            self,
            V4SimulatedPositionAction::CloseLong | V4SimulatedPositionAction::CloseShort
        )
    }
}

fn is_conditional_order_type(order_type: V4SimulatedOrderType) -> bool {
    matches!(
        order_type,
        V4SimulatedOrderType::StopMarket
            | V4SimulatedOrderType::StopLimit
            | V4SimulatedOrderType::TakeProfitMarket
            | V4SimulatedOrderType::TakeProfitLimit
            | V4SimulatedOrderType::TrailingStop
    )
}

fn conditional_order_execution_type(order_type: V4SimulatedOrderType) -> V4SimulatedOrderType {
    match order_type {
        V4SimulatedOrderType::StopMarket | V4SimulatedOrderType::TakeProfitMarket => {
            V4SimulatedOrderType::Market
        }
        V4SimulatedOrderType::TrailingStop => V4SimulatedOrderType::Market,
        V4SimulatedOrderType::StopLimit | V4SimulatedOrderType::TakeProfitLimit => {
            V4SimulatedOrderType::Limit
        }
        other => other,
    }
}

fn conditional_order_is_triggered(
    order: &V4SimulatedOrder,
    market_price: f64,
    trigger_price: f64,
) -> bool {
    match order.order_type {
        V4SimulatedOrderType::StopMarket
        | V4SimulatedOrderType::StopLimit
        | V4SimulatedOrderType::TrailingStop => match order.side {
            V4SimulatedOrderSide::Buy => market_price >= trigger_price,
            V4SimulatedOrderSide::Sell => market_price <= trigger_price,
        },
        V4SimulatedOrderType::TakeProfitMarket | V4SimulatedOrderType::TakeProfitLimit => {
            match order.side {
                V4SimulatedOrderSide::Buy => market_price <= trigger_price,
                V4SimulatedOrderSide::Sell => market_price >= trigger_price,
            }
        }
        _ => false,
    }
}

fn validate_simulated_execution_config(config: &V4SimulatedExecutionConfig) -> Result<()> {
    if !config.starting_cash.is_finite() {
        return Err(anyhow!("模拟执行 starting_cash 必须是有限数"));
    }
    if config.quote_asset.trim().is_empty() {
        return Err(anyhow!("模拟执行 quote_asset 不能为空"));
    }
    if config.default_venue_id.trim().is_empty() {
        return Err(anyhow!("模拟执行 default_venue_id 不能为空"));
    }
    if config.default_symbol.trim().is_empty() {
        return Err(anyhow!("模拟执行 default_symbol 不能为空"));
    }
    if !config.default_quantity.is_finite() || config.default_quantity <= 0.0 {
        return Err(anyhow!("模拟执行 default_quantity 必须是有限数且大于 0"));
    }
    if !config.default_price.is_finite() || config.default_price <= 0.0 {
        return Err(anyhow!("模拟执行 default_price 必须是有限数且大于 0"));
    }
    Ok(())
}

fn validate_simulated_order_request(request: &V4SimulatedOrderRequest) -> Result<(), String> {
    if request.venue_id.trim().is_empty() {
        return Err("本地模拟订单 venue_id 不能为空".to_string());
    }
    if request.symbol.trim().is_empty() {
        return Err("本地模拟订单 symbol 不能为空".to_string());
    }
    if !request.quantity.is_finite() || request.quantity <= 0.0 {
        return Err("quantity 必须是有限数且大于 0".to_string());
    }
    if !request.reference_price.is_finite() || request.reference_price <= 0.0 {
        return Err("reference_price 必须是有限数且大于 0".to_string());
    }
    if request
        .limit_price
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        return Err("提供 limit_price 时必须是有限数且大于 0".to_string());
    }
    if request
        .trigger_price
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        return Err("提供 trigger_price 时必须是有限数且大于 0".to_string());
    }
    if request
        .take_profit_price
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        return Err("take_profit_price must be finite and positive".to_string());
    }
    if request
        .stop_loss_price
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        return Err("stop_loss_price must be finite and positive".to_string());
    }
    if request
        .trailing_offset_bps
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        return Err("trailing_offset_bps must be finite and positive".to_string());
    }
    if matches!(request.time_in_force, Some(V4SimulatedTimeInForce::Gtd))
        && request.expire_at_ms.is_none()
    {
        return Err("GTD order requires expire_at_ms".to_string());
    }
    if !request.fee_bps.is_finite() || request.fee_bps < 0.0 {
        return Err("fee_bps 必须是有限数且不小于 0".to_string());
    }
    if !request.slippage_bps.is_finite() || request.slippage_bps < 0.0 {
        return Err("slippage_bps 必须是有限数且不小于 0".to_string());
    }
    Ok(())
}

fn simulated_order_required_capabilities(
    request: &V4SimulatedOrderRequest,
) -> BTreeSet<ExecutionCapabilityKind> {
    let mut capabilities = BTreeSet::new();
    capabilities.insert(match request.order_type {
        V4SimulatedOrderType::Market => ExecutionCapabilityKind::Market,
        V4SimulatedOrderType::Limit => ExecutionCapabilityKind::Limit,
        V4SimulatedOrderType::StopMarket => ExecutionCapabilityKind::StopMarket,
        V4SimulatedOrderType::StopLimit => ExecutionCapabilityKind::StopLimit,
        V4SimulatedOrderType::TakeProfitMarket => ExecutionCapabilityKind::TakeProfitMarket,
        V4SimulatedOrderType::TakeProfitLimit => ExecutionCapabilityKind::TakeProfitLimit,
        V4SimulatedOrderType::OcoBracket => ExecutionCapabilityKind::OcoBracket,
        V4SimulatedOrderType::TrailingStop => ExecutionCapabilityKind::TrailingStop,
    });
    if let Some(time_in_force) = request.time_in_force {
        capabilities.insert(match time_in_force {
            V4SimulatedTimeInForce::Gtc => ExecutionCapabilityKind::Gtc,
            V4SimulatedTimeInForce::Ioc => ExecutionCapabilityKind::Ioc,
            V4SimulatedTimeInForce::Fok => ExecutionCapabilityKind::Fok,
            V4SimulatedTimeInForce::Day => ExecutionCapabilityKind::Day,
            V4SimulatedTimeInForce::Gtd => ExecutionCapabilityKind::Gtd,
        });
    }
    match request.action {
        V4SimulatedPositionAction::OpenLong => {
            capabilities.insert(ExecutionCapabilityKind::OpenLong);
        }
        V4SimulatedPositionAction::CloseLong => {
            capabilities.insert(ExecutionCapabilityKind::CloseLong);
        }
        V4SimulatedPositionAction::OpenShort => {
            capabilities.insert(ExecutionCapabilityKind::OpenShort);
        }
        V4SimulatedPositionAction::CloseShort => {
            capabilities.insert(ExecutionCapabilityKind::CloseShort);
        }
        V4SimulatedPositionAction::Buy | V4SimulatedPositionAction::Sell => {}
    }
    if request.post_only {
        capabilities.insert(ExecutionCapabilityKind::PostOnly);
    }
    if request.reduce_only {
        capabilities.insert(ExecutionCapabilityKind::ReduceOnly);
    }
    if request.close_only {
        capabilities.insert(ExecutionCapabilityKind::CloseOnly);
    }
    if request.client_order_id.is_some() {
        capabilities.insert(ExecutionCapabilityKind::ClientOrderId);
    }
    capabilities
}

fn compute_simulated_fill_price(
    request: &V4SimulatedOrderRequest,
    side: V4SimulatedOrderSide,
) -> f64 {
    let base_price = match request.order_type {
        V4SimulatedOrderType::Limit | V4SimulatedOrderType::StopLimit => {
            request.limit_price.unwrap_or(request.reference_price)
        }
        _ => request.reference_price,
    };
    let slippage_ratio = request.slippage_bps.max(0.0) / 10_000.0;
    match side {
        V4SimulatedOrderSide::Buy => base_price * (1.0 + slippage_ratio),
        V4SimulatedOrderSide::Sell => base_price * (1.0 - slippage_ratio),
    }
}

fn limit_order_is_marketable(
    request: &V4SimulatedOrderRequest,
    side: V4SimulatedOrderSide,
) -> bool {
    let limit = request.limit_price.unwrap_or(request.reference_price);
    match side {
        V4SimulatedOrderSide::Buy => request.reference_price <= limit,
        V4SimulatedOrderSide::Sell => request.reference_price >= limit,
    }
}

fn payload_string(payload: &Value, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        payload
            .get(*name)
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string)
    })
}

fn payload_f64(payload: &Value, names: &[&str]) -> Option<f64> {
    names.iter().find_map(|name| {
        let value = payload.get(*name)?;
        value
            .as_f64()
            .or_else(|| value.as_str().and_then(|raw| raw.parse::<f64>().ok()))
    })
}

fn payload_u64(payload: &Value, names: &[&str]) -> Option<u64> {
    names.iter().find_map(|name| {
        let value = payload.get(*name)?;
        value
            .as_u64()
            .or_else(|| value.as_str().and_then(|raw| raw.parse::<u64>().ok()))
    })
}

fn payload_bool(payload: &Value, names: &[&str]) -> Option<bool> {
    names.iter().find_map(|name| {
        let value = payload.get(*name)?;
        value.as_bool().or_else(|| match value.as_str()? {
            "true" | "True" | "TRUE" | "1" => Some(true),
            "false" | "False" | "FALSE" | "0" => Some(false),
            _ => None,
        })
    })
}

fn metadata_string(metadata: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
}

fn parse_position_action(raw: &str) -> Option<V4SimulatedPositionAction> {
    match normalize_token(raw).as_str() {
        "buy" => Some(V4SimulatedPositionAction::Buy),
        "sell" => Some(V4SimulatedPositionAction::Sell),
        "openlong" => Some(V4SimulatedPositionAction::OpenLong),
        "closelong" => Some(V4SimulatedPositionAction::CloseLong),
        "openshort" => Some(V4SimulatedPositionAction::OpenShort),
        "closeshort" => Some(V4SimulatedPositionAction::CloseShort),
        _ => None,
    }
}

fn parse_order_type(raw: &str) -> Option<V4SimulatedOrderType> {
    match normalize_token(raw).as_str() {
        "market" => Some(V4SimulatedOrderType::Market),
        "limit" => Some(V4SimulatedOrderType::Limit),
        "stopmarket" => Some(V4SimulatedOrderType::StopMarket),
        "stoplimit" => Some(V4SimulatedOrderType::StopLimit),
        "takeprofitmarket" => Some(V4SimulatedOrderType::TakeProfitMarket),
        "takeprofitlimit" => Some(V4SimulatedOrderType::TakeProfitLimit),
        "ocobracket" | "oco" => Some(V4SimulatedOrderType::OcoBracket),
        "trailingstop" => Some(V4SimulatedOrderType::TrailingStop),
        _ => None,
    }
}

fn parse_time_in_force(raw: &str) -> Option<V4SimulatedTimeInForce> {
    match normalize_token(raw).as_str() {
        "gtc" => Some(V4SimulatedTimeInForce::Gtc),
        "ioc" => Some(V4SimulatedTimeInForce::Ioc),
        "fok" => Some(V4SimulatedTimeInForce::Fok),
        "day" => Some(V4SimulatedTimeInForce::Day),
        "gtd" => Some(V4SimulatedTimeInForce::Gtd),
        _ => None,
    }
}

fn normalize_token(raw: &str) -> String {
    raw.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn round_money(value: f64) -> f64 {
    (value * 100_000_000.0).round() / 100_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core_ir::v4::{
        bridge_core_ir_to_v4_machine_graph, unsupported_v4_first_wave_matrix,
        CapabilitySupportSource, ExecutionCapabilityKind, MachineActionSpec, MachineEventSelector,
        MachineMemoryField, MachineState, MachineTransition, StateGroup, V4MachineContract,
        V4MachineGraphContract, VenueCapabilityMatrix, V4_COMPAT_CORE_IR_LOADED_EVENT,
        V4_COMPAT_DECISION_MACHINE_ID, V4_COMPAT_EXECUTION_MACHINE_ID,
        V4_COMPAT_OBSERVATION_MACHINE_ID, V4_COMPAT_OBSERVATION_READY_EVENT,
        V4_COMPAT_RISK_APPROVED_EVENT,
    };
    use qrpc_core_ir::{
        moving_average_compare_expr, AgentPolicy, AgentPolicyKind, ComparisonOp, CoreIndicatorKind,
        CoreMetadata, CoreSourceKind, CoreStrategyIr, CoreTimeInForce, DataBinding,
        DataBindingKind, ExecutionRule, ExecutionSizingKind, IndicatorNode, RiskPolicy, SeriesExpr,
        SignalKind, SignalRule,
    };

    fn sample_core_ir_for_v4_runtime() -> CoreStrategyIr {
        let mut core_ir = CoreStrategyIr::new(
            CoreMetadata {
                strategy_id: "runtime.compat.sample".to_string(),
                name: "Runtime Compat Sample".to_string(),
                source_kind: CoreSourceKind::StrategyIr,
            },
            ExecutionRule {
                execution_id: "exec_1".to_string(),
                venue_kind: "paper".to_string(),
                sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
                slippage_bps: 5.0,
                taker_fee_bps: 10.0,
                total_cost_buffer_bps: 20.0,
                time_in_force: CoreTimeInForce::Gtc,
                params: BTreeMap::new(),
            },
        );
        core_ir.data_bindings.push(DataBinding {
            data_id: "btc_1d".to_string(),
            kind: DataBindingKind::KlineSeries,
            source_hints: BTreeMap::new(),
        });
        core_ir.indicators.push(IndicatorNode {
            indicator_id: "ma_cross_1".to_string(),
            kind: CoreIndicatorKind::MaCross,
            inputs: vec![SeriesExpr::DataRef {
                data_id: "btc_1d".to_string(),
            }],
            spread_spec: None,
            custom_expr: None,
            params: BTreeMap::new(),
        });
        core_ir.signal_rules.push(SignalRule {
            signal_id: "signal_1".to_string(),
            indicator_id: "ma_cross_1".to_string(),
            signal_kind: SignalKind::Long,
            condition: moving_average_compare_expr("btc_1d", 20, ComparisonOp::Gt, 100).unwrap(),
        });
        core_ir.agent_policies.push(AgentPolicy {
            agent_id: "agent_1".to_string(),
            name: "Weighted Agent".to_string(),
            kind: AgentPolicyKind::WeightedSignals,
            input_signal_ids: vec!["signal_1".to_string()],
            rebalance_symbols: Vec::new(),
            rebalance_schedule: None,
            rebalance_allocation_kind: None,
            rebalance_rank_method: None,
            rebalance_score_normalize: None,
            rebalance_target_weights: Vec::new(),
            decision_threshold: Some(0.05),
            max_quantity_ratio: 0.2,
            spread_trigger_bps: None,
            enabled: true,
        });
        core_ir.risk_policies.push(RiskPolicy {
            policy_id: "risk_1".to_string(),
            name: "Risk Guard".to_string(),
            observed_agent_ids: vec!["agent_1".to_string()],
            max_position_ratio: 0.3,
            max_single_weight: None,
            max_concentration_ratio: None,
            max_symbol_net_exposure_ratio: None,
            max_portfolio_net_exposure_ratio: None,
            max_turnover: None,
            min_trade_weight: None,
            max_new_positions_per_rebalance: None,
            max_total_leverage: 1.0,
            max_exchange_leverage: 1.0,
            min_action_interval_ms: 1_000,
            enabled: true,
            max_cross_symbol_leverage: None,
        });
        core_ir
    }

    fn sample_compat_graph() -> V4MachineGraphContract {
        let bridge_report = bridge_core_ir_to_v4_machine_graph(&sample_core_ir_for_v4_runtime());
        bridge_report.graph.unwrap()
    }

    fn runtime_simulated_market_matrix() -> VenueCapabilityMatrix {
        let mut matrix = unsupported_v4_first_wave_matrix("paper-local");
        let market = matrix
            .capabilities
            .iter_mut()
            .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
            .unwrap();
        market.source = CapabilitySupportSource::RuntimeSimulated;
        market.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
        let gtc = matrix
            .capabilities
            .iter_mut()
            .find(|entry| entry.capability == ExecutionCapabilityKind::Gtc)
            .unwrap();
        gtc.source = CapabilitySupportSource::RuntimeSimulated;
        gtc.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
        matrix
    }

    #[test]
    fn market_price_tick_helper_records_market_snapshot() {
        let mut runtime = V4PaperSimulatedRuntime::new_with_execution_capabilities(
            sample_compat_graph(),
            runtime_simulated_market_matrix(),
            vec![ExecutionCapabilityKind::Market],
        )
        .unwrap();

        let output = runtime
            .submit_market_price_tick("paper-local", "BTCUSDT", 70_000.0, 123, "price_tick")
            .unwrap();

        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == "price_tick"));
        assert_eq!(
            output.memory_snapshot.simulated_execution.portfolio_value,
            100_000.0
        );
    }

    fn provider_native_market_matrix_for_paper() -> VenueCapabilityMatrix {
        let mut matrix = unsupported_v4_first_wave_matrix("paper-local");
        let market = matrix
            .capabilities
            .iter_mut()
            .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
            .unwrap();
        market.source = CapabilitySupportSource::ProviderNative;
        market.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
        matrix
    }

    fn provider_native_market_matrix_for_live_actual() -> VenueCapabilityMatrix {
        let mut matrix = unsupported_v4_first_wave_matrix("paper-local");
        let market = matrix
            .capabilities
            .iter_mut()
            .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
            .unwrap();
        market.source = CapabilitySupportSource::ProviderNative;
        market.supported_modes = vec![RuntimeTradingMode::LiveActual];
        matrix
    }

    fn runtime_simulated_market_matrix_for_live_actual() -> VenueCapabilityMatrix {
        let mut matrix = unsupported_v4_first_wave_matrix("paper-local");
        let market = matrix
            .capabilities
            .iter_mut()
            .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
            .unwrap();
        market.source = CapabilitySupportSource::RuntimeSimulated;
        market.supported_modes = vec![RuntimeTradingMode::LiveActual];
        matrix
    }

    fn sample_runtime() -> V4PaperSimulatedRuntime {
        V4PaperSimulatedRuntime::new_with_execution_capabilities(
            sample_compat_graph(),
            runtime_simulated_market_matrix(),
            vec![ExecutionCapabilityKind::Market],
        )
        .unwrap()
    }

    fn nested_observation_graph(parent_matches: bool) -> V4MachineGraphContract {
        let mut graph = sample_compat_graph();
        let observation = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == V4_COMPAT_OBSERVATION_MACHINE_ID)
            .unwrap();
        observation.transitions[0].to_state = "idle".to_string();
        observation.transitions[0].event.event_type = if parent_matches {
            V4_COMPAT_CORE_IR_LOADED_EVENT.to_string()
        } else {
            V4_COMPAT_OBSERVATION_READY_EVENT.to_string()
        };
        observation.states[0].child_machine = Some(Box::new(V4MachineContract {
            schema_version: qrpc_core_ir::v4::V4_MACHINE_CONTRACT_VERSION.to_string(),
            machine_id: "data.market.child".to_string(),
            template: observation.template.clone(),
            states: vec![
                MachineState {
                    state_id: "idle".to_string(),
                    group_id: Some("child_flow".to_string()),
                    initial: true,
                    terminal: false,
                    child_machine: None,
                },
                MachineState {
                    state_id: "ready".to_string(),
                    group_id: Some("child_flow".to_string()),
                    initial: false,
                    terminal: false,
                    child_machine: None,
                },
            ],
            state_groups: vec![StateGroup {
                group_id: "child_flow".to_string(),
                state_ids: vec!["idle".to_string(), "ready".to_string()],
                conflict_policy: qrpc_core_ir::v4::TransitionConflictPolicy::FirstMatch,
                timeout_ms: None,
            }],
            transitions: vec![MachineTransition {
                transition_id: "data.market.child.t0".to_string(),
                from_state: "idle".to_string(),
                to_state: "ready".to_string(),
                event: MachineEventSelector {
                    event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                    source: None,
                    freshness: None,
                },
                guard: None,
                priority: 0,
                action: Some(MachineActionSpec {
                    emits: Vec::new(),
                    memory_writes: vec!["price".to_string()],
                    diagnostics: Vec::new(),
                }),
            }],
            memory: vec![MachineMemoryField {
                name: "price".to_string(),
                type_name: "price".to_string(),
                type_ref: None,
                default_value: Some(json!(0.0)),
                nullable: false,
            }],
            cache_policy: MachineCachePolicy::ReturnLastThenRecover,
            silence_policy: MachineSilencePolicy::SoftDormantAfter { ttl_ms: 60_000 },
            recovery_policy: MachineRecoveryPolicy::AsyncRecover,
            priority: 0,
            metadata: BTreeMap::new(),
        }));
        let catalog = graph.event_catalog.as_mut().unwrap();
        for event in &mut catalog.events {
            if event.event_type == V4_COMPAT_CORE_IR_LOADED_EVENT {
                event
                    .allowed_consumers
                    .push("data.market.child".to_string());
            }
            if event.event_type == V4_COMPAT_OBSERVATION_READY_EVENT {
                event
                    .allowed_consumers
                    .push(V4_COMPAT_OBSERVATION_MACHINE_ID.to_string());
            }
        }
        graph
    }

    #[test]
    fn v4_nested_machine_parent_transition_wins_over_child_transition() {
        let mut runtime = V4PaperSimulatedRuntime::new(nested_observation_graph(true)).unwrap();
        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({
                    "strategy_id": "runtime.compat.sample",
                    "price": 70_000.0
                }),
                ts_ms: 123,
            })
            .unwrap();

        let parent = output
            .memory_snapshot
            .machines
            .iter()
            .find(|machine| machine.machine_id == V4_COMPAT_OBSERVATION_MACHINE_ID)
            .unwrap();
        assert_eq!(parent.state_id, "idle");
        assert_eq!(parent.children[0].machine_id, "data.market.child");
        assert_eq!(parent.children[0].state_id, "idle");
    }

    #[test]
    fn v4_nested_machine_memory_is_isolated_and_visible_in_snapshot() {
        let mut runtime = V4PaperSimulatedRuntime::new(nested_observation_graph(false)).unwrap();
        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({
                    "strategy_id": "runtime.compat.sample",
                    "price": 70_000.0
                }),
                ts_ms: 123,
            })
            .unwrap();

        let parent = output
            .memory_snapshot
            .machines
            .iter()
            .find(|machine| machine.machine_id == V4_COMPAT_OBSERVATION_MACHINE_ID)
            .unwrap();
        let child = &parent.children[0];
        assert!(!parent.memory.contains_key("price"));
        assert_eq!(child.state_id, "ready", "{:?}", output.events);
        assert_eq!(child.memory.get("price"), Some(&json!(70_000.0)));
    }

    #[test]
    fn v4_backtest_bar_replay_is_deterministic() {
        let bars = vec![
            V4BacktestBarInput {
                venue_id: "paper-local".to_string(),
                symbol: "BTCUSDT".to_string(),
                close: 70_000.0,
                ts_ms: 1_700_000_000_000,
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            },
            V4BacktestBarInput {
                venue_id: "paper-local".to_string(),
                symbol: "BTCUSDT".to_string(),
                close: 70_250.0,
                ts_ms: 1_700_000_060_000,
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            },
        ];
        let run_once = || {
            let mut runtime = V4PaperSimulatedRuntime::new_for_backtest(
                sample_compat_graph(),
                runtime_simulated_market_matrix(),
                vec![ExecutionCapabilityKind::Market],
            )
            .unwrap();
            runtime.run_backtest_bars(&bars).unwrap()
        };

        let left = run_once();
        let right = run_once();

        assert_eq!(left.machine_trajectory, right.machine_trajectory);
        assert_eq!(left.final_snapshot, right.final_snapshot);
        assert_eq!(left.input_bar_count, 2);
        assert_eq!(left.symbols, vec!["BTCUSDT".to_string()]);
    }

    #[test]
    fn v4_multi_symbol_expansion_creates_independent_machine_instances() {
        let graph = sample_compat_graph();
        let expanded =
            expand_v4_graph_for_symbols(&graph, &["BTCUSDT".to_string(), "ETHUSDT".to_string()])
                .unwrap();

        assert_eq!(expanded.machines.len(), graph.machines.len() * 2);
        assert!(expanded
            .machines
            .iter()
            .any(|machine| machine.machine_id.starts_with("btcusdt::")));
        assert!(expanded
            .machines
            .iter()
            .any(|machine| machine.machine_id.starts_with("ethusdt::")));
        assert_eq!(
            expanded
                .risk_plane
                .as_ref()
                .map(|plane| plane.machine_ids.len()),
            graph
                .risk_plane
                .as_ref()
                .map(|plane| plane.machine_ids.len() * 2)
        );
        expanded.validate_static_contract().unwrap();
    }

    #[test]
    fn v4_runtime_rejects_event_payload_missing_required_catalog_field() {
        let mut runtime = sample_runtime();

        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({}),
                ts_ms: 1,
            })
            .unwrap();

        assert_eq!(
            runtime.machine_state_id(V4_COMPAT_OBSERVATION_MACHINE_ID),
            Some("idle")
        );
        let rejection = output
            .events
            .iter()
            .find(|event| event.event_type == V4_RUNTIME_EVENT_REJECTED_EVENT)
            .expect("missing v4 runtime rejection event");
        assert!(rejection
            .payload
            .get("reason")
            .and_then(Value::as_str)
            .unwrap()
            .contains("strategy_id"));
    }

    #[test]
    fn v4_runtime_rejects_event_payload_with_wrong_catalog_type() {
        let mut runtime = sample_runtime();

        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": 42 }),
                ts_ms: 1,
            })
            .unwrap();

        assert_eq!(
            runtime.machine_state_id(V4_COMPAT_OBSERVATION_MACHINE_ID),
            Some("idle")
        );
        let rejection = output
            .events
            .iter()
            .find(|event| event.event_type == V4_RUNTIME_EVENT_REJECTED_EVENT)
            .expect("missing v4 runtime rejection event");
        assert!(rejection
            .payload
            .get("reason")
            .and_then(Value::as_str)
            .unwrap()
            .contains("type mismatch"));
    }

    #[test]
    fn v4_paper_simulated_runtime_runs_compat_bridge_graph_until_execution_ready() {
        let mut runtime = sample_runtime();

        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        assert_eq!(output.runtime_mode, RuntimeTradingMode::PaperSimulated);
        assert!(!output.provider_order_submission_attached);
        assert_eq!(
            runtime.machine_state_id(V4_COMPAT_OBSERVATION_MACHINE_ID),
            Some("ready")
        );
        assert_eq!(
            runtime.machine_state_id(V4_COMPAT_DECISION_MACHINE_ID),
            Some("ready")
        );
        assert_eq!(
            runtime.machine_state_id(V4_COMPAT_EXECUTION_MACHINE_ID),
            Some("ready")
        );
        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == V4_COMPAT_RISK_APPROVED_EVENT));
        assert!(output.events.iter().any(|event| {
            event.event_type == EVENT_TRANSITION_APPLIED
                && event.source == V4_COMPAT_EXECUTION_MACHINE_ID
        }));
        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == EVENT_RISK_PLANE_APPROVED));
        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_CAPABILITY_ACCEPTED));
        assert_eq!(output.memory_snapshot.risk_plane.approved_event_count, 1);
        assert_eq!(output.memory_snapshot.risk_plane.rejected_event_count, 0);
        assert!(output.memory_snapshot.risk_plane.real_order_path_unlocked);
        assert_eq!(output.memory_snapshot.execution.accepted_count, 1);
        assert_eq!(output.memory_snapshot.execution.rejected_count, 0);
        let execution_decision = output
            .memory_snapshot
            .execution
            .last_decision
            .as_ref()
            .unwrap();
        assert!(execution_decision.accepted);
        assert_eq!(
            execution_decision.entries[0].source,
            CapabilitySupportSource::RuntimeSimulated
        );
        assert_eq!(
            execution_decision.entries[0].status,
            V4ExecutionCapabilityRuntimeStatus::Accepted
        );
        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_ORDER_ACKNOWLEDGED));
        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_ORDER_FILLED));
        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_FEE_CHARGED));
        assert_eq!(output.memory_snapshot.simulated_execution.order_count, 1);
        assert_eq!(output.memory_snapshot.simulated_execution.fill_count, 1);
        assert_eq!(
            output
                .memory_snapshot
                .simulated_execution
                .last_order
                .as_ref()
                .unwrap()
                .status,
            V4SimulatedOrderStatus::Filled
        );
        assert_eq!(
            output
                .memory_snapshot
                .simulated_execution
                .positions
                .first()
                .unwrap()
                .net_quantity,
            1.0
        );
        assert!(
            !output
                .memory_snapshot
                .venue_adapter_boundary
                .provider_order_submission_attached
        );
        assert!(
            output
                .memory_snapshot
                .venue_adapter_boundary
                .rejection_before_provider_submit
        );
    }

    #[test]
    fn v4_runtime_simulated_execution_config_controls_fill_fee_and_asset_curve() {
        let mut runtime = V4PaperSimulatedRuntime::new_with_execution_capabilities(
            sample_compat_graph(),
            runtime_simulated_market_matrix(),
            vec![ExecutionCapabilityKind::Market],
        )
        .unwrap()
        .with_simulated_execution_config(V4SimulatedExecutionConfig {
            starting_cash: 1_000.0,
            quote_asset: "USDT".to_string(),
            default_venue_id: "paper-local".to_string(),
            default_symbol: "ETHUSDT".to_string(),
            default_quantity: 2.0,
            default_price: 100.0,
            default_fee_bps: 10.0,
            default_slippage_bps: 50.0,
            allow_partial_fill: true,
            max_fill_quantity: Some(1.0),
        })
        .unwrap();

        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        let simulated = output.memory_snapshot.simulated_execution;
        assert_eq!(simulated.order_count, 1);
        assert_eq!(simulated.fill_count, 1);
        assert_eq!(
            simulated.last_order.as_ref().unwrap().status,
            V4SimulatedOrderStatus::PartiallyFilled
        );
        assert_eq!(simulated.last_fill.as_ref().unwrap().quantity, 1.0);
        assert!((simulated.last_fill.as_ref().unwrap().price - 100.5).abs() < 1e-9);
        assert!((simulated.last_fill.as_ref().unwrap().fee - 0.1005).abs() < 1e-9);
        assert!((simulated.cash_balance - 899.3995).abs() < 1e-9);
        assert!((simulated.position_market_value - 100.5).abs() < 1e-9);
        assert!((simulated.portfolio_value - 999.8995).abs() < 1e-9);
        assert_eq!(simulated.asset_curve.len(), 1);
    }

    #[test]
    fn v4_runtime_updates_simulated_portfolio_from_market_price() {
        let mut runtime = sample_runtime();
        runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        let events = runtime
            .update_simulated_market_price("paper-local", "BTCUSDT", 120.0, 2)
            .unwrap();
        let snapshot = runtime.simulated_execution_snapshot();

        assert!(events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_PORTFOLIO_CHANGED));
        assert_eq!(snapshot.positions[0].market_price, 120.0);
        assert_eq!(snapshot.position_market_value, 120.0);
        assert_eq!(snapshot.asset_curve.len(), 2);
    }

    #[test]
    fn v4_runtime_triggers_stop_market_order_on_market_price_update() {
        let mut runtime = sample_runtime();
        runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        let request = V4SimulatedOrderRequest {
            order_id: Some("close-long-stop".to_string()),
            client_order_id: None,
            venue_id: "paper-local".to_string(),
            symbol: "BTCUSDT".to_string(),
            action: V4SimulatedPositionAction::CloseLong,
            order_type: V4SimulatedOrderType::StopMarket,
            quantity: 1.0,
            reference_price: 100.0,
            limit_price: None,
            trigger_price: Some(95.0),
            take_profit_price: None,
            stop_loss_price: None,
            trailing_offset_bps: None,
            expire_at_ms: None,
            time_in_force: Some(V4SimulatedTimeInForce::Gtc),
            post_only: false,
            reduce_only: true,
            close_only: true,
            allow_partial_fill: true,
            fee_bps: 10.0,
            slippage_bps: 0.0,
            max_fill_quantity: None,
        };
        let acknowledgement = runtime.simulated_execution.submit_order(request, 1, 2);
        assert!(acknowledgement
            .events
            .iter()
            .any(|(event_type, _)| *event_type == EVENT_EXECUTION_ORDER_ACKNOWLEDGED));
        assert_eq!(runtime.simulated_execution.snapshot().open_order_count, 1);

        let events = runtime
            .update_simulated_market_price("paper-local", "BTCUSDT", 94.0, 3)
            .unwrap();
        let snapshot = runtime.simulated_execution_snapshot();

        assert!(events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_CONDITIONAL_ORDER_TRIGGERED));
        assert!(events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_ORDER_FILLED));
        assert_eq!(snapshot.open_order_count, 0);
        assert_eq!(snapshot.fill_count, 2);
        assert!(snapshot.positions[0].net_quantity.abs() < 1e-9);
    }

    #[test]
    fn v4_runtime_oco_bracket_fills_one_leg_and_cancels_the_other() {
        let mut runtime = sample_runtime();
        runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        let request = V4SimulatedOrderRequest {
            order_id: Some("bracket-1".to_string()),
            client_order_id: None,
            venue_id: "paper-local".to_string(),
            symbol: "BTCUSDT".to_string(),
            action: V4SimulatedPositionAction::CloseLong,
            order_type: V4SimulatedOrderType::OcoBracket,
            quantity: 1.0,
            reference_price: 100.0,
            limit_price: None,
            trigger_price: None,
            take_profit_price: Some(110.0),
            stop_loss_price: Some(95.0),
            trailing_offset_bps: None,
            expire_at_ms: None,
            time_in_force: Some(V4SimulatedTimeInForce::Gtc),
            post_only: false,
            reduce_only: true,
            close_only: true,
            allow_partial_fill: true,
            fee_bps: 10.0,
            slippage_bps: 0.0,
            max_fill_quantity: None,
        };
        runtime.simulated_execution.submit_order(request, 1, 2);

        let events = runtime
            .update_simulated_market_price("paper-local", "BTCUSDT", 111.0, 3)
            .unwrap();
        let snapshot = runtime.simulated_execution_snapshot();

        assert!(events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_ORDER_CANCELED));
        assert_eq!(snapshot.open_order_count, 0);
        assert_eq!(snapshot.fill_count, 2);
        assert!(snapshot.positions[0].net_quantity.abs() < 1e-9);
    }

    #[test]
    fn v4_runtime_trailing_stop_updates_trigger_then_fills() {
        let mut runtime = sample_runtime();
        runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        let request = V4SimulatedOrderRequest {
            order_id: Some("trail-1".to_string()),
            client_order_id: None,
            venue_id: "paper-local".to_string(),
            symbol: "BTCUSDT".to_string(),
            action: V4SimulatedPositionAction::CloseLong,
            order_type: V4SimulatedOrderType::TrailingStop,
            quantity: 1.0,
            reference_price: 100.0,
            limit_price: None,
            trigger_price: None,
            take_profit_price: None,
            stop_loss_price: None,
            trailing_offset_bps: Some(100.0),
            expire_at_ms: None,
            time_in_force: Some(V4SimulatedTimeInForce::Gtc),
            post_only: false,
            reduce_only: true,
            close_only: true,
            allow_partial_fill: true,
            fee_bps: 10.0,
            slippage_bps: 0.0,
            max_fill_quantity: None,
        };
        runtime.simulated_execution.submit_order(request, 1, 2);

        let first_update = runtime
            .update_simulated_market_price("paper-local", "BTCUSDT", 110.0, 3)
            .unwrap();
        assert!(first_update
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_ORDER_AMENDED));
        let last_order = runtime.simulated_execution_snapshot().last_order.unwrap();
        assert_eq!(last_order.trigger_price, Some(108.9));

        let fill_events = runtime
            .update_simulated_market_price("paper-local", "BTCUSDT", 108.0, 4)
            .unwrap();
        assert!(fill_events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_ORDER_FILLED));
        assert!(
            runtime.simulated_execution_snapshot().positions[0]
                .net_quantity
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn v4_runtime_gtd_order_expires_on_advance_time() {
        let mut runtime = sample_runtime();
        let request = V4SimulatedOrderRequest {
            order_id: Some("gtd-1".to_string()),
            client_order_id: None,
            venue_id: "paper-local".to_string(),
            symbol: "BTCUSDT".to_string(),
            action: V4SimulatedPositionAction::Buy,
            order_type: V4SimulatedOrderType::Limit,
            quantity: 1.0,
            reference_price: 100.0,
            limit_price: Some(90.0),
            trigger_price: None,
            take_profit_price: None,
            stop_loss_price: None,
            trailing_offset_bps: None,
            expire_at_ms: Some(10),
            time_in_force: Some(V4SimulatedTimeInForce::Gtd),
            post_only: false,
            reduce_only: false,
            close_only: false,
            allow_partial_fill: true,
            fee_bps: 10.0,
            slippage_bps: 0.0,
            max_fill_quantity: None,
        };
        runtime.simulated_execution.submit_order(request, 1, 2);

        let events = runtime.advance_time(10);

        assert!(events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_ORDER_EXPIRED));
        assert_eq!(runtime.simulated_execution_snapshot().open_order_count, 0);
    }

    #[test]
    fn v4_runtime_amends_open_order_without_replacing_order_id() {
        let mut runtime = sample_runtime();
        let request = V4SimulatedOrderRequest {
            order_id: Some("amendable-1".to_string()),
            client_order_id: None,
            venue_id: "paper-local".to_string(),
            symbol: "BTCUSDT".to_string(),
            action: V4SimulatedPositionAction::Buy,
            order_type: V4SimulatedOrderType::Limit,
            quantity: 1.0,
            reference_price: 100.0,
            limit_price: Some(90.0),
            trigger_price: None,
            take_profit_price: None,
            stop_loss_price: None,
            trailing_offset_bps: None,
            expire_at_ms: None,
            time_in_force: Some(V4SimulatedTimeInForce::Gtc),
            post_only: false,
            reduce_only: false,
            close_only: false,
            allow_partial_fill: true,
            fee_bps: 10.0,
            slippage_bps: 0.0,
            max_fill_quantity: None,
        };
        runtime.simulated_execution.submit_order(request, 1, 2);

        let outcome = runtime.simulated_execution.amend_order(
            "amendable-1",
            Some(100.0),
            Some(91.0),
            None,
            Some(0.5),
            3,
        );
        let order = runtime.simulated_execution_snapshot().last_order.unwrap();

        assert!(outcome
            .events
            .iter()
            .any(|(event_type, _)| *event_type == EVENT_EXECUTION_ORDER_AMENDED));
        assert_eq!(order.order_id, "amendable-1");
        assert_eq!(order.limit_price, Some(91.0));
        assert_eq!(order.requested_quantity, 0.5);
        assert_eq!(order.amend_revision, 1);
    }

    #[test]
    fn v4_backtest_tick_replay_is_deterministic_and_reports_micro_metrics() {
        let ticks = vec![
            V4BacktestTickInput {
                venue_id: "paper-local".to_string(),
                symbol: "BTCUSDT".to_string(),
                price: 70_250.0,
                size: 1.0,
                ts_ms: 1_700_000_060_000,
                sequence: 2,
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            },
            V4BacktestTickInput {
                venue_id: "paper-local".to_string(),
                symbol: "BTCUSDT".to_string(),
                price: 70_000.0,
                size: 1.0,
                ts_ms: 1_700_000_000_000,
                sequence: 1,
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
            },
        ];
        let run_once = || {
            let mut runtime = V4PaperSimulatedRuntime::new_for_backtest(
                sample_compat_graph(),
                runtime_simulated_market_matrix(),
                vec![ExecutionCapabilityKind::Market],
            )
            .unwrap();
            runtime.run_backtest_ticks(&ticks).unwrap()
        };

        let left = run_once();
        let right = run_once();

        assert_eq!(left.machine_trajectory, right.machine_trajectory);
        assert_eq!(left.replay_mode, "tick_replay");
        assert_eq!(left.input_tick_count, Some(2));
        assert_eq!(left.input_bar_count, 0);
        assert!(left.microstructure_metrics.is_some());
    }

    #[test]
    fn v4_runtime_rejects_order_when_request_uses_undeclared_capability() {
        let mut runtime = sample_runtime();
        let mut graph = sample_compat_graph();
        graph.metadata.insert(
            "default_symbol".to_string(),
            Value::String("BTCUSDT".to_string()),
        );
        runtime.graph = graph;

        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        assert_eq!(
            output
                .memory_snapshot
                .simulated_execution
                .rejected_order_count,
            0
        );

        let request = V4SimulatedOrderRequest {
            order_id: Some("bad-limit".to_string()),
            client_order_id: None,
            venue_id: "paper-local".to_string(),
            symbol: "BTCUSDT".to_string(),
            action: V4SimulatedPositionAction::Buy,
            order_type: V4SimulatedOrderType::Limit,
            quantity: 1.0,
            reference_price: 100.0,
            limit_price: Some(99.0),
            trigger_price: None,
            take_profit_price: None,
            stop_loss_price: None,
            trailing_offset_bps: None,
            expire_at_ms: None,
            time_in_force: None,
            post_only: false,
            reduce_only: false,
            close_only: false,
            allow_partial_fill: true,
            fee_bps: 10.0,
            slippage_bps: 0.0,
            max_fill_quantity: None,
        };
        let reason = runtime
            .validate_simulated_order_capabilities(&request)
            .unwrap_err();
        assert!(reason.contains("Limit"));
    }

    #[test]
    fn v4_runtime_returns_last_cache_and_recovers_soft_silent_machine() {
        let mut runtime = sample_runtime();
        runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        let silence_events = runtime.advance_time(60_001);
        assert!(silence_events.iter().any(|event| {
            event.event_type == EVENT_SILENCE_ENTERED
                && event.payload["machine_id"] == V4_COMPAT_OBSERVATION_MACHINE_ID
        }));
        assert_eq!(
            runtime.machine_status(V4_COMPAT_OBSERVATION_MACHINE_ID),
            Some(V4MachineRuntimeStatus::SoftSilent)
        );

        let pull_events = runtime
            .pull_machine(V4_COMPAT_OBSERVATION_MACHINE_ID, 60_010)
            .unwrap();
        assert!(pull_events
            .iter()
            .any(|event| event.event_type == EVENT_CACHE_RETURNED));
        assert!(pull_events
            .iter()
            .any(|event| event.event_type == EVENT_RECOVERY_STARTED));
        assert_eq!(
            runtime.machine_status(V4_COMPAT_OBSERVATION_MACHINE_ID),
            Some(V4MachineRuntimeStatus::Recovering)
        );

        let recovery_events = runtime
            .complete_recovery(V4_COMPAT_OBSERVATION_MACHINE_ID, 60_020)
            .unwrap();
        assert!(recovery_events
            .iter()
            .any(|event| event.event_type == EVENT_RECOVERY_COMPLETED));
        assert_eq!(
            runtime.machine_status(V4_COMPAT_OBSERVATION_MACHINE_ID),
            Some(V4MachineRuntimeStatus::Active)
        );
    }

    #[test]
    fn v4_runtime_memory_snapshot_records_machine_memory_and_cache() {
        let mut runtime = sample_runtime();
        runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        let snapshot = runtime.memory_snapshot(2);
        let observation = snapshot
            .machines
            .iter()
            .find(|machine| machine.machine_id == V4_COMPAT_OBSERVATION_MACHINE_ID)
            .unwrap();

        assert_eq!(snapshot.runtime_mode, RuntimeTradingMode::PaperSimulated);
        assert!(!snapshot.provider_order_submission_attached);
        assert_eq!(observation.memory["data_binding_count"], Value::from(1_u64));
        assert!(observation.cached_output.is_some());
        assert!(snapshot.event_sequence >= 6);
    }

    #[test]
    fn v4_runtime_rejects_live_simulated_mode_until_supported() {
        let bridge_report = bridge_core_ir_to_v4_machine_graph(&sample_core_ir_for_v4_runtime());
        let error = V4PaperSimulatedRuntime::new_for_mode(
            bridge_report.graph.unwrap(),
            RuntimeTradingMode::LiveSimulated,
        )
        .unwrap_err();

        assert!(error.to_string().contains("只允许 PaperSimulated 模式"));
    }

    #[test]
    fn v4_live_actual_boundary_allows_provider_actual_mode_but_keeps_submit_detached() {
        let runtime = V4Runtime::new_for_mode_with_execution_capabilities(
            sample_compat_graph(),
            RuntimeTradingMode::LiveActual,
            provider_native_market_matrix_for_live_actual(),
            vec![ExecutionCapabilityKind::Market],
        )
        .unwrap();
        let snapshot = runtime.memory_snapshot(1);

        assert_eq!(snapshot.runtime_mode, RuntimeTradingMode::LiveActual);
        assert!(
            snapshot
                .venue_adapter_boundary
                .provider_order_submission_allowed
        );
        assert!(
            !snapshot
                .venue_adapter_boundary
                .provider_order_submission_attached
        );
        assert_eq!(
            snapshot.venue_adapter_boundary.settlement_authority,
            RuntimeSettlementAuthority::ProviderActual
        );
    }

    #[test]
    fn v4_live_actual_rejects_runtime_simulated_capability_source() {
        let mut runtime = V4Runtime::new_for_mode_with_execution_capabilities(
            sample_compat_graph(),
            RuntimeTradingMode::LiveActual,
            runtime_simulated_market_matrix_for_live_actual(),
            vec![ExecutionCapabilityKind::Market],
        )
        .unwrap();

        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_CAPABILITY_REJECTED));
        assert_eq!(output.memory_snapshot.execution.accepted_count, 0);
        assert_eq!(output.memory_snapshot.execution.rejected_count, 1);
        assert!(output
            .memory_snapshot
            .execution
            .last_decision
            .as_ref()
            .unwrap()
            .reason
            .contains("provider_native"));
    }

    #[test]
    fn v4_live_actual_without_risk_plane_is_rejected() {
        let mut graph = sample_compat_graph();
        graph.risk_plane = None;
        let error = V4Runtime::new_for_mode(graph, RuntimeTradingMode::LiveActual).unwrap_err();

        assert!(error.to_string().contains("Risk Plane"));
    }

    #[test]
    fn v4_runtime_rejects_unsupported_execution_capability_before_execution() {
        let mut runtime = V4PaperSimulatedRuntime::new_with_execution_capabilities(
            sample_compat_graph(),
            unsupported_v4_first_wave_matrix("paper-local"),
            vec![ExecutionCapabilityKind::Market],
        )
        .unwrap();

        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        assert_eq!(
            runtime.machine_state_id(V4_COMPAT_EXECUTION_MACHINE_ID),
            Some("idle")
        );
        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_CAPABILITY_REJECTED));
        assert_eq!(output.memory_snapshot.execution.accepted_count, 0);
        assert_eq!(output.memory_snapshot.execution.rejected_count, 1);
        let decision = output
            .memory_snapshot
            .execution
            .last_decision
            .as_ref()
            .unwrap();
        assert!(!decision.accepted);
        assert_eq!(
            decision.entries[0].status,
            V4ExecutionCapabilityRuntimeStatus::Unsupported
        );
    }

    #[test]
    fn v4_runtime_rejects_provider_native_capability_in_paper_simulated() {
        let mut runtime = V4PaperSimulatedRuntime::new_with_execution_capabilities(
            sample_compat_graph(),
            provider_native_market_matrix_for_paper(),
            vec![ExecutionCapabilityKind::Market],
        )
        .unwrap();

        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        assert_eq!(
            runtime.machine_state_id(V4_COMPAT_EXECUTION_MACHINE_ID),
            Some("idle")
        );
        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_CAPABILITY_REJECTED));
        let decision = output
            .memory_snapshot
            .execution
            .last_decision
            .as_ref()
            .unwrap();
        assert!(!decision.accepted);
        assert_eq!(
            decision.entries[0].status,
            V4ExecutionCapabilityRuntimeStatus::ModeRejected
        );
        assert!(decision.reason.contains("requires runtime_simulated"));
    }

    #[test]
    fn v4_runtime_rejects_missing_execution_capability_policy() {
        let mut runtime = V4PaperSimulatedRuntime::new(sample_compat_graph()).unwrap();

        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_CORE_IR_LOADED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({ "strategy_id": "runtime.compat.sample" }),
                ts_ms: 1,
            })
            .unwrap();

        assert_eq!(
            runtime.machine_state_id(V4_COMPAT_EXECUTION_MACHINE_ID),
            Some("idle")
        );
        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_CAPABILITY_REJECTED));
        let decision = output
            .memory_snapshot
            .execution
            .last_decision
            .as_ref()
            .unwrap();
        assert!(!decision.accepted);
        assert_eq!(
            decision.entries[0].status,
            V4ExecutionCapabilityRuntimeStatus::PolicyMissing
        );
    }

    #[test]
    fn v4_runtime_rejects_forged_external_risk_plane_event_for_execution() {
        let mut graph = sample_compat_graph();
        let execution = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == V4_COMPAT_EXECUTION_MACHINE_ID)
            .unwrap();
        execution.transitions[0].event.source = None;

        let mut runtime = V4PaperSimulatedRuntime::new(graph).unwrap();
        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_RISK_APPROVED_EVENT.to_string(),
                source: V4_COMPAT_DECISION_MACHINE_ID.to_string(),
                payload: json!({
                    "execution_id": "exec_1",
                    "risk_plane_approved": true
                }),
                ts_ms: 1,
            })
            .unwrap();

        assert_eq!(
            runtime.machine_state_id(V4_COMPAT_EXECUTION_MACHINE_ID),
            Some("idle")
        );
        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == EVENT_RISK_PLANE_REJECTED));
        assert_eq!(output.memory_snapshot.risk_plane.approved_event_count, 0);
        assert_eq!(output.memory_snapshot.risk_plane.rejected_event_count, 1);
        assert!(output
            .memory_snapshot
            .risk_plane
            .last_decision
            .as_ref()
            .unwrap()
            .reason
            .contains("must be emitted"));
    }

    #[test]
    fn v4_runtime_rejects_execution_event_from_non_risk_plane_source() {
        let mut graph = sample_compat_graph();
        let execution = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == V4_COMPAT_EXECUTION_MACHINE_ID)
            .unwrap();
        execution.transitions[0].event.source = None;

        let mut runtime = V4PaperSimulatedRuntime::new(graph).unwrap();
        let output = runtime
            .submit_event(V4RuntimeInputEvent {
                event_type: V4_COMPAT_RISK_APPROVED_EVENT.to_string(),
                source: "runtime".to_string(),
                payload: json!({
                    "execution_id": "exec_1",
                    "risk_plane_approved": true
                }),
                ts_ms: 1,
            })
            .unwrap();

        assert_eq!(
            runtime.machine_state_id(V4_COMPAT_EXECUTION_MACHINE_ID),
            Some("idle")
        );
        assert!(output
            .events
            .iter()
            .any(|event| event.event_type == EVENT_RISK_PLANE_REJECTED));
        assert!(output
            .memory_snapshot
            .risk_plane
            .last_decision
            .as_ref()
            .unwrap()
            .reason
            .contains("not a runtime Risk Plane machine"));
    }
}
