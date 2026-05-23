use anyhow::{anyhow, Result};
use qrpc_core_ir::v4::{
    EventFreshnessRequirement, MachineCachePolicy, MachineRecoveryPolicy, MachineSilencePolicy,
    MachineTemplateKind, RuntimeTradingMode, V4MachineGraphContract,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, VecDeque};

const V4_RUNTIME_MAX_EVENT_STEPS: usize = 1_024;
const EVENT_DOWNSTREAM_PULL: &str = "downstream_pull";
const EVENT_SILENCE_ENTERED: &str = "silence_entered";
const EVENT_SILENCE_EXITED: &str = "silence_exited";
const EVENT_CACHE_RETURNED: &str = "cache_returned";
const EVENT_RECOVERY_STARTED: &str = "recovery_started";
const EVENT_RECOVERY_COMPLETED: &str = "recovery_completed";
const EVENT_TRANSITION_APPLIED: &str = "machine_transition_applied";

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
    pub ts_ms: u64,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub replayable: bool,
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
pub struct V4PaperSimulatedRuntime {
    graph: V4MachineGraphContract,
    runtime_mode: RuntimeTradingMode,
    machines: BTreeMap<String, MachineRuntimeState>,
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

        Ok(Self {
            graph,
            runtime_mode,
            machines,
            event_queue: VecDeque::new(),
            event_log: Vec::new(),
            sequence: 0,
            provider_order_submission_attached: false,
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
                );
            }
        }

        Ok(())
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
        }

        Value::Object(payload)
    }

    fn enqueue_graph_event(
        &mut self,
        event_type: impl Into<String>,
        source: impl Into<String>,
        payload: Value,
        ts_ms: u64,
        replayable: bool,
    ) {
        self.sequence += 1;
        let event = V4RuntimeEventEnvelope {
            sequence: self.sequence,
            event_type: event_type.into(),
            source: source.into(),
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
        bridge_core_ir_to_v4_machine_graph, V4_COMPAT_CORE_IR_LOADED_EVENT,
        V4_COMPAT_DECISION_MACHINE_ID, V4_COMPAT_EXECUTION_MACHINE_ID,
        V4_COMPAT_OBSERVATION_MACHINE_ID, V4_COMPAT_RISK_APPROVED_EVENT,
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

    fn sample_runtime() -> V4PaperSimulatedRuntime {
        let bridge_report = bridge_core_ir_to_v4_machine_graph(&sample_core_ir_for_v4_runtime());
        V4PaperSimulatedRuntime::new(bridge_report.graph.unwrap()).unwrap()
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
}
