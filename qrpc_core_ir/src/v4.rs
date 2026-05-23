use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const V4_MACHINE_CONTRACT_VERSION: &str = "quantpilot/machine-contract/v1";
pub const V4_VENUE_CAPABILITY_MATRIX_VERSION: &str = "quantpilot/venue-capability-matrix/v1";

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeTradingMode {
    PaperActual,
    PaperSimulated,
    LiveActual,
    LiveSimulated,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineState {
    pub state_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(default)]
    pub initial: bool,
    #[serde(default)]
    pub terminal: bool,
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
    pub default_value: Option<Value>,
    #[serde(default)]
    pub nullable: bool,
}

impl V4MachineContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_MACHINE_CONTRACT_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_MACHINE_CONTRACT_VERSION
            ));
        }
        if self.machine_id.trim().is_empty() {
            errors.push("machine_id is required".to_string());
        }
        if self.states.is_empty() {
            errors.push("at least one state is required".to_string());
        }

        let mut state_ids = BTreeSet::new();
        let mut initial_count = 0;
        for state in &self.states {
            if state.state_id.trim().is_empty() {
                errors.push("state_id is required".to_string());
                continue;
            }
            if !state_ids.insert(state.state_id.as_str()) {
                errors.push(format!("duplicate state `{}`", state.state_id));
            }
            if state.initial {
                initial_count += 1;
            }
        }
        if initial_count != 1 {
            errors.push(format!(
                "exactly one initial state is required, found {}",
                initial_count
            ));
        }

        let mut group_ids = BTreeSet::new();
        for group in &self.state_groups {
            if group.group_id.trim().is_empty() {
                errors.push("state_group group_id is required".to_string());
                continue;
            }
            if !group_ids.insert(group.group_id.as_str()) {
                errors.push(format!("duplicate state_group `{}`", group.group_id));
            }
            for state_id in &group.state_ids {
                if !state_ids.contains(state_id.as_str()) {
                    errors.push(format!(
                        "state_group `{}` references unknown state `{}`",
                        group.group_id, state_id
                    ));
                }
            }
        }

        for state in &self.states {
            if let Some(group_id) = &state.group_id {
                if !group_ids.contains(group_id.as_str()) {
                    errors.push(format!(
                        "state `{}` references unknown state_group `{}`",
                        state.state_id, group_id
                    ));
                }
            }
        }

        let mut transition_ids = BTreeSet::new();
        for transition in &self.transitions {
            if transition.transition_id.trim().is_empty() {
                errors.push("transition_id is required".to_string());
            } else if !transition_ids.insert(transition.transition_id.as_str()) {
                errors.push(format!(
                    "duplicate transition `{}`",
                    transition.transition_id
                ));
            }
            if !state_ids.contains(transition.from_state.as_str()) {
                errors.push(format!(
                    "transition `{}` references unknown from_state `{}`",
                    transition.transition_id, transition.from_state
                ));
            }
            if !state_ids.contains(transition.to_state.as_str()) {
                errors.push(format!(
                    "transition `{}` references unknown to_state `{}`",
                    transition.transition_id, transition.to_state
                ));
            }
            if transition.event.event_type.trim().is_empty() {
                errors.push(format!(
                    "transition `{}` must declare an event_type",
                    transition.transition_id
                ));
            }
        }

        let mut memory_names = BTreeSet::new();
        for field in &self.memory {
            if field.name.trim().is_empty() {
                errors.push("memory field name is required".to_string());
            } else if !memory_names.insert(field.name.as_str()) {
                errors.push(format!("duplicate memory field `{}`", field.name));
            }
            if field.type_name.trim().is_empty() {
                errors.push(format!(
                    "memory field `{}` must declare a type_name",
                    field.name
                ));
            }
            if field.default_value.is_none() && !field.nullable {
                errors.push(format!(
                    "memory field `{}` needs a default_value or nullable=true",
                    field.name
                ));
            }
        }

        if matches!(self.silence_policy, MachineSilencePolicy::Pinned)
            && !matches!(self.recovery_policy, MachineRecoveryPolicy::ManualRecover)
            && matches!(self.cache_policy, MachineCachePolicy::ReturnLastThenRecover)
        {
            errors.push(
                "pinned machines must not use return_last_then_recover cache semantics".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VenueCapabilityMatrix {
    #[serde(default = "default_venue_capability_matrix_version")]
    pub schema_version: String,
    pub venue_id: String,
    #[serde(default)]
    pub capabilities: Vec<VenueCapability>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionCapabilityKind {
    Market,
    Limit,
    PostOnly,
    StopMarket,
    StopLimit,
    TakeProfitMarket,
    TakeProfitLimit,
    Ioc,
    Fok,
    OcoBracket,
    TrailingStop,
    ReduceOnly,
    CloseOnly,
    OpenLong,
    CloseLong,
    OpenShort,
    CloseShort,
    OneWayPositionMode,
    HedgePositionMode,
    Gtc,
    Day,
    Gtd,
    ClientOrderId,
    CancelReplaceAmend,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilitySupportSource {
    ProviderNative,
    RuntimeSimulated,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VenueCapability {
    pub capability: ExecutionCapabilityKind,
    pub source: CapabilitySupportSource,
    #[serde(default)]
    pub supported_modes: Vec<RuntimeTradingMode>,
    #[serde(default)]
    pub constraints: BTreeMap<String, Value>,
}

pub const V4_FIRST_WAVE_EXECUTION_CAPABILITIES: [ExecutionCapabilityKind; 24] = [
    ExecutionCapabilityKind::Market,
    ExecutionCapabilityKind::Limit,
    ExecutionCapabilityKind::PostOnly,
    ExecutionCapabilityKind::StopMarket,
    ExecutionCapabilityKind::StopLimit,
    ExecutionCapabilityKind::TakeProfitMarket,
    ExecutionCapabilityKind::TakeProfitLimit,
    ExecutionCapabilityKind::Ioc,
    ExecutionCapabilityKind::Fok,
    ExecutionCapabilityKind::OcoBracket,
    ExecutionCapabilityKind::TrailingStop,
    ExecutionCapabilityKind::ReduceOnly,
    ExecutionCapabilityKind::CloseOnly,
    ExecutionCapabilityKind::OpenLong,
    ExecutionCapabilityKind::CloseLong,
    ExecutionCapabilityKind::OpenShort,
    ExecutionCapabilityKind::CloseShort,
    ExecutionCapabilityKind::OneWayPositionMode,
    ExecutionCapabilityKind::HedgePositionMode,
    ExecutionCapabilityKind::Gtc,
    ExecutionCapabilityKind::Day,
    ExecutionCapabilityKind::Gtd,
    ExecutionCapabilityKind::ClientOrderId,
    ExecutionCapabilityKind::CancelReplaceAmend,
];

pub fn v4_first_wave_execution_capabilities() -> &'static [ExecutionCapabilityKind] {
    &V4_FIRST_WAVE_EXECUTION_CAPABILITIES
}

pub fn unsupported_v4_first_wave_matrix(venue_id: impl Into<String>) -> VenueCapabilityMatrix {
    VenueCapabilityMatrix {
        schema_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
        venue_id: venue_id.into(),
        capabilities: v4_first_wave_execution_capabilities()
            .iter()
            .copied()
            .map(|capability| VenueCapability {
                capability,
                source: CapabilitySupportSource::Unsupported,
                supported_modes: Vec::new(),
                constraints: BTreeMap::new(),
            })
            .collect(),
        metadata: BTreeMap::new(),
    }
}

impl VenueCapabilityMatrix {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_VENUE_CAPABILITY_MATRIX_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_VENUE_CAPABILITY_MATRIX_VERSION
            ));
        }
        if self.venue_id.trim().is_empty() {
            errors.push("venue_id is required".to_string());
        }

        let mut seen = BTreeSet::new();
        for capability in &self.capabilities {
            if !seen.insert(&capability.capability) {
                errors.push(format!(
                    "duplicate execution capability `{:?}`",
                    capability.capability
                ));
            }
            if !matches!(capability.source, CapabilitySupportSource::Unsupported)
                && capability.supported_modes.is_empty()
            {
                errors.push(format!(
                    "capability `{:?}` needs at least one supported mode",
                    capability.capability
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn support_source(&self, capability: &ExecutionCapabilityKind) -> CapabilitySupportSource {
        self.capabilities
            .iter()
            .find(|entry| &entry.capability == capability)
            .map(|entry| entry.source.clone())
            .unwrap_or(CapabilitySupportSource::Unsupported)
    }

    pub fn require_supported(
        &self,
        capability: &ExecutionCapabilityKind,
    ) -> Result<CapabilitySupportSource, String> {
        let source = self.support_source(capability);
        if matches!(source, CapabilitySupportSource::Unsupported) {
            Err(format!(
                "execution capability `{:?}` is unsupported for venue `{}`",
                capability, self.venue_id
            ))
        } else {
            Ok(source)
        }
    }

    pub fn validate_required_capability_sources(
        &self,
        required: &[ExecutionCapabilityKind],
    ) -> Result<(), Vec<String>> {
        let mut errors = self.validate_static_contract().err().unwrap_or_default();
        let declared = self
            .capabilities
            .iter()
            .map(|entry| entry.capability)
            .collect::<BTreeSet<_>>();

        for capability in required {
            if !declared.contains(capability) {
                errors.push(format!(
                    "required execution capability `{:?}` must be explicitly marked as provider_native, runtime_simulated, or unsupported",
                    capability
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn validate_v4_first_wave_contract(&self) -> Result<(), Vec<String>> {
        self.validate_required_capability_sources(v4_first_wave_execution_capabilities())
    }
}

fn default_machine_contract_version() -> String {
    V4_MACHINE_CONTRACT_VERSION.to_string()
}

fn default_venue_capability_matrix_version() -> String {
    V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string()
}

fn default_transition_conflict_policy() -> TransitionConflictPolicy {
    TransitionConflictPolicy::Error
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_machine() -> V4MachineContract {
        V4MachineContract {
            schema_version: V4_MACHINE_CONTRACT_VERSION.to_string(),
            machine_id: "intent.trend".to_string(),
            template: MachineTemplateKind::Decision,
            states: vec![
                MachineState {
                    state_id: "idle".to_string(),
                    group_id: Some("signal_flow".to_string()),
                    initial: true,
                    terminal: false,
                },
                MachineState {
                    state_id: "long_bias".to_string(),
                    group_id: Some("signal_flow".to_string()),
                    initial: false,
                    terminal: false,
                },
            ],
            state_groups: vec![StateGroup {
                group_id: "signal_flow".to_string(),
                state_ids: vec!["idle".to_string(), "long_bias".to_string()],
                conflict_policy: TransitionConflictPolicy::Error,
                timeout_ms: None,
            }],
            transitions: vec![MachineTransition {
                transition_id: "idle_to_long".to_string(),
                from_state: "idle".to_string(),
                to_state: "long_bias".to_string(),
                event: MachineEventSelector {
                    event_type: "bar_closed".to_string(),
                    source: Some("market.btc_1m".to_string()),
                    freshness: Some(EventFreshnessRequirement::FreshOnly),
                },
                guard: Some("ema_fast > ema_slow".to_string()),
                priority: 100,
                action: Some(MachineActionSpec {
                    emits: vec!["intent.long".to_string()],
                    memory_writes: vec!["last_signal_at".to_string()],
                    diagnostics: vec!["trend_score".to_string()],
                }),
            }],
            memory: vec![MachineMemoryField {
                name: "last_signal_at".to_string(),
                type_name: "time?".to_string(),
                default_value: None,
                nullable: true,
            }],
            cache_policy: MachineCachePolicy::ReturnLastThenRecover,
            silence_policy: MachineSilencePolicy::SoftDormantAfter { ttl_ms: 30_000 },
            recovery_policy: MachineRecoveryPolicy::AsyncRecover,
            priority: 5_200,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn machine_contract_accepts_flat_state_group() {
        let machine = sample_machine();
        assert_eq!(machine.validate_static_contract(), Ok(()));
    }

    #[test]
    fn machine_contract_rejects_transition_without_event() {
        let mut machine = sample_machine();
        machine.transitions[0].event.event_type.clear();

        let errors = machine.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("must declare an event_type")));
    }

    #[test]
    fn machine_contract_rejects_unknown_transition_state() {
        let mut machine = sample_machine();
        machine.transitions[0].to_state = "nested.child".to_string();

        let errors = machine.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("unknown to_state")));
    }

    #[test]
    fn venue_matrix_rejects_duplicate_capabilities() {
        let matrix = VenueCapabilityMatrix {
            schema_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
            venue_id: "okx".to_string(),
            capabilities: vec![
                VenueCapability {
                    capability: ExecutionCapabilityKind::Market,
                    source: CapabilitySupportSource::ProviderNative,
                    supported_modes: vec![RuntimeTradingMode::PaperActual],
                    constraints: BTreeMap::new(),
                },
                VenueCapability {
                    capability: ExecutionCapabilityKind::Market,
                    source: CapabilitySupportSource::RuntimeSimulated,
                    supported_modes: vec![RuntimeTradingMode::PaperSimulated],
                    constraints: BTreeMap::new(),
                },
            ],
            metadata: BTreeMap::new(),
        };

        let errors = matrix.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("duplicate execution capability")));
    }

    #[test]
    fn venue_matrix_does_not_silently_support_missing_capability() {
        let matrix = VenueCapabilityMatrix {
            schema_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
            venue_id: "paper-local".to_string(),
            capabilities: vec![VenueCapability {
                capability: ExecutionCapabilityKind::Market,
                source: CapabilitySupportSource::RuntimeSimulated,
                supported_modes: vec![RuntimeTradingMode::PaperSimulated],
                constraints: BTreeMap::new(),
            }],
            metadata: BTreeMap::new(),
        };

        assert_eq!(
            matrix.require_supported(&ExecutionCapabilityKind::Market),
            Ok(CapabilitySupportSource::RuntimeSimulated)
        );
        assert!(matrix
            .require_supported(&ExecutionCapabilityKind::TrailingStop)
            .is_err());
    }

    #[test]
    fn venue_matrix_requires_explicit_first_wave_capability_sources() {
        let matrix = VenueCapabilityMatrix {
            schema_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
            venue_id: "paper-local".to_string(),
            capabilities: vec![VenueCapability {
                capability: ExecutionCapabilityKind::Market,
                source: CapabilitySupportSource::RuntimeSimulated,
                supported_modes: vec![RuntimeTradingMode::PaperSimulated],
                constraints: BTreeMap::new(),
            }],
            metadata: BTreeMap::new(),
        };

        assert_eq!(matrix.validate_static_contract(), Ok(()));

        let errors = matrix.validate_v4_first_wave_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("required execution capability")));
    }

    #[test]
    fn unsupported_first_wave_matrix_declares_every_source_without_supporting_them() {
        let matrix = unsupported_v4_first_wave_matrix("unknown-venue");

        assert_eq!(matrix.validate_v4_first_wave_contract(), Ok(()));
        assert_eq!(
            matrix.support_source(&ExecutionCapabilityKind::Market),
            CapabilitySupportSource::Unsupported
        );
        assert!(matrix
            .require_supported(&ExecutionCapabilityKind::Market)
            .is_err());
        assert_eq!(
            matrix.capabilities.len(),
            v4_first_wave_execution_capabilities().len()
        );
    }
}
