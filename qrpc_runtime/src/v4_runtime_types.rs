use crate::backtest_metrics::{
    compute_microstructure_metrics, MicrostructureFillSample, MicrostructureOrderSample,
};
use anyhow::{anyhow, Result};
use qrpc_core_ir::v4::{
    default_v4_runtime_mode_contract, CapabilitySupportSource, ComplexityMetrics,
    EventFreshnessRequirement, ExecutionCapabilityKind, MachineCachePolicy,
    MachineEventPayloadField, MachineEventSourceKind, MachineRecoveryPolicy, MachineSilencePolicy,
    MachineTemplateKind, MachineTransition, RuntimeSettlementAuthority, RuntimeTradingMode,
    V4BacktestArtifact, V4BacktestExecutionCapabilitySourceRecord,
    V4BacktestMachineTrajectoryPoint, V4BacktestRiskPlaneDecisionRecord, V4MachineContract,
    V4MachineGraphContract, VenueCapabilityMatrix, V4_BACKTEST_ARTIFACT_VERSION,
    V4_RUNTIME_EVENT_REJECTED_EVENT,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const V4_RUNTIME_MAX_EVENT_STEPS: usize = 1_024;
const V4_RUNTIME_MAX_EVENT_LOG_ENTRIES: usize = 100_000;
const V4_SIMULATED_MAX_ORDER_HISTORY: usize = 10_000;
const V4_SIMULATED_MAX_FILL_HISTORY: usize = 10_000;
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
