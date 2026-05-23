use anyhow::{anyhow, Result};
use qrpc_core_ir::v4::{
    default_v4_runtime_mode_contract, CapabilitySupportSource, EventFreshnessRequirement,
    ExecutionCapabilityKind, MachineCachePolicy, MachineEventSourceKind, MachineRecoveryPolicy,
    MachineSilencePolicy, MachineTemplateKind, RuntimeTradingMode, V4MachineGraphContract,
    VenueCapabilityMatrix,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4RuntimeInputEvent {
    pub event_type: String,
    pub source: String,
    #[serde(default)]
    pub payload: Value,
    pub ts_ms: u64,
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
pub struct V4PaperSimulatedRuntime {
    graph: V4MachineGraphContract,
    runtime_mode: RuntimeTradingMode,
    machines: BTreeMap<String, MachineRuntimeState>,
    risk_plane: V4RiskPlaneRuntimeState,
    execution: V4ExecutionRuntimeState,
    event_queue: VecDeque<V4RuntimeEventEnvelope>,
    event_log: Vec<V4RuntimeEventEnvelope>,
    sequence: u64,
    provider_order_submission_attached: bool,
}

impl V4PaperSimulatedRuntime {
    pub fn new(graph: V4MachineGraphContract) -> Result<Self> {
        Self::new_for_mode(graph, RuntimeTradingMode::PaperSimulated)
    }

    pub fn new_for_mode(
        graph: V4MachineGraphContract,
        runtime_mode: RuntimeTradingMode,
    ) -> Result<Self> {
        if runtime_mode != RuntimeTradingMode::PaperSimulated {
            return Err(anyhow!(
                "v4 Phase 5 runtime only accepts PaperSimulated mode, got {:?}",
                runtime_mode
            ));
        }
        graph.validate_static_contract().map_err(|errors| {
            anyhow!(
                "v4 machine graph failed static contract before PaperSimulated runtime: {:?}",
                errors
            )
        })?;

        let mut machines = BTreeMap::new();
        for machine in &graph.machines {
            let initial_state = machine
                .states
                .iter()
                .find(|state| state.initial)
                .ok_or_else(|| anyhow!("machine `{}` has no initial state", machine.machine_id))?;
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

    pub fn with_execution_capabilities(
        mut self,
        venue_matrix: VenueCapabilityMatrix,
        required_capabilities: Vec<ExecutionCapabilityKind>,
    ) -> Result<Self> {
        venue_matrix
            .validate_required_capability_sources(&required_capabilities)
            .map_err(|errors| {
                anyhow!(
                    "v4 execution capability policy failed static contract: {:?}",
                    errors
                )
            })?;
        self.execution.capability_policy = Some(V4ExecutionCapabilityRuntimePolicy {
            venue_matrix,
            required_capabilities,
        });
        Ok(self)
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
            .ok_or_else(|| anyhow!("unknown machine `{machine_id}`"))?
            .cache_policy
            .clone();
        let mut cached_to_return = None;
        let mut recovery_started = false;
        {
            let state = self
                .machines
                .get_mut(machine_id)
                .ok_or_else(|| anyhow!("unknown machine `{machine_id}`"))?;
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
                .ok_or_else(|| anyhow!("unknown machine `{machine_id}`"))?;
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

    pub fn memory_snapshot(&self, now_ms: u64) -> V4RuntimeMemorySnapshot {
        V4RuntimeMemorySnapshot {
            graph_id: self.graph.graph_id.clone(),
            runtime_mode: self.runtime_mode,
            ts_ms: now_ms,
            machines: self
                .graph
                .machines
                .iter()
                .filter_map(|machine| {
                    let state = self.machines.get(&machine.machine_id)?;
                    Some(V4MachineRuntimeSnapshot {
                        machine_id: machine.machine_id.clone(),
                        template: machine.template.clone(),
                        state_id: state.state_id.clone(),
                        status: state.status,
                        memory: state.memory.clone(),
                        cached_output: state.cached_output.clone(),
                        last_pulled_at_ms: state.last_pulled_at_ms,
                        last_event_at_ms: state.last_event_at_ms,
                    })
                })
                .collect(),
            risk_plane: self.risk_plane_snapshot(),
            execution: self.execution_snapshot(),
            event_sequence: self.sequence,
            provider_order_submission_attached: self.provider_order_submission_attached,
        }
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
                    "v4 runtime exceeded max event steps {}",
                    V4_RUNTIME_MAX_EVENT_STEPS
                ));
            }
            self.process_event(event)?;
        }
        Ok(())
    }

    fn process_event(&mut self, event: V4RuntimeEventEnvelope) -> Result<()> {
        if self.machines.contains_key(event.source.as_str()) {
            if let Some(source_state) = self.machines.get_mut(event.source.as_str()) {
                source_state.last_pulled_at_ms = Some(event.ts_ms);
            }
        }

        let mut candidates = self
            .graph
            .machines
            .iter()
            .filter_map(|machine| {
                let runtime_state = self.machines.get(machine.machine_id.as_str())?;
                let transition = machine.transitions.iter().find(|transition| {
                    transition.from_state == runtime_state.state_id
                        && transition.event.event_type == event.event_type
                        && transition_source_matches(transition.event.source.as_deref(), &event)
                        && transition_freshness_matches(transition.event.freshness.clone(), &event)
                })?;
                Some((
                    machine.priority,
                    machine.machine_id.clone(),
                    transition.clone(),
                ))
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));

        for (_, machine_id, transition) in candidates {
            let Some(machine) = self.machine_spec(&machine_id).cloned() else {
                continue;
            };
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
                reason: "Execution capability policy is missing".to_string(),
                entries: vec![V4ExecutionCapabilityRuntimeEntry {
                    capability: ExecutionCapabilityKind::Market,
                    source: CapabilitySupportSource::Unsupported,
                    status: V4ExecutionCapabilityRuntimeStatus::PolicyMissing,
                    reason: "Execution capability policy is missing".to_string(),
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
                reason: "ExecutionMachine requires at least one declared execution capability"
                    .to_string(),
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
                        "execution capability `{:?}` is not declared for venue `{}`",
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
                    "execution capability `{:?}` is unsupported for venue `{}`",
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
                        "execution capability `{:?}` is accepted as `{:?}` in `{:?}`",
                        capability, source, self.runtime_mode
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
                "Execution capabilities accepted for runtime mode".to_string()
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

    fn machine_spec(&self, machine_id: &str) -> Option<&qrpc_core_ir::v4::V4MachineContract> {
        self.graph
            .machines
            .iter()
            .find(|machine| machine.machine_id == machine_id)
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

#[allow(dead_code)]
fn recovery_policy_allows_async(policy: &MachineRecoveryPolicy) -> bool {
    matches!(policy, MachineRecoveryPolicy::AsyncRecover)
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core_ir::v4::{
        bridge_core_ir_to_v4_machine_graph, unsupported_v4_first_wave_matrix,
        CapabilitySupportSource, ExecutionCapabilityKind, V4MachineGraphContract,
        VenueCapabilityMatrix, V4_COMPAT_CORE_IR_LOADED_EVENT, V4_COMPAT_DECISION_MACHINE_ID,
        V4_COMPAT_EXECUTION_MACHINE_ID, V4_COMPAT_OBSERVATION_MACHINE_ID,
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
        matrix
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

    fn sample_runtime() -> V4PaperSimulatedRuntime {
        V4PaperSimulatedRuntime::new_with_execution_capabilities(
            sample_compat_graph(),
            runtime_simulated_market_matrix(),
            vec![ExecutionCapabilityKind::Market],
        )
        .unwrap()
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
    fn v4_runtime_rejects_non_paper_simulated_mode_in_phase_five() {
        let bridge_report = bridge_core_ir_to_v4_machine_graph(&sample_core_ir_for_v4_runtime());
        let error = V4PaperSimulatedRuntime::new_for_mode(
            bridge_report.graph.unwrap(),
            RuntimeTradingMode::LiveActual,
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("only accepts PaperSimulated mode"));
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
                payload: json!({ "risk_plane_approved": true }),
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
                payload: json!({ "risk_plane_approved": true }),
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
