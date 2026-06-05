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
    #[serde(default)]
    pub priority: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<MachineActionSpec>,
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
