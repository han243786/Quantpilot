mod static_validation;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{default_machine_contract_version, default_transition_conflict_policy, QsTypeRef};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MachineTemplateKind {
    Observation,
    Decision,
    Execution,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineCachePolicy {
    NoCache,
    ReturnLastThenRecover,
    InvalidateOnSilence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MachineSilencePolicy {
    Pinned,
    ManualOnly,
    SoftDormantAfter { ttl_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineRecoveryPolicy {
    AsyncRecover,
    SyncRecover,
    ManualRecover,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransitionConflictPolicy {
    Error,
    FirstMatch,
    MaxConfidence,
    RiskFirst,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventFreshnessRequirement {
    FreshOnly,
    FreshOrStale,
    RecoveringAllowed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4MachineContract {
    #[serde(default = "default_machine_contract_version")]
    pub schema_version: String,
    pub machine_id: String,
    pub template: MachineTemplateKind,
    #[serde(default)]
    pub states: Vec<MachineState>,
    #[serde(default)]
    pub state_groups: Vec<StateGroup>,
    #[serde(default)]
    pub transitions: Vec<MachineTransition>,
    #[serde(default)]
    pub memory: Vec<MachineMemoryField>,
    pub cache_policy: MachineCachePolicy,
    pub silence_policy: MachineSilencePolicy,
    pub recovery_policy: MachineRecoveryPolicy,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MachineState {
    pub state_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(default)]
    pub initial: bool,
    #[serde(default)]
    pub terminal: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_machine: Option<Box<V4MachineContract>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateGroup {
    pub group_id: String,
    #[serde(default)]
    pub state_ids: Vec<String>,
    #[serde(default = "default_transition_conflict_policy")]
    pub conflict_policy: TransitionConflictPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MachineTransition {
    pub transition_id: String,
    pub from_state: String,
    pub to_state: String,
    pub event: MachineEventSelector,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard_descriptor: Option<MachineGuardDescriptor>,
    #[serde(default)]
    pub priority: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<MachineActionSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineGuardDescriptor {
    pub guard_id: String,
    #[serde(default)]
    pub reads: Vec<MachineGuardReadRef>,
    #[serde(default)]
    pub parameter_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<MachineGuardPolicySpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineGuardPolicySpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooldown_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<MachineGuardFallbackPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineGuardFallbackPolicy {
    FailClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineGuardReadRef {
    pub source: MachineGuardReadSource,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineGuardReadSource {
    EventPayload,
    MachineMemory,
    ReadonlyRuntimeFact,
}

pub const MACHINE_GUARD_READONLY_RUNTIME_FACTS: &[&str] =
    &["clock.tick_ms", "runtime.mode", "capability.snapshot_id"];

pub fn machine_guard_readonly_runtime_fact_allowed(path: &str) -> bool {
    MACHINE_GUARD_READONLY_RUNTIME_FACTS.contains(&path)
}

pub fn machine_guard_parameter_path_allowed(path: &str) -> bool {
    let path = path.trim().to_ascii_lowercase();
    if path.is_empty() {
        return false;
    }

    let forbidden = [
        "topology",
        "graph.",
        "graph.edges",
        "event_catalog",
        "event_schema",
        "event.",
        "capability_source",
        "capability.source",
        "active_strategy",
    ];
    if forbidden.iter().any(|needle| path.contains(needle)) {
        return false;
    }

    let allowed = [
        "guard",
        "cooldown",
        "threshold",
        "max_notional",
        "max_position",
        "max_drawdown",
        "drawdown",
    ];
    allowed.iter().any(|needle| path.contains(needle))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineGuardDescriptorReadiness {
    pub guard_id: String,
    pub read_count: usize,
    pub event_payload_read_count: usize,
    pub machine_memory_read_count: usize,
    pub readonly_runtime_fact_read_count: usize,
    pub parameter_path_count: usize,
    pub policy_declared: bool,
    pub timeout_declared: bool,
    pub cooldown_declared: bool,
    pub fallback_declared: bool,
    pub execution_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineGuardDescriptorProjection {
    pub transition_id: String,
    pub from_state: String,
    pub to_state: String,
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_source: Option<String>,
    pub readiness: MachineGuardDescriptorReadiness,
    pub reads: Vec<MachineGuardReadRef>,
    pub parameter_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<MachineGuardPolicySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineEventSelector {
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freshness: Option<EventFreshnessRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MachineActionSpec {
    #[serde(default)]
    pub emits: Vec<String>,
    #[serde(default)]
    pub memory_writes: Vec<String>,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MachineMemoryField {
    pub name: String,
    pub type_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_ref: Option<QsTypeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<Value>,
    #[serde(default)]
    pub nullable: bool,
}

impl MachineGuardDescriptor {
    pub fn readiness(&self) -> MachineGuardDescriptorReadiness {
        MachineGuardDescriptorReadiness {
            guard_id: self.guard_id.clone(),
            read_count: self.reads.len(),
            event_payload_read_count: self
                .reads
                .iter()
                .filter(|read| read.source == MachineGuardReadSource::EventPayload)
                .count(),
            machine_memory_read_count: self
                .reads
                .iter()
                .filter(|read| read.source == MachineGuardReadSource::MachineMemory)
                .count(),
            readonly_runtime_fact_read_count: self
                .reads
                .iter()
                .filter(|read| read.source == MachineGuardReadSource::ReadonlyRuntimeFact)
                .count(),
            parameter_path_count: self.parameter_paths.len(),
            policy_declared: self.policy.is_some(),
            timeout_declared: self
                .policy
                .as_ref()
                .and_then(|policy| policy.timeout_ms)
                .is_some(),
            cooldown_declared: self
                .policy
                .as_ref()
                .and_then(|policy| policy.cooldown_ms)
                .is_some(),
            fallback_declared: self
                .policy
                .as_ref()
                .and_then(|policy| policy.fallback.as_ref())
                .is_some(),
            execution_enabled: false,
        }
    }
}

impl V4MachineContract {
    pub fn guard_descriptor_projections(&self) -> Vec<MachineGuardDescriptorProjection> {
        self.transitions
            .iter()
            .filter_map(|transition| {
                let guard_descriptor = transition.guard_descriptor.as_ref()?;
                Some(MachineGuardDescriptorProjection {
                    transition_id: transition.transition_id.clone(),
                    from_state: transition.from_state.clone(),
                    to_state: transition.to_state.clone(),
                    event_type: transition.event.event_type.clone(),
                    event_source: transition.event.source.clone(),
                    readiness: guard_descriptor.readiness(),
                    reads: guard_descriptor.reads.clone(),
                    parameter_paths: guard_descriptor.parameter_paths.clone(),
                    policy: guard_descriptor.policy.clone(),
                })
            })
            .collect()
    }
}
