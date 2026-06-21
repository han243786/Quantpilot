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
    #[serde(default)]
    pub conditions: Vec<MachineGuardConditionSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<MachineGuardPolicySpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineGuardConditionSpec {
    pub condition_id: String,
    pub left_read: MachineGuardReadRef,
    pub comparator: MachineGuardConditionComparator,
    pub right_parameter_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineGuardConditionProjection {
    pub condition_id: String,
    pub left_read: MachineGuardReadRef,
    pub comparator: MachineGuardConditionComparator,
    pub right_parameter_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_parameter_path_kind: Option<MachineGuardParameterPathKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineGuardConditionComparator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
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

impl MachineGuardReadSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            MachineGuardReadSource::EventPayload => "event_payload",
            MachineGuardReadSource::MachineMemory => "machine_memory",
            MachineGuardReadSource::ReadonlyRuntimeFact => "readonly_runtime_fact",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineGuardParameterPathKind {
    Guard,
    Timeout,
    Cooldown,
    Threshold,
    RiskLimit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineGuardExecutionReadinessState {
    DisabledFailClosed,
}

pub const MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE: &str =
    "guard_execution_disabled_fail_closed";
pub const MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON: &str =
    "guard execution is not enabled and v4 runtime fails closed";

pub const MACHINE_GUARD_READONLY_RUNTIME_FACTS: &[&str] =
    &["clock.tick_ms", "runtime.mode", "capability.snapshot_id"];

impl MachineGuardExecutionReadinessState {
    pub fn blocker_code(self) -> &'static str {
        match self {
            MachineGuardExecutionReadinessState::DisabledFailClosed => {
                MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_CODE
            }
        }
    }

    pub fn blocker_reason(self) -> &'static str {
        match self {
            MachineGuardExecutionReadinessState::DisabledFailClosed => {
                MACHINE_GUARD_EXECUTION_DISABLED_FAIL_CLOSED_REASON
            }
        }
    }
}

pub fn machine_guard_readonly_runtime_fact_allowed(path: &str) -> bool {
    MACHINE_GUARD_READONLY_RUNTIME_FACTS.contains(&path)
}

pub fn machine_guard_parameter_path_kind(path: &str) -> Option<MachineGuardParameterPathKind> {
    let path = path.trim().to_ascii_lowercase();
    if path.is_empty() {
        return None;
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
        return None;
    }

    if path.contains("timeout") {
        return Some(MachineGuardParameterPathKind::Timeout);
    }
    if path.contains("cooldown") {
        return Some(MachineGuardParameterPathKind::Cooldown);
    }
    if path.contains("threshold") {
        return Some(MachineGuardParameterPathKind::Threshold);
    }
    let risk_limit = ["max_notional", "max_position", "max_drawdown", "drawdown"];
    if risk_limit.iter().any(|needle| path.contains(needle)) {
        return Some(MachineGuardParameterPathKind::RiskLimit);
    }
    if path.contains("guard") {
        return Some(MachineGuardParameterPathKind::Guard);
    }
    None
}

pub fn machine_guard_parameter_path_allowed(path: &str) -> bool {
    machine_guard_parameter_path_kind(path).is_some()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineGuardDescriptorReadiness {
    pub guard_id: String,
    pub read_count: usize,
    pub event_payload_read_count: usize,
    pub machine_memory_read_count: usize,
    pub readonly_runtime_fact_read_count: usize,
    pub parameter_path_count: usize,
    pub guard_parameter_path_count: usize,
    pub timeout_parameter_path_count: usize,
    pub cooldown_parameter_path_count: usize,
    pub threshold_parameter_path_count: usize,
    pub risk_limit_parameter_path_count: usize,
    pub condition_count: usize,
    pub equal_condition_count: usize,
    pub not_equal_condition_count: usize,
    pub greater_than_condition_count: usize,
    pub greater_than_or_equal_condition_count: usize,
    pub less_than_condition_count: usize,
    pub less_than_or_equal_condition_count: usize,
    pub condition_event_payload_read_count: usize,
    pub condition_machine_memory_read_count: usize,
    pub condition_readonly_runtime_fact_read_count: usize,
    pub condition_guard_parameter_path_count: usize,
    pub condition_timeout_parameter_path_count: usize,
    pub condition_cooldown_parameter_path_count: usize,
    pub condition_threshold_parameter_path_count: usize,
    pub condition_risk_limit_parameter_path_count: usize,
    pub policy_declared: bool,
    pub timeout_declared: bool,
    pub cooldown_declared: bool,
    pub fallback_declared: bool,
    pub execution_enabled: bool,
    pub execution_state: MachineGuardExecutionReadinessState,
    pub execution_blocker_code: String,
    pub execution_blocker_reason: String,
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
    pub parameter_path_kinds: Vec<MachineGuardParameterPathKind>,
    pub conditions: Vec<MachineGuardConditionSpec>,
    pub condition_projections: Vec<MachineGuardConditionProjection>,
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
    pub fn condition_projections(&self) -> Vec<MachineGuardConditionProjection> {
        self.conditions
            .iter()
            .map(|condition| MachineGuardConditionProjection {
                condition_id: condition.condition_id.clone(),
                left_read: condition.left_read.clone(),
                comparator: condition.comparator.clone(),
                right_parameter_path: condition.right_parameter_path.clone(),
                right_parameter_path_kind: machine_guard_parameter_path_kind(
                    condition.right_parameter_path.as_str(),
                ),
            })
            .collect()
    }

    pub fn parameter_path_kinds(&self) -> Vec<MachineGuardParameterPathKind> {
        self.parameter_paths
            .iter()
            .filter_map(|path| machine_guard_parameter_path_kind(path))
            .collect()
    }

    pub fn readiness(&self) -> MachineGuardDescriptorReadiness {
        let parameter_path_kinds = self.parameter_path_kinds();
        let condition_parameter_path_kinds = self
            .conditions
            .iter()
            .filter_map(|condition| {
                machine_guard_parameter_path_kind(condition.right_parameter_path.as_str())
            })
            .collect::<Vec<_>>();
        let execution_state = MachineGuardExecutionReadinessState::DisabledFailClosed;
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
            guard_parameter_path_count: parameter_path_kinds
                .iter()
                .filter(|kind| **kind == MachineGuardParameterPathKind::Guard)
                .count(),
            timeout_parameter_path_count: parameter_path_kinds
                .iter()
                .filter(|kind| **kind == MachineGuardParameterPathKind::Timeout)
                .count(),
            cooldown_parameter_path_count: parameter_path_kinds
                .iter()
                .filter(|kind| **kind == MachineGuardParameterPathKind::Cooldown)
                .count(),
            threshold_parameter_path_count: parameter_path_kinds
                .iter()
                .filter(|kind| **kind == MachineGuardParameterPathKind::Threshold)
                .count(),
            risk_limit_parameter_path_count: parameter_path_kinds
                .iter()
                .filter(|kind| **kind == MachineGuardParameterPathKind::RiskLimit)
                .count(),
            condition_count: self.conditions.len(),
            equal_condition_count: self
                .conditions
                .iter()
                .filter(|condition| condition.comparator == MachineGuardConditionComparator::Equal)
                .count(),
            not_equal_condition_count: self
                .conditions
                .iter()
                .filter(|condition| {
                    condition.comparator == MachineGuardConditionComparator::NotEqual
                })
                .count(),
            greater_than_condition_count: self
                .conditions
                .iter()
                .filter(|condition| {
                    condition.comparator == MachineGuardConditionComparator::GreaterThan
                })
                .count(),
            greater_than_or_equal_condition_count: self
                .conditions
                .iter()
                .filter(|condition| {
                    condition.comparator == MachineGuardConditionComparator::GreaterThanOrEqual
                })
                .count(),
            less_than_condition_count: self
                .conditions
                .iter()
                .filter(|condition| {
                    condition.comparator == MachineGuardConditionComparator::LessThan
                })
                .count(),
            less_than_or_equal_condition_count: self
                .conditions
                .iter()
                .filter(|condition| {
                    condition.comparator == MachineGuardConditionComparator::LessThanOrEqual
                })
                .count(),
            condition_event_payload_read_count: self
                .conditions
                .iter()
                .filter(|condition| {
                    condition.left_read.source == MachineGuardReadSource::EventPayload
                })
                .count(),
            condition_machine_memory_read_count: self
                .conditions
                .iter()
                .filter(|condition| {
                    condition.left_read.source == MachineGuardReadSource::MachineMemory
                })
                .count(),
            condition_readonly_runtime_fact_read_count: self
                .conditions
                .iter()
                .filter(|condition| {
                    condition.left_read.source == MachineGuardReadSource::ReadonlyRuntimeFact
                })
                .count(),
            condition_guard_parameter_path_count: condition_parameter_path_kinds
                .iter()
                .filter(|kind| **kind == MachineGuardParameterPathKind::Guard)
                .count(),
            condition_timeout_parameter_path_count: condition_parameter_path_kinds
                .iter()
                .filter(|kind| **kind == MachineGuardParameterPathKind::Timeout)
                .count(),
            condition_cooldown_parameter_path_count: condition_parameter_path_kinds
                .iter()
                .filter(|kind| **kind == MachineGuardParameterPathKind::Cooldown)
                .count(),
            condition_threshold_parameter_path_count: condition_parameter_path_kinds
                .iter()
                .filter(|kind| **kind == MachineGuardParameterPathKind::Threshold)
                .count(),
            condition_risk_limit_parameter_path_count: condition_parameter_path_kinds
                .iter()
                .filter(|kind| **kind == MachineGuardParameterPathKind::RiskLimit)
                .count(),
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
            execution_state,
            execution_blocker_code: execution_state.blocker_code().to_string(),
            execution_blocker_reason: execution_state.blocker_reason().to_string(),
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
                    parameter_path_kinds: guard_descriptor.parameter_path_kinds(),
                    conditions: guard_descriptor.conditions.clone(),
                    condition_projections: guard_descriptor.condition_projections(),
                    policy: guard_descriptor.policy.clone(),
                })
            })
            .collect()
    }
}
