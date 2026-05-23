use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const V4_MACHINE_CONTRACT_VERSION: &str = "quantpilot/machine-contract/v1";
pub const V4_VENUE_CAPABILITY_MATRIX_VERSION: &str = "quantpilot/venue-capability-matrix/v1";
pub const V4_QS_STATE_MACHINE_PROFILE_VERSION: &str = "quantpilot/qs-state-machine-profile/v1";
pub const V4_MACHINE_GRAPH_CONTRACT_VERSION: &str = "quantpilot/machine-graph-contract/v1";
pub const V4_MACHINE_EVENT_CATALOG_VERSION: &str = "quantpilot/machine-event-catalog/v1";
pub const V4_RUNTIME_MODE_CONTRACT_VERSION: &str = "quantpilot/runtime-mode-contract/v1";
pub const V4_QS_TYPE_SYSTEM_VERSION: &str = "quantpilot/qs-type-system/v1";
pub const V4_STATIC_CONTRACT_BUNDLE_VERSION: &str = "quantpilot/static-contract-bundle/v1";
pub const V4_VERSION_MANIFEST_VERSION: &str = "quantpilot/version-manifest/v1";
pub const V4_PLUGIN_GOVERNANCE_VERSION: &str = "quantpilot/plugin-governance/v1";
pub const V4_REPRODUCIBILITY_CONTRACT_VERSION: &str = "quantpilot/reproducibility-contract/v1";
pub const V4_COMPLEXITY_BUDGET_CONTRACT_VERSION: &str = "quantpilot/complexity-budget/v1";
pub const V4_LEARNING_PIPELINE_CONTRACT_VERSION: &str = "quantpilot/learning-pipeline/v1";
pub const V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION: &str =
    "quantpilot/compile-time-capability-request/v1";
pub const V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION: &str =
    "quantpilot/compile-time-capability-report/v1";
pub const V4_RISK_PLANE_MIN_PRIORITY: i32 = 9_000;
pub const V4_QS_TYPE_MAX_NESTING_DEPTH: u8 = 8;

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
pub struct V4MachineGraphContract {
    #[serde(default = "default_machine_graph_contract_version")]
    pub schema_version: String,
    pub graph_id: String,
    #[serde(default)]
    pub machines: Vec<V4MachineContract>,
    #[serde(default)]
    pub edges: Vec<MachineGraphEdge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_catalog: Option<MachineEventCatalog>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_plane: Option<MachineGraphRiskPlane>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MachineGraphEdge {
    pub edge_id: String,
    pub source_machine_id: String,
    pub target_machine_id: String,
    pub event_type: String,
    #[serde(default = "default_machine_graph_edge_activation")]
    pub activation: MachineGraphEdgeActivation,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineGraphEdgeActivation {
    Always,
    RuntimeGated,
    MutedWhenUnpulled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineGraphRiskPlane {
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub machine_ids: Vec<String>,
    #[serde(default = "default_risk_plane_min_priority")]
    pub min_priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MachineEventCatalog {
    #[serde(default = "default_machine_event_catalog_version")]
    pub schema_version: String,
    #[serde(default)]
    pub events: Vec<MachineEventTypeSpec>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineEventTypeSpec {
    pub event_type: String,
    pub source_kind: MachineEventSourceKind,
    pub scope: MachineEventScope,
    #[serde(default)]
    pub payload_fields: Vec<MachineEventPayloadField>,
    #[serde(default)]
    pub allowed_emitters: Vec<String>,
    #[serde(default)]
    pub allowed_consumers: Vec<String>,
    #[serde(default = "default_true")]
    pub replayable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineEventPayloadField {
    pub name: String,
    pub type_name: String,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineEventSourceKind {
    MarketData,
    Machine,
    RiskPlane,
    VenueProvider,
    Runtime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineEventScope {
    MachineInternal,
    Graph,
    Runtime,
    Venue,
}

impl MachineEventCatalog {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_MACHINE_EVENT_CATALOG_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_MACHINE_EVENT_CATALOG_VERSION
            ));
        }
        if self.events.is_empty() {
            errors.push("event catalog must declare at least one event".to_string());
        }

        let mut event_types = BTreeSet::new();
        for event in &self.events {
            if event.event_type.trim().is_empty() {
                errors.push("event_type is required".to_string());
            } else if !event_types.insert(event.event_type.as_str()) {
                errors.push(format!("duplicate event_type `{}`", event.event_type));
            }

            let mut payload_names = BTreeSet::new();
            for field in &event.payload_fields {
                if field.name.trim().is_empty() {
                    errors.push(format!(
                        "event `{}` payload field name is required",
                        event.event_type
                    ));
                } else if !payload_names.insert(field.name.as_str()) {
                    errors.push(format!(
                        "event `{}` has duplicate payload field `{}`",
                        event.event_type, field.name
                    ));
                }
                if field.type_name.trim().is_empty() {
                    errors.push(format!(
                        "event `{}` payload field `{}` must declare a type_name",
                        event.event_type, field.name
                    ));
                }
                if field.required && field.nullable {
                    errors.push(format!(
                        "event `{}` payload field `{}` cannot be required and nullable",
                        event.event_type, field.name
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl V4MachineGraphContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_MACHINE_GRAPH_CONTRACT_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_MACHINE_GRAPH_CONTRACT_VERSION
            ));
        }
        if self.graph_id.trim().is_empty() {
            errors.push("graph_id is required".to_string());
        }
        if self.machines.is_empty() {
            errors.push("at least one machine is required".to_string());
        }

        let mut machines_by_id = BTreeMap::new();
        for machine in &self.machines {
            if machine.machine_id.trim().is_empty() {
                errors.push("machine_id is required".to_string());
            } else if machines_by_id
                .insert(machine.machine_id.as_str(), machine)
                .is_some()
            {
                errors.push(format!("duplicate machine `{}`", machine.machine_id));
            }

            if let Err(machine_errors) = machine.validate_static_contract() {
                for machine_error in machine_errors {
                    errors.push(format!(
                        "machine `{}` failed static contract: {}",
                        machine.machine_id, machine_error
                    ));
                }
            }
        }

        let mut edge_ids = BTreeSet::new();
        for edge in &self.edges {
            if edge.edge_id.trim().is_empty() {
                errors.push("edge_id is required".to_string());
            } else if !edge_ids.insert(edge.edge_id.as_str()) {
                errors.push(format!("duplicate edge `{}`", edge.edge_id));
            }
            if edge.event_type.trim().is_empty() {
                errors.push(format!(
                    "edge `{}` must declare an event_type",
                    edge.edge_id
                ));
            }
            if !machines_by_id.contains_key(edge.source_machine_id.as_str()) {
                errors.push(format!(
                    "edge `{}` references unknown source_machine_id `{}`",
                    edge.edge_id, edge.source_machine_id
                ));
            }
            if !machines_by_id.contains_key(edge.target_machine_id.as_str()) {
                errors.push(format!(
                    "edge `{}` references unknown target_machine_id `{}`",
                    edge.edge_id, edge.target_machine_id
                ));
            }
            if edge.source_machine_id == edge.target_machine_id {
                errors.push(format!(
                    "edge `{}` must not connect a machine to itself",
                    edge.edge_id
                ));
            }
        }

        errors.extend(self.validate_graph_acyclic().err().unwrap_or_default());
        errors.extend(self.validate_event_catalog(&machines_by_id));
        errors.extend(self.validate_risk_plane(&machines_by_id));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_graph_acyclic(&self) -> Result<(), Vec<String>> {
        let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for edge in &self.edges {
            adjacency
                .entry(edge.source_machine_id.as_str())
                .or_default()
                .push(edge.target_machine_id.as_str());
        }

        let mut visited = BTreeSet::new();
        let mut in_stack = BTreeSet::new();
        let mut cycle_path = Vec::new();

        fn dfs<'a>(
            node: &'a str,
            adjacency: &BTreeMap<&'a str, Vec<&'a str>>,
            visited: &mut BTreeSet<&'a str>,
            in_stack: &mut BTreeSet<&'a str>,
            cycle_path: &mut Vec<String>,
        ) -> bool {
            visited.insert(node);
            in_stack.insert(node);
            if let Some(neighbors) = adjacency.get(node) {
                for &next in neighbors {
                    if !visited.contains(next) {
                        if dfs(next, adjacency, visited, in_stack, cycle_path) {
                            cycle_path.push(node.to_string());
                            return true;
                        }
                    } else if in_stack.contains(next) {
                        cycle_path.push(next.to_string());
                        cycle_path.push(node.to_string());
                        return true;
                    }
                }
            }
            in_stack.remove(node);
            false
        }

        for edge in &self.edges {
            let source = edge.source_machine_id.as_str();
            if !visited.contains(source)
                && dfs(
                    source,
                    &adjacency,
                    &mut visited,
                    &mut in_stack,
                    &mut cycle_path,
                )
            {
                cycle_path.reverse();
                return Err(vec![format!(
                    "machine graph must be acyclic, cycle: {}",
                    cycle_path.join(" -> ")
                )]);
            }
        }

        Ok(())
    }

    fn validate_event_catalog(
        &self,
        machines_by_id: &BTreeMap<&str, &V4MachineContract>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        let mut referenced_events = BTreeSet::new();

        for machine in &self.machines {
            for transition in &machine.transitions {
                if !transition.event.event_type.trim().is_empty() {
                    referenced_events.insert(transition.event.event_type.as_str());
                }
                if let Some(action) = &transition.action {
                    for event_type in &action.emits {
                        if event_type.trim().is_empty() {
                            errors.push(format!(
                                "machine `{}` transition `{}` action emits an empty event_type",
                                machine.machine_id, transition.transition_id
                            ));
                        } else {
                            referenced_events.insert(event_type.as_str());
                        }
                    }
                }
            }
        }
        for edge in &self.edges {
            if !edge.event_type.trim().is_empty() {
                referenced_events.insert(edge.event_type.as_str());
            }
        }

        let Some(catalog) = &self.event_catalog else {
            if !referenced_events.is_empty() {
                errors.push(
                    "machine graph with transition or edge events must declare event_catalog"
                        .to_string(),
                );
            }
            return errors;
        };

        errors.extend(catalog.validate_static_contract().err().unwrap_or_default());

        let event_specs = catalog
            .events
            .iter()
            .map(|event| (event.event_type.as_str(), event))
            .collect::<BTreeMap<_, _>>();

        for event_type in referenced_events {
            if !event_specs.contains_key(event_type) {
                errors.push(format!(
                    "event_type `{}` must be declared in event_catalog",
                    event_type
                ));
            }
        }

        for machine in &self.machines {
            for transition in &machine.transitions {
                let Some(spec) = event_specs.get(transition.event.event_type.as_str()) else {
                    continue;
                };

                if !machine_event_party_allowed(&spec.allowed_consumers, &machine.machine_id) {
                    errors.push(format!(
                        "machine `{}` transition `{}` is not an allowed consumer of event `{}`",
                        machine.machine_id, transition.transition_id, transition.event.event_type
                    ));
                }
                if let Some(source) = &transition.event.source {
                    if !machine_event_party_allowed(&spec.allowed_emitters, source) {
                        errors.push(format!(
                            "machine `{}` transition `{}` source `{}` is not an allowed emitter of event `{}`",
                            machine.machine_id,
                            transition.transition_id,
                            source,
                            transition.event.event_type
                        ));
                    }
                }

                if let Some(action) = &transition.action {
                    for event_type in &action.emits {
                        let Some(emitted_spec) = event_specs.get(event_type.as_str()) else {
                            continue;
                        };
                        if !machine_event_party_allowed(
                            &emitted_spec.allowed_emitters,
                            &machine.machine_id,
                        ) {
                            errors.push(format!(
                                "machine `{}` transition `{}` cannot emit event `{}`",
                                machine.machine_id, transition.transition_id, event_type
                            ));
                        }
                    }
                }
            }
        }

        for edge in &self.edges {
            let Some(spec) = event_specs.get(edge.event_type.as_str()) else {
                continue;
            };
            if !machines_by_id.contains_key(edge.source_machine_id.as_str())
                || !machines_by_id.contains_key(edge.target_machine_id.as_str())
            {
                continue;
            }
            if !machine_event_party_allowed(&spec.allowed_emitters, &edge.source_machine_id) {
                errors.push(format!(
                    "edge `{}` source `{}` is not an allowed emitter of event `{}`",
                    edge.edge_id, edge.source_machine_id, edge.event_type
                ));
            }
            if !machine_event_party_allowed(&spec.allowed_consumers, &edge.target_machine_id) {
                errors.push(format!(
                    "edge `{}` target `{}` is not an allowed consumer of event `{}`",
                    edge.edge_id, edge.target_machine_id, edge.event_type
                ));
            }
        }

        errors
    }

    fn validate_risk_plane(
        &self,
        machines_by_id: &BTreeMap<&str, &V4MachineContract>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        let execution_machine_ids = self
            .machines
            .iter()
            .filter(|machine| matches!(machine.template, MachineTemplateKind::Execution))
            .map(|machine| machine.machine_id.as_str())
            .collect::<BTreeSet<_>>();

        let Some(risk_plane) = &self.risk_plane else {
            if !execution_machine_ids.is_empty() {
                errors.push(
                    "execution machine graphs must declare a dedicated risk_plane".to_string(),
                );
            }
            return errors;
        };

        if !execution_machine_ids.is_empty() && !risk_plane.required {
            errors.push("execution machine graphs must require the risk_plane".to_string());
        }
        if risk_plane.required && risk_plane.machine_ids.is_empty() {
            errors.push("required risk_plane must list at least one machine_id".to_string());
        }
        if risk_plane.min_priority < V4_RISK_PLANE_MIN_PRIORITY {
            errors.push(format!(
                "risk_plane min_priority must be at least {}",
                V4_RISK_PLANE_MIN_PRIORITY
            ));
        }

        let risk_machine_ids = risk_plane
            .machine_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();

        for machine_id in &risk_plane.machine_ids {
            match machines_by_id.get(machine_id.as_str()) {
                Some(machine) => {
                    if !matches!(machine.template, MachineTemplateKind::Decision) {
                        errors.push(format!(
                            "risk_plane machine `{}` must use Decision template",
                            machine_id
                        ));
                    }
                    if machine.priority < risk_plane.min_priority {
                        errors.push(format!(
                            "risk_plane machine `{}` priority {} is below min_priority {}",
                            machine_id, machine.priority, risk_plane.min_priority
                        ));
                    }
                }
                None => errors.push(format!(
                    "risk_plane references unknown machine `{}`",
                    machine_id
                )),
            }
        }

        for execution_machine_id in &execution_machine_ids {
            let mut has_risk_inbound_edge = false;
            for edge in self
                .edges
                .iter()
                .filter(|edge| edge.target_machine_id == *execution_machine_id)
            {
                if risk_machine_ids.contains(edge.source_machine_id.as_str()) {
                    has_risk_inbound_edge = true;
                } else {
                    errors.push(format!(
                        "execution machine `{}` inbound edge `{}` must originate from risk_plane",
                        execution_machine_id, edge.edge_id
                    ));
                }
            }
            if !has_risk_inbound_edge {
                errors.push(format!(
                    "execution machine `{}` must have an inbound edge from risk_plane",
                    execution_machine_id
                ));
            }
        }

        errors
    }
}

fn machine_event_party_allowed(allowed_parties: &[String], party: &str) -> bool {
    allowed_parties.is_empty() || allowed_parties.iter().any(|allowed| allowed == party)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QsStateMachineProfile {
    #[serde(default = "default_qs_state_machine_profile_version")]
    pub schema_version: String,
    #[serde(default = "default_qs_machine_templates")]
    pub allowed_templates: Vec<MachineTemplateKind>,
    #[serde(default)]
    pub state_policy: QsStatePolicy,
    #[serde(default)]
    pub action_block_policy: QsActionBlockPolicy,
    #[serde(default)]
    pub memory_policy: QsMemoryPolicy,
    #[serde(default)]
    pub event_policy: QsEventPolicy,
    #[serde(default)]
    pub priority_policy: QsPriorityPolicy,
    #[serde(default)]
    pub risk_plane_policy: QsRiskPlanePolicy,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QsStatePolicy {
    #[serde(default = "default_true")]
    pub allow_state_groups: bool,
    #[serde(default)]
    pub allow_nested_state_machines: bool,
    #[serde(default = "default_true")]
    pub nested_state_machine_warning_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_states: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_state_groups: Option<u32>,
}

impl Default for QsStatePolicy {
    fn default() -> Self {
        Self {
            allow_state_groups: true,
            allow_nested_state_machines: false,
            nested_state_machine_warning_required: true,
            max_states: Some(256),
            max_state_groups: Some(64),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QsActionBlockPolicy {
    #[serde(default = "default_true")]
    pub allow_emit: bool,
    #[serde(default = "default_true")]
    pub allow_memory_writes: bool,
    #[serde(default = "default_true")]
    pub allow_diagnostics: bool,
    #[serde(default)]
    pub allow_network_access: bool,
    #[serde(default)]
    pub allow_file_access: bool,
    #[serde(default)]
    pub allow_direct_order_submit: bool,
    #[serde(default)]
    pub allow_cross_machine_memory_write: bool,
    #[serde(default)]
    pub allow_unbounded_loop: bool,
    #[serde(default)]
    pub allow_dynamic_eval: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_loop_iterations: Option<u32>,
}

impl Default for QsActionBlockPolicy {
    fn default() -> Self {
        Self {
            allow_emit: true,
            allow_memory_writes: true,
            allow_diagnostics: true,
            allow_network_access: false,
            allow_file_access: false,
            allow_direct_order_submit: false,
            allow_cross_machine_memory_write: false,
            allow_unbounded_loop: false,
            allow_dynamic_eval: false,
            max_loop_iterations: Some(1_024),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QsMemoryPolicy {
    #[serde(default = "default_true")]
    pub typed_memory_required: bool,
    #[serde(default = "default_true")]
    pub default_or_nullable_required: bool,
    #[serde(default = "default_true")]
    pub machine_private_memory_required: bool,
}

impl Default for QsMemoryPolicy {
    fn default() -> Self {
        Self {
            typed_memory_required: true,
            default_or_nullable_required: true,
            machine_private_memory_required: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QsEventPolicy {
    #[serde(default = "default_true")]
    pub transition_event_required: bool,
    #[serde(default = "default_true")]
    pub strong_typed_events_required: bool,
    #[serde(default)]
    pub allow_polling_transition_without_event: bool,
}

impl Default for QsEventPolicy {
    fn default() -> Self {
        Self {
            transition_event_required: true,
            strong_typed_events_required: true,
            allow_polling_transition_without_event: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QsPriorityPolicy {
    #[serde(default = "default_true")]
    pub user_defined_priority_allowed: bool,
    #[serde(default = "default_true")]
    pub deterministic_conflict_resolution_required: bool,
    #[serde(default = "default_true")]
    pub risk_first_conflict_policy_allowed: bool,
}

impl Default for QsPriorityPolicy {
    fn default() -> Self {
        Self {
            user_defined_priority_allowed: true,
            deterministic_conflict_resolution_required: true,
            risk_first_conflict_policy_allowed: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QsRiskPlanePolicy {
    #[serde(default = "default_true")]
    pub decision_machine_can_express_risk: bool,
    #[serde(default = "default_true")]
    pub dedicated_high_priority_risk_plane_required: bool,
    #[serde(default)]
    pub qs_can_bypass_risk_plane: bool,
}

impl Default for QsRiskPlanePolicy {
    fn default() -> Self {
        Self {
            decision_machine_can_express_risk: true,
            dedicated_high_priority_risk_plane_required: true,
            qs_can_bypass_risk_plane: false,
        }
    }
}

impl Default for QsStateMachineProfile {
    fn default() -> Self {
        default_v4_qs_state_machine_profile()
    }
}

pub fn default_v4_qs_state_machine_profile() -> QsStateMachineProfile {
    QsStateMachineProfile {
        schema_version: V4_QS_STATE_MACHINE_PROFILE_VERSION.to_string(),
        allowed_templates: default_qs_machine_templates(),
        state_policy: QsStatePolicy::default(),
        action_block_policy: QsActionBlockPolicy::default(),
        memory_policy: QsMemoryPolicy::default(),
        event_policy: QsEventPolicy::default(),
        priority_policy: QsPriorityPolicy::default(),
        risk_plane_policy: QsRiskPlanePolicy::default(),
        metadata: BTreeMap::new(),
    }
}

impl QsStateMachineProfile {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_QS_STATE_MACHINE_PROFILE_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_QS_STATE_MACHINE_PROFILE_VERSION
            ));
        }

        let mut template_set = BTreeSet::new();
        for template in &self.allowed_templates {
            if !template_set.insert(template.clone()) {
                errors.push(format!("duplicate machine template `{:?}`", template));
            }
        }
        for template in [
            MachineTemplateKind::Observation,
            MachineTemplateKind::Decision,
            MachineTemplateKind::Execution,
        ] {
            if !template_set.contains(&template) {
                errors.push(format!(
                    "QS state machine profile must allow `{:?}` template",
                    template
                ));
            }
        }

        if !self.state_policy.allow_state_groups {
            errors.push("QS state machine profile must allow state_group".to_string());
        }
        if self.state_policy.allow_nested_state_machines {
            errors.push(
                "nested state machines are reserved for future versions and must stay disabled"
                    .to_string(),
            );
        }
        if !self.state_policy.nested_state_machine_warning_required {
            errors.push(
                "future nested state machines must require a performance warning".to_string(),
            );
        }
        if matches!(self.state_policy.max_states, Some(0)) {
            errors.push("max_states must be greater than 0 when declared".to_string());
        }
        if matches!(self.state_policy.max_state_groups, Some(0)) {
            errors.push("max_state_groups must be greater than 0 when declared".to_string());
        }

        if !self.action_block_policy.allow_emit {
            errors.push("controlled action blocks must allow event emission".to_string());
        }
        if !self.action_block_policy.allow_memory_writes {
            errors.push("controlled action blocks must allow typed memory writes".to_string());
        }
        if !self.action_block_policy.allow_diagnostics {
            errors.push("controlled action blocks must allow diagnostics".to_string());
        }
        if self.action_block_policy.allow_network_access {
            errors.push("controlled action blocks must not allow network access".to_string());
        }
        if self.action_block_policy.allow_file_access {
            errors.push("controlled action blocks must not allow file access".to_string());
        }
        if self.action_block_policy.allow_direct_order_submit {
            errors.push("QS action blocks must not submit orders directly".to_string());
        }
        if self.action_block_policy.allow_cross_machine_memory_write {
            errors.push(
                "QS action blocks must not write memory owned by another machine".to_string(),
            );
        }
        if self.action_block_policy.allow_unbounded_loop {
            errors.push("QS action blocks must not allow unbounded loops".to_string());
        }
        if self.action_block_policy.max_loop_iterations.is_none()
            && !self.action_block_policy.allow_unbounded_loop
        {
            errors.push("bounded QS action blocks must declare max_loop_iterations".to_string());
        }
        if self.action_block_policy.allow_dynamic_eval {
            errors.push("QS action blocks must not allow dynamic eval".to_string());
        }

        if !self.memory_policy.typed_memory_required {
            errors.push("QS machine memory must be strongly typed".to_string());
        }
        if !self.memory_policy.default_or_nullable_required {
            errors
                .push("QS machine memory must require default_value or nullable=true".to_string());
        }
        if !self.memory_policy.machine_private_memory_required {
            errors.push("QS machine memory must stay private to the owning machine".to_string());
        }

        if !self.event_policy.transition_event_required {
            errors.push("QS transitions must require an event".to_string());
        }
        if !self.event_policy.strong_typed_events_required {
            errors.push("QS transition events must be strongly typed".to_string());
        }
        if self.event_policy.allow_polling_transition_without_event {
            errors.push("QS transitions must not poll without an event".to_string());
        }

        if !self.priority_policy.user_defined_priority_allowed {
            errors.push("QS profile must allow user-defined priority".to_string());
        }
        if !self
            .priority_policy
            .deterministic_conflict_resolution_required
        {
            errors.push("QS profile must require deterministic conflict resolution".to_string());
        }
        if !self.priority_policy.risk_first_conflict_policy_allowed {
            errors.push("QS profile must allow risk_first conflict policy".to_string());
        }

        if !self.risk_plane_policy.decision_machine_can_express_risk {
            errors.push("DecisionMachine must be able to express risk decisions".to_string());
        }
        if !self
            .risk_plane_policy
            .dedicated_high_priority_risk_plane_required
        {
            errors
                .push("runtime must keep a dedicated high-priority risk safety plane".to_string());
        }
        if self.risk_plane_policy.qs_can_bypass_risk_plane {
            errors.push("QS must not be able to bypass the runtime risk plane".to_string());
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeModeContract {
    #[serde(default = "default_runtime_mode_contract_version")]
    pub schema_version: String,
    #[serde(default = "default_runtime_mode_specs")]
    pub modes: Vec<RuntimeTradingModeSpec>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeTradingModeSpec {
    pub mode: RuntimeTradingMode,
    pub account_domain: RuntimeAccountDomain,
    pub settlement_authority: RuntimeSettlementAuthority,
    pub execution_event_source: RuntimeExecutionEventSource,
    #[serde(default)]
    pub provider_order_submission_allowed: bool,
    #[serde(default)]
    pub provider_fill_required: bool,
    #[serde(default)]
    pub local_fill_engine_required: bool,
    #[serde(default)]
    pub local_ledger_required: bool,
    #[serde(default)]
    pub provider_account_context_required: bool,
    #[serde(default = "default_true")]
    pub risk_plane_required: bool,
    #[serde(default = "default_runtime_execution_events")]
    pub required_events: Vec<RuntimeExecutionEventKind>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeAccountDomain {
    Paper,
    Live,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSettlementAuthority {
    ProviderActual,
    LocalSimulated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeExecutionEventSource {
    ProviderActual,
    LocalSimulated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeExecutionEventKind {
    OrderAcknowledged,
    OrderRejected,
    OrderPartiallyFilled,
    OrderFilled,
    FeeCharged,
    PortfolioChanged,
}

pub const V4_RUNTIME_EXECUTION_EVENTS: [RuntimeExecutionEventKind; 6] = [
    RuntimeExecutionEventKind::OrderAcknowledged,
    RuntimeExecutionEventKind::OrderRejected,
    RuntimeExecutionEventKind::OrderPartiallyFilled,
    RuntimeExecutionEventKind::OrderFilled,
    RuntimeExecutionEventKind::FeeCharged,
    RuntimeExecutionEventKind::PortfolioChanged,
];

pub fn v4_runtime_execution_events() -> &'static [RuntimeExecutionEventKind] {
    &V4_RUNTIME_EXECUTION_EVENTS
}

pub fn default_v4_runtime_mode_contract() -> RuntimeModeContract {
    RuntimeModeContract {
        schema_version: V4_RUNTIME_MODE_CONTRACT_VERSION.to_string(),
        modes: default_runtime_mode_specs(),
        metadata: BTreeMap::new(),
    }
}

impl Default for RuntimeModeContract {
    fn default() -> Self {
        default_v4_runtime_mode_contract()
    }
}

impl RuntimeModeContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_RUNTIME_MODE_CONTRACT_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_RUNTIME_MODE_CONTRACT_VERSION
            ));
        }

        let mut seen_modes = BTreeSet::new();
        for spec in &self.modes {
            if !seen_modes.insert(spec.mode) {
                errors.push(format!("duplicate runtime trading mode `{:?}`", spec.mode));
            }
            errors.extend(validate_runtime_mode_spec(spec));
        }

        for mode in required_runtime_trading_modes() {
            if !seen_modes.contains(&mode) {
                errors.push(format!("runtime mode contract must declare `{:?}`", mode));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn mode_spec(&self, mode: RuntimeTradingMode) -> Option<&RuntimeTradingModeSpec> {
        self.modes.iter().find(|spec| spec.mode == mode)
    }

    pub fn settlement_authority_for(
        &self,
        mode: RuntimeTradingMode,
    ) -> Option<RuntimeSettlementAuthority> {
        self.mode_spec(mode).map(|spec| spec.settlement_authority)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QsTypeSystemContract {
    #[serde(default = "default_qs_type_system_version")]
    pub schema_version: String,
    #[serde(default = "default_qs_scalar_types")]
    pub scalar_types: Vec<QsScalarTypeKind>,
    #[serde(default = "default_qs_composite_type_specs")]
    pub composite_types: Vec<QsCompositeTypeSpec>,
    #[serde(default = "default_qs_type_max_nesting_depth")]
    pub max_nesting_depth: u8,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum QsScalarTypeKind {
    Bool,
    Int,
    Decimal,
    Time,
    Duration,
    Price,
    Quantity,
    Notional,
    Percent,
    Ratio,
    Fee,
    Slippage,
    Leverage,
    Symbol,
    Venue,
    Account,
    Side,
    PositionSide,
    OrderType,
    TimeInForce,
    Freshness,
    RuntimeMode,
    OrderPermission,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum QsCompositeTypeKind {
    Optional,
    List,
    Map,
    Fresh,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QsCompositeTypeSpec {
    pub kind: QsCompositeTypeKind,
    #[serde(default)]
    pub max_items_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items_upper_bound: Option<u32>,
    #[serde(default = "default_true")]
    pub replay_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QsTypeRef {
    Scalar {
        scalar: QsScalarTypeKind,
    },
    Optional {
        inner: Box<QsTypeRef>,
    },
    List {
        item: Box<QsTypeRef>,
        max_items: u32,
    },
    Map {
        key: QsScalarTypeKind,
        value: Box<QsTypeRef>,
        max_items: u32,
    },
    Fresh {
        inner: Box<QsTypeRef>,
    },
    Stale {
        inner: Box<QsTypeRef>,
    },
}

pub const V4_FIRST_WAVE_QS_SCALAR_TYPES: [QsScalarTypeKind; 23] = [
    QsScalarTypeKind::Bool,
    QsScalarTypeKind::Int,
    QsScalarTypeKind::Decimal,
    QsScalarTypeKind::Time,
    QsScalarTypeKind::Duration,
    QsScalarTypeKind::Price,
    QsScalarTypeKind::Quantity,
    QsScalarTypeKind::Notional,
    QsScalarTypeKind::Percent,
    QsScalarTypeKind::Ratio,
    QsScalarTypeKind::Fee,
    QsScalarTypeKind::Slippage,
    QsScalarTypeKind::Leverage,
    QsScalarTypeKind::Symbol,
    QsScalarTypeKind::Venue,
    QsScalarTypeKind::Account,
    QsScalarTypeKind::Side,
    QsScalarTypeKind::PositionSide,
    QsScalarTypeKind::OrderType,
    QsScalarTypeKind::TimeInForce,
    QsScalarTypeKind::Freshness,
    QsScalarTypeKind::RuntimeMode,
    QsScalarTypeKind::OrderPermission,
];

pub const V4_FIRST_WAVE_QS_COMPOSITE_TYPES: [QsCompositeTypeKind; 5] = [
    QsCompositeTypeKind::Optional,
    QsCompositeTypeKind::List,
    QsCompositeTypeKind::Map,
    QsCompositeTypeKind::Fresh,
    QsCompositeTypeKind::Stale,
];

pub fn v4_first_wave_scalar_types() -> &'static [QsScalarTypeKind] {
    &V4_FIRST_WAVE_QS_SCALAR_TYPES
}

pub fn v4_first_wave_composite_types() -> &'static [QsCompositeTypeKind] {
    &V4_FIRST_WAVE_QS_COMPOSITE_TYPES
}

impl Default for QsTypeSystemContract {
    fn default() -> Self {
        default_v4_qs_type_system_contract()
    }
}

pub fn default_v4_qs_type_system_contract() -> QsTypeSystemContract {
    QsTypeSystemContract {
        schema_version: V4_QS_TYPE_SYSTEM_VERSION.to_string(),
        scalar_types: default_qs_scalar_types(),
        composite_types: default_qs_composite_type_specs(),
        max_nesting_depth: V4_QS_TYPE_MAX_NESTING_DEPTH,
        metadata: BTreeMap::new(),
    }
}

impl QsTypeSystemContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_QS_TYPE_SYSTEM_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_QS_TYPE_SYSTEM_VERSION
            ));
        }
        if self.max_nesting_depth == 0 {
            errors.push("max_nesting_depth must be greater than 0".to_string());
        }

        let mut scalar_set = BTreeSet::new();
        for scalar in &self.scalar_types {
            if !scalar_set.insert(*scalar) {
                errors.push(format!("duplicate scalar type `{:?}`", scalar));
            }
        }
        for scalar in v4_first_wave_scalar_types() {
            if !scalar_set.contains(scalar) {
                errors.push(format!(
                    "QS type system must declare scalar type `{:?}`",
                    scalar
                ));
            }
        }

        let mut composite_specs = BTreeMap::new();
        for spec in &self.composite_types {
            if composite_specs.insert(spec.kind, spec).is_some() {
                errors.push(format!("duplicate composite type `{:?}`", spec.kind));
            }
            if spec.max_items_required {
                match spec.max_items_upper_bound {
                    Some(0) => errors.push(format!(
                        "composite type `{:?}` max_items_upper_bound must be greater than 0",
                        spec.kind
                    )),
                    None => errors.push(format!(
                        "composite type `{:?}` requires max_items_upper_bound",
                        spec.kind
                    )),
                    Some(_) => {}
                }
            }
            if !spec.replay_safe {
                errors.push(format!(
                    "composite type `{:?}` must be replay_safe",
                    spec.kind
                ));
            }
        }
        for composite in v4_first_wave_composite_types() {
            if !composite_specs.contains_key(composite) {
                errors.push(format!(
                    "QS type system must declare composite type `{:?}`",
                    composite
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn validate_type_ref(&self, type_ref: &QsTypeRef) -> Result<(), Vec<String>> {
        let mut errors = self.validate_static_contract().err().unwrap_or_default();
        self.validate_type_ref_inner(type_ref, 1, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_type_ref_inner(&self, type_ref: &QsTypeRef, depth: u8, errors: &mut Vec<String>) {
        if depth > self.max_nesting_depth {
            errors.push(format!(
                "QS type nesting depth {} exceeds max_nesting_depth {}",
                depth, self.max_nesting_depth
            ));
            return;
        }

        match type_ref {
            QsTypeRef::Scalar { scalar } => {
                if !self.scalar_types.contains(scalar) {
                    errors.push(format!("scalar type `{:?}` is not declared", scalar));
                }
            }
            QsTypeRef::Optional { inner } => {
                self.validate_composite_ref(QsCompositeTypeKind::Optional, None, errors);
                self.validate_type_ref_inner(inner, depth + 1, errors);
            }
            QsTypeRef::List { item, max_items } => {
                self.validate_composite_ref(QsCompositeTypeKind::List, Some(*max_items), errors);
                self.validate_type_ref_inner(item, depth + 1, errors);
            }
            QsTypeRef::Map {
                key,
                value,
                max_items,
            } => {
                self.validate_composite_ref(QsCompositeTypeKind::Map, Some(*max_items), errors);
                if !self.scalar_types.contains(key) {
                    errors.push(format!("map key scalar type `{:?}` is not declared", key));
                }
                self.validate_type_ref_inner(value, depth + 1, errors);
            }
            QsTypeRef::Fresh { inner } => {
                self.validate_composite_ref(QsCompositeTypeKind::Fresh, None, errors);
                self.validate_type_ref_inner(inner, depth + 1, errors);
            }
            QsTypeRef::Stale { inner } => {
                self.validate_composite_ref(QsCompositeTypeKind::Stale, None, errors);
                self.validate_type_ref_inner(inner, depth + 1, errors);
            }
        }
    }

    fn validate_composite_ref(
        &self,
        kind: QsCompositeTypeKind,
        max_items: Option<u32>,
        errors: &mut Vec<String>,
    ) {
        let Some(spec) = self.composite_types.iter().find(|spec| spec.kind == kind) else {
            errors.push(format!("composite type `{:?}` is not declared", kind));
            return;
        };

        if spec.max_items_required {
            match max_items {
                Some(0) | None => errors.push(format!(
                    "composite type `{:?}` requires max_items greater than 0",
                    kind
                )),
                Some(value) => {
                    if let Some(limit) = spec.max_items_upper_bound {
                        if value > limit {
                            errors.push(format!(
                                "composite type `{:?}` max_items {} exceeds upper bound {}",
                                kind, value, limit
                            ));
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4VersionManifest {
    #[serde(default = "default_version_manifest_version")]
    pub schema_version: String,
    #[serde(default = "default_qs_language_version")]
    pub qs_language_version: String,
    #[serde(default = "default_qs_type_system_version")]
    pub type_schema_version: String,
    #[serde(default = "default_machine_contract_version")]
    pub machine_template_version: String,
    #[serde(default = "default_venue_capability_matrix_version")]
    pub capability_matrix_version: String,
    #[serde(default = "default_true")]
    pub additive_types_are_compatible: bool,
    #[serde(default = "default_true")]
    pub additive_defaulted_fields_are_compatible: bool,
    #[serde(default = "default_true")]
    pub type_tightening_requires_migration: bool,
    #[serde(default = "default_true")]
    pub type_deletion_requires_deprecation_first: bool,
    #[serde(default = "default_true")]
    pub semantic_change_requires_schema_bump: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginGovernanceContract {
    #[serde(default = "default_plugin_governance_version")]
    pub schema_version: String,
    #[serde(default = "default_plugin_kinds")]
    pub allowed_kinds: Vec<PluginKind>,
    #[serde(default = "default_plugin_required_fields")]
    pub required_fields: Vec<PluginManifestField>,
    #[serde(default = "default_true")]
    pub qs_declares_capabilities_only: bool,
    #[serde(default = "default_true")]
    pub real_order_requires_venue_plugin_and_risk_plane: bool,
    #[serde(default = "default_true")]
    pub pure_plugins_must_be_deterministic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginManifestSpec {
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub kind: PluginKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<QsTypeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<QsTypeRef>,
    #[serde(default)]
    pub deterministic: bool,
    pub side_effect: PluginSideEffect,
    pub runtime_permission: PluginRuntimePermission,
    pub network_permission: PluginNetworkPermission,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_matrix: Option<VenueCapabilityMatrix>,
    pub test_fixture_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    Pure,
    Runtime,
    Venue,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PluginManifestField {
    Name,
    Version,
    InputSchema,
    OutputSchema,
    Deterministic,
    SideEffect,
    RuntimePermission,
    NetworkPermission,
    CapabilityMatrix,
    TestFixture,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginSideEffect {
    None,
    LocalRuntimeState,
    ProviderNetwork,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginRuntimePermission {
    None,
    LocalSimulation,
    RuntimeState,
    VenueAdapter,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginNetworkPermission {
    None,
    ProviderOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReproducibilityContract {
    #[serde(default = "default_reproducibility_contract_version")]
    pub schema_version: String,
    #[serde(default = "default_reproducibility_evidence")]
    pub required_evidence: Vec<RunEvidenceKind>,
    #[serde(default = "default_event_envelope_fields")]
    pub required_event_envelope_fields: Vec<EventEnvelopeField>,
    #[serde(default = "default_true")]
    pub key_decision_path_replay_required: bool,
    #[serde(default)]
    pub full_tick_replay_required: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RunEvidenceKind {
    StrategyRunId,
    EventSequence,
    InputSnapshotId,
    MemoryChangeLog,
    CapabilityHash,
    DeploymentRevision,
    OrderCapabilitySource,
    RiskDecisionEvidence,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventEnvelopeField {
    EventId,
    EventType,
    EventTime,
    Source,
    Payload,
    Freshness,
    Sequence,
    Replayable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexityBudgetContract {
    #[serde(default = "default_complexity_budget_contract_version")]
    pub schema_version: String,
    pub max_state_count: u32,
    pub max_transition_count: u32,
    pub max_memory_field_count: u32,
    pub max_plugin_call_count: u32,
    pub max_mode_count: u32,
    pub max_stale_dependency_count: u32,
    pub max_estimated_order_paths: u32,
    pub max_event_rate_estimate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComplexityMetrics {
    pub state_count: u32,
    pub transition_count: u32,
    pub memory_field_count: u32,
    pub plugin_call_count: u32,
    pub mode_count: u32,
    pub stale_dependency_count: u32,
    pub estimated_order_paths: u32,
    pub event_rate_estimate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeveloperLearningPipelineContract {
    #[serde(default = "default_learning_pipeline_contract_version")]
    pub schema_version: String,
    #[serde(default = "default_true")]
    pub core_pipeline_in_repo: bool,
    #[serde(default = "default_learning_dir")]
    pub local_learning_dir: String,
    #[serde(default = "default_true")]
    pub local_learning_dir_gitignored: bool,
    #[serde(default = "default_true")]
    pub write_requires_explicit_user_command: bool,
    #[serde(default)]
    pub included_in_regular_gates: bool,
    #[serde(default = "default_true")]
    pub major_closeout_question_required: bool,
    #[serde(default = "default_true")]
    pub owner_first_iteration_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct V4StaticContractBundle {
    #[serde(default = "default_static_contract_bundle_version")]
    pub schema_version: String,
    #[serde(default)]
    pub version_manifest: V4VersionManifest,
    #[serde(default)]
    pub qs_profile: QsStateMachineProfile,
    #[serde(default)]
    pub type_system: QsTypeSystemContract,
    #[serde(default)]
    pub runtime_modes: RuntimeModeContract,
    #[serde(default)]
    pub plugin_governance: PluginGovernanceContract,
    #[serde(default)]
    pub reproducibility: ReproducibilityContract,
    #[serde(default)]
    pub complexity_budget: ComplexityBudgetContract,
    #[serde(default)]
    pub learning_pipeline: DeveloperLearningPipelineContract,
    #[serde(default)]
    pub machine_graphs: Vec<V4MachineGraphContract>,
    #[serde(default)]
    pub venue_matrices: Vec<VenueCapabilityMatrix>,
    #[serde(default)]
    pub plugin_manifests: Vec<PluginManifestSpec>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V4CompileTimeCapabilityRequest {
    #[serde(default = "default_compile_time_capability_request_version")]
    pub schema_version: String,
    pub graph_id: String,
    pub venue_id: String,
    pub runtime_mode: RuntimeTradingMode,
    #[serde(default)]
    pub required_execution_capabilities: Vec<ExecutionCapabilityKind>,
    #[serde(default)]
    pub required_type_refs: Vec<QsTypeRef>,
    #[serde(default)]
    pub required_plugin_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V4CompileTimeCapabilityReport {
    #[serde(default = "default_compile_time_capability_report_version")]
    pub schema_version: String,
    pub request: V4CompileTimeCapabilityRequest,
    pub verdict: V4CapabilityReportVerdict,
    pub graph_found: bool,
    pub venue_found: bool,
    pub runtime_mode_found: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complexity_metrics: Option<ComplexityMetrics>,
    #[serde(default)]
    pub type_entries: Vec<V4TypeCapabilityReportEntry>,
    #[serde(default)]
    pub execution_entries: Vec<V4ExecutionCapabilityReportEntry>,
    #[serde(default)]
    pub plugin_entries: Vec<V4PluginCapabilityReportEntry>,
    #[serde(default)]
    pub diagnostics: Vec<V4CapabilityReportDiagnostic>,
    #[serde(default)]
    pub execution_submission_attached: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4CapabilityReportVerdict {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4CapabilityReportDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V4CapabilityReportDiagnostic {
    pub severity: V4CapabilityReportDiagnosticSeverity,
    pub code: String,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4TypeCapabilityStatus {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V4TypeCapabilityReportEntry {
    pub type_ref: QsTypeRef,
    pub status: V4TypeCapabilityStatus,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4ExecutionCapabilityStatus {
    Supported,
    Unsupported,
    ModeRejected,
    NotDeclared,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V4ExecutionCapabilityReportEntry {
    pub capability: ExecutionCapabilityKind,
    pub required: bool,
    pub status: V4ExecutionCapabilityStatus,
    pub source: CapabilitySupportSource,
    #[serde(default)]
    pub supported_modes: Vec<RuntimeTradingMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_source: Option<CapabilitySupportSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4PluginCapabilityStatus {
    Accepted,
    Rejected,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V4PluginCapabilityReportEntry {
    pub plugin_id: String,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<PluginKind>,
    pub status: V4PluginCapabilityStatus,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

impl Default for V4VersionManifest {
    fn default() -> Self {
        Self {
            schema_version: V4_VERSION_MANIFEST_VERSION.to_string(),
            qs_language_version: default_qs_language_version(),
            type_schema_version: V4_QS_TYPE_SYSTEM_VERSION.to_string(),
            machine_template_version: V4_MACHINE_CONTRACT_VERSION.to_string(),
            capability_matrix_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
            additive_types_are_compatible: true,
            additive_defaulted_fields_are_compatible: true,
            type_tightening_requires_migration: true,
            type_deletion_requires_deprecation_first: true,
            semantic_change_requires_schema_bump: true,
        }
    }
}

impl Default for PluginGovernanceContract {
    fn default() -> Self {
        Self {
            schema_version: V4_PLUGIN_GOVERNANCE_VERSION.to_string(),
            allowed_kinds: default_plugin_kinds(),
            required_fields: default_plugin_required_fields(),
            qs_declares_capabilities_only: true,
            real_order_requires_venue_plugin_and_risk_plane: true,
            pure_plugins_must_be_deterministic: true,
        }
    }
}

impl Default for ReproducibilityContract {
    fn default() -> Self {
        Self {
            schema_version: V4_REPRODUCIBILITY_CONTRACT_VERSION.to_string(),
            required_evidence: default_reproducibility_evidence(),
            required_event_envelope_fields: default_event_envelope_fields(),
            key_decision_path_replay_required: true,
            full_tick_replay_required: false,
        }
    }
}

impl Default for ComplexityBudgetContract {
    fn default() -> Self {
        Self {
            schema_version: V4_COMPLEXITY_BUDGET_CONTRACT_VERSION.to_string(),
            max_state_count: 512,
            max_transition_count: 1_024,
            max_memory_field_count: 512,
            max_plugin_call_count: 256,
            max_mode_count: 4,
            max_stale_dependency_count: 128,
            max_estimated_order_paths: 512,
            max_event_rate_estimate: 100_000,
        }
    }
}

impl Default for DeveloperLearningPipelineContract {
    fn default() -> Self {
        Self {
            schema_version: V4_LEARNING_PIPELINE_CONTRACT_VERSION.to_string(),
            core_pipeline_in_repo: true,
            local_learning_dir: default_learning_dir(),
            local_learning_dir_gitignored: true,
            write_requires_explicit_user_command: true,
            included_in_regular_gates: false,
            major_closeout_question_required: true,
            owner_first_iteration_only: true,
        }
    }
}

impl Default for V4StaticContractBundle {
    fn default() -> Self {
        Self {
            schema_version: V4_STATIC_CONTRACT_BUNDLE_VERSION.to_string(),
            version_manifest: V4VersionManifest::default(),
            qs_profile: QsStateMachineProfile::default(),
            type_system: QsTypeSystemContract::default(),
            runtime_modes: RuntimeModeContract::default(),
            plugin_governance: PluginGovernanceContract::default(),
            reproducibility: ReproducibilityContract::default(),
            complexity_budget: ComplexityBudgetContract::default(),
            learning_pipeline: DeveloperLearningPipelineContract::default(),
            machine_graphs: Vec::new(),
            venue_matrices: Vec::new(),
            plugin_manifests: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }
}

impl V4VersionManifest {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_VERSION_MANIFEST_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_VERSION_MANIFEST_VERSION
            ));
        }
        if self.qs_language_version.trim().is_empty() {
            errors.push("qs_language_version is required".to_string());
        }
        if self.type_schema_version != V4_QS_TYPE_SYSTEM_VERSION {
            errors.push(format!(
                "type_schema_version must be `{}`",
                V4_QS_TYPE_SYSTEM_VERSION
            ));
        }
        if self.machine_template_version != V4_MACHINE_CONTRACT_VERSION {
            errors.push(format!(
                "machine_template_version must be `{}`",
                V4_MACHINE_CONTRACT_VERSION
            ));
        }
        if self.capability_matrix_version != V4_VENUE_CAPABILITY_MATRIX_VERSION {
            errors.push(format!(
                "capability_matrix_version must be `{}`",
                V4_VENUE_CAPABILITY_MATRIX_VERSION
            ));
        }
        if !self.additive_types_are_compatible {
            errors.push("additive types must stay compatible".to_string());
        }
        if !self.additive_defaulted_fields_are_compatible {
            errors.push("additive defaulted fields must stay compatible".to_string());
        }
        if !self.type_tightening_requires_migration {
            errors.push("type tightening must require migration".to_string());
        }
        if !self.type_deletion_requires_deprecation_first {
            errors.push("type deletion must require deprecation first".to_string());
        }
        if !self.semantic_change_requires_schema_bump {
            errors.push("semantic changes must require a schema bump".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl PluginGovernanceContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_PLUGIN_GOVERNANCE_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_PLUGIN_GOVERNANCE_VERSION
            ));
        }

        let allowed = self.allowed_kinds.iter().copied().collect::<BTreeSet<_>>();
        for kind in [PluginKind::Pure, PluginKind::Runtime, PluginKind::Venue] {
            if !allowed.contains(&kind) {
                errors.push(format!("plugin governance must allow `{:?}` plugins", kind));
            }
        }

        let fields = self
            .required_fields
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for field in default_plugin_required_fields() {
            if !fields.contains(&field) {
                errors.push(format!(
                    "plugin governance must require manifest field `{:?}`",
                    field
                ));
            }
        }

        if !self.qs_declares_capabilities_only {
            errors.push("QS must declare capabilities only; plugins implement them".to_string());
        }
        if !self.real_order_requires_venue_plugin_and_risk_plane {
            errors.push(
                "real order paths must require a venue plugin and runtime risk plane".to_string(),
            );
        }
        if !self.pure_plugins_must_be_deterministic {
            errors.push("pure plugins must be deterministic".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn validate_plugin_manifest(
        &self,
        manifest: &PluginManifestSpec,
        type_system: &QsTypeSystemContract,
        runtime_modes: &RuntimeModeContract,
    ) -> Result<(), Vec<String>> {
        let mut errors = self.validate_static_contract().err().unwrap_or_default();

        if manifest.plugin_id.trim().is_empty() {
            errors.push("plugin_id is required".to_string());
        }
        if manifest.name.trim().is_empty() {
            errors.push("plugin name is required".to_string());
        }
        if manifest.version.trim().is_empty() {
            errors.push("plugin version is required".to_string());
        }
        if manifest.test_fixture_id.trim().is_empty() {
            errors.push("plugin test_fixture_id is required".to_string());
        }
        if !self.allowed_kinds.contains(&manifest.kind) {
            errors.push(format!("plugin kind `{:?}` is not allowed", manifest.kind));
        }
        match &manifest.input_schema {
            Some(type_ref) => {
                errors.extend(
                    type_system
                        .validate_type_ref(type_ref)
                        .err()
                        .unwrap_or_default(),
                );
            }
            None => errors.push("plugin input_schema is required".to_string()),
        }
        match &manifest.output_schema {
            Some(type_ref) => {
                errors.extend(
                    type_system
                        .validate_type_ref(type_ref)
                        .err()
                        .unwrap_or_default(),
                );
            }
            None => errors.push("plugin output_schema is required".to_string()),
        }

        match manifest.kind {
            PluginKind::Pure => {
                if self.pure_plugins_must_be_deterministic && !manifest.deterministic {
                    errors.push("pure plugins must be deterministic".to_string());
                }
                if !matches!(manifest.side_effect, PluginSideEffect::None) {
                    errors.push("pure plugins must not declare side effects".to_string());
                }
                if !matches!(manifest.runtime_permission, PluginRuntimePermission::None) {
                    errors.push("pure plugins must not require runtime permission".to_string());
                }
                if !matches!(manifest.network_permission, PluginNetworkPermission::None) {
                    errors.push("pure plugins must not require network permission".to_string());
                }
                if manifest.capability_matrix.is_some() {
                    errors.push(
                        "pure plugins must not declare a venue capability matrix".to_string(),
                    );
                }
            }
            PluginKind::Runtime => {
                if matches!(
                    manifest.network_permission,
                    PluginNetworkPermission::ProviderOnly
                ) {
                    errors.push("runtime plugins must not access provider network".to_string());
                }
                if !matches!(
                    manifest.runtime_permission,
                    PluginRuntimePermission::LocalSimulation
                        | PluginRuntimePermission::RuntimeState
                ) {
                    errors.push(
                        "runtime plugins must declare local simulation or runtime state permission"
                            .to_string(),
                    );
                }
            }
            PluginKind::Venue => {
                if !matches!(manifest.side_effect, PluginSideEffect::ProviderNetwork) {
                    errors.push(
                        "venue plugins must declare provider network side effects".to_string(),
                    );
                }
                if !matches!(
                    manifest.runtime_permission,
                    PluginRuntimePermission::VenueAdapter
                ) {
                    errors.push(
                        "venue plugins must require venue_adapter runtime permission".to_string(),
                    );
                }
                if !matches!(
                    manifest.network_permission,
                    PluginNetworkPermission::ProviderOnly
                ) {
                    errors.push(
                        "venue plugins must use provider_only network permission".to_string(),
                    );
                }
                match &manifest.capability_matrix {
                    Some(matrix) => {
                        errors.extend(
                            matrix
                                .validate_v4_first_wave_contract()
                                .err()
                                .unwrap_or_default(),
                        );
                        for capability in v4_first_wave_execution_capabilities() {
                            for mode in required_runtime_trading_modes() {
                                let _ = matrix.require_supported_for_mode(
                                    capability,
                                    mode,
                                    runtime_modes,
                                );
                            }
                        }
                    }
                    None => errors.push("venue plugins must declare capability_matrix".to_string()),
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ReproducibilityContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_REPRODUCIBILITY_CONTRACT_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_REPRODUCIBILITY_CONTRACT_VERSION
            ));
        }

        let evidence = self
            .required_evidence
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for kind in default_reproducibility_evidence() {
            if !evidence.contains(&kind) {
                errors.push(format!("reproducibility evidence `{:?}` is required", kind));
            }
        }

        let fields = self
            .required_event_envelope_fields
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for field in default_event_envelope_fields() {
            if !fields.contains(&field) {
                errors.push(format!("event envelope field `{:?}` is required", field));
            }
        }

        if !self.key_decision_path_replay_required {
            errors.push("key decision path replay must be required".to_string());
        }
        if self.full_tick_replay_required {
            errors.push("full tick replay is reserved for a later phase".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ComplexityBudgetContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_COMPLEXITY_BUDGET_CONTRACT_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_COMPLEXITY_BUDGET_CONTRACT_VERSION
            ));
        }
        for (name, value) in [
            ("max_state_count", self.max_state_count),
            ("max_transition_count", self.max_transition_count),
            ("max_memory_field_count", self.max_memory_field_count),
            ("max_plugin_call_count", self.max_plugin_call_count),
            ("max_mode_count", self.max_mode_count),
            (
                "max_stale_dependency_count",
                self.max_stale_dependency_count,
            ),
            ("max_estimated_order_paths", self.max_estimated_order_paths),
            ("max_event_rate_estimate", self.max_event_rate_estimate),
        ] {
            if value == 0 {
                errors.push(format!("{name} must be greater than 0"));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn validate_metrics(&self, metrics: &ComplexityMetrics) -> Result<(), Vec<String>> {
        let mut errors = self.validate_static_contract().err().unwrap_or_default();

        for (name, value, limit) in [
            ("state_count", metrics.state_count, self.max_state_count),
            (
                "transition_count",
                metrics.transition_count,
                self.max_transition_count,
            ),
            (
                "memory_field_count",
                metrics.memory_field_count,
                self.max_memory_field_count,
            ),
            (
                "plugin_call_count",
                metrics.plugin_call_count,
                self.max_plugin_call_count,
            ),
            ("mode_count", metrics.mode_count, self.max_mode_count),
            (
                "stale_dependency_count",
                metrics.stale_dependency_count,
                self.max_stale_dependency_count,
            ),
            (
                "estimated_order_paths",
                metrics.estimated_order_paths,
                self.max_estimated_order_paths,
            ),
            (
                "event_rate_estimate",
                metrics.event_rate_estimate,
                self.max_event_rate_estimate,
            ),
        ] {
            if value > limit {
                errors.push(format!("{name} {value} exceeds budget {limit}"));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl DeveloperLearningPipelineContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_LEARNING_PIPELINE_CONTRACT_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_LEARNING_PIPELINE_CONTRACT_VERSION
            ));
        }
        if !self.core_pipeline_in_repo {
            errors.push("core learning pipeline must live in the repository".to_string());
        }
        if self.local_learning_dir != default_learning_dir() {
            errors.push(format!(
                "local_learning_dir must be `{}`",
                default_learning_dir()
            ));
        }
        if !self.local_learning_dir_gitignored {
            errors.push("local learning records must stay gitignored".to_string());
        }
        if !self.write_requires_explicit_user_command {
            errors.push("learning records must require explicit user command".to_string());
        }
        if self.included_in_regular_gates {
            errors.push("learning pipeline must not enter regular mandatory gates".to_string());
        }
        if !self.major_closeout_question_required {
            errors.push("MAJOR closeout must ask the learning pipeline question".to_string());
        }
        if !self.owner_first_iteration_only {
            errors.push("first learning pipeline iteration must stay owner-first".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl V4StaticContractBundle {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_STATIC_CONTRACT_BUNDLE_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_STATIC_CONTRACT_BUNDLE_VERSION
            ));
        }
        errors.extend(
            self.version_manifest
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.qs_profile
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.type_system
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.runtime_modes
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.plugin_governance
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.reproducibility
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.complexity_budget
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );
        errors.extend(
            self.learning_pipeline
                .validate_static_contract()
                .err()
                .unwrap_or_default(),
        );

        if self.machine_graphs.is_empty() {
            errors
                .push("static contract bundle must include at least one machine graph".to_string());
        }
        if self.venue_matrices.is_empty() {
            errors
                .push("static contract bundle must include at least one venue matrix".to_string());
        }

        for graph in &self.machine_graphs {
            errors.extend(graph.validate_static_contract().err().unwrap_or_default());
            let metrics = ComplexityMetrics::from_machine_graph(
                graph,
                self.runtime_modes.modes.len() as u32,
                self.plugin_manifests.len() as u32,
            );
            errors.extend(
                self.complexity_budget
                    .validate_metrics(&metrics)
                    .err()
                    .unwrap_or_default(),
            );
        }

        for matrix in &self.venue_matrices {
            errors.extend(
                matrix
                    .validate_v4_first_wave_contract()
                    .err()
                    .unwrap_or_default(),
            );
        }

        for manifest in &self.plugin_manifests {
            errors.extend(
                self.plugin_governance
                    .validate_plugin_manifest(manifest, &self.type_system, &self.runtime_modes)
                    .err()
                    .unwrap_or_default(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn build_compile_time_capability_report(
        &self,
        request: V4CompileTimeCapabilityRequest,
    ) -> V4CompileTimeCapabilityReport {
        let mut diagnostics = Vec::new();

        if request.schema_version != V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION {
            push_capability_diagnostic(
                &mut diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP000",
                "request.schema_version",
                format!(
                    "compile-time capability request schema_version must be `{}`",
                    V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION
                ),
            );
        }
        if request.graph_id.trim().is_empty() {
            push_capability_diagnostic(
                &mut diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP001",
                "request.graph_id",
                "compile-time capability request requires graph_id",
            );
        }
        if request.venue_id.trim().is_empty() {
            push_capability_diagnostic(
                &mut diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP002",
                "request.venue_id",
                "compile-time capability request requires venue_id",
            );
        }
        if let Err(errors) = self.validate_static_contract() {
            for error in errors {
                push_capability_diagnostic(
                    &mut diagnostics,
                    V4CapabilityReportDiagnosticSeverity::Error,
                    "V4CAP003",
                    "static_contract_bundle",
                    error,
                );
            }
        }

        let graph = self
            .machine_graphs
            .iter()
            .find(|graph| graph.graph_id == request.graph_id);
        let graph_found = graph.is_some();
        if !graph_found && !request.graph_id.trim().is_empty() {
            push_capability_diagnostic(
                &mut diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP004",
                "request.graph_id",
                format!("machine graph `{}` is not declared", request.graph_id),
            );
        }

        let venue = self
            .venue_matrices
            .iter()
            .find(|matrix| matrix.venue_id == request.venue_id);
        let venue_found = venue.is_some();
        if !venue_found && !request.venue_id.trim().is_empty() {
            push_capability_diagnostic(
                &mut diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP005",
                "request.venue_id",
                format!(
                    "venue capability matrix `{}` is not declared",
                    request.venue_id
                ),
            );
        }

        let runtime_mode_found = self.runtime_modes.mode_spec(request.runtime_mode).is_some();
        if !runtime_mode_found {
            push_capability_diagnostic(
                &mut diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP006",
                "request.runtime_mode",
                format!("runtime mode `{:?}` is not declared", request.runtime_mode),
            );
        }

        let complexity_metrics = graph.map(|graph| {
            ComplexityMetrics::from_machine_graph(
                graph,
                self.runtime_modes.modes.len() as u32,
                self.plugin_manifests.len() as u32,
            )
        });
        if let Some(metrics) = &complexity_metrics {
            if let Err(errors) = self.complexity_budget.validate_metrics(metrics) {
                for error in errors {
                    push_capability_diagnostic(
                        &mut diagnostics,
                        V4CapabilityReportDiagnosticSeverity::Error,
                        "V4CAP007",
                        "complexity_metrics",
                        error,
                    );
                }
            }
        }

        let type_entries = self.build_type_capability_entries(&request, &mut diagnostics);
        let execution_entries =
            self.build_execution_capability_entries(&request, venue, &mut diagnostics);
        let plugin_entries = self.build_plugin_capability_entries(&request, &mut diagnostics);

        let verdict = if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == V4CapabilityReportDiagnosticSeverity::Error)
        {
            V4CapabilityReportVerdict::Rejected
        } else {
            V4CapabilityReportVerdict::Accepted
        };

        V4CompileTimeCapabilityReport {
            schema_version: V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION.to_string(),
            request,
            verdict,
            graph_found,
            venue_found,
            runtime_mode_found,
            complexity_metrics,
            type_entries,
            execution_entries,
            plugin_entries,
            diagnostics,
            execution_submission_attached: false,
        }
    }

    fn build_type_capability_entries(
        &self,
        request: &V4CompileTimeCapabilityRequest,
        diagnostics: &mut Vec<V4CapabilityReportDiagnostic>,
    ) -> Vec<V4TypeCapabilityReportEntry> {
        let mut entries = Vec::new();

        for (index, type_ref) in request.required_type_refs.iter().enumerate() {
            match self.type_system.validate_type_ref(type_ref) {
                Ok(()) => entries.push(V4TypeCapabilityReportEntry {
                    type_ref: type_ref.clone(),
                    status: V4TypeCapabilityStatus::Accepted,
                    diagnostics: Vec::new(),
                }),
                Err(errors) => {
                    for error in &errors {
                        push_capability_diagnostic(
                            diagnostics,
                            V4CapabilityReportDiagnosticSeverity::Error,
                            "V4CAP100",
                            format!("request.required_type_refs[{index}]"),
                            error.clone(),
                        );
                    }
                    entries.push(V4TypeCapabilityReportEntry {
                        type_ref: type_ref.clone(),
                        status: V4TypeCapabilityStatus::Rejected,
                        diagnostics: errors,
                    });
                }
            }
        }

        entries
    }

    fn build_execution_capability_entries(
        &self,
        request: &V4CompileTimeCapabilityRequest,
        venue: Option<&VenueCapabilityMatrix>,
        diagnostics: &mut Vec<V4CapabilityReportDiagnostic>,
    ) -> Vec<V4ExecutionCapabilityReportEntry> {
        let required = request
            .required_execution_capabilities
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut capabilities = v4_first_wave_execution_capabilities()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        capabilities.extend(required.iter().copied());

        capabilities
            .into_iter()
            .map(|capability| {
                let is_required = required.contains(&capability);
                let Some(matrix) = venue else {
                    if is_required {
                        push_capability_diagnostic(
                            diagnostics,
                            V4CapabilityReportDiagnosticSeverity::Error,
                            "V4CAP200",
                            format!("execution_capability.{:?}", capability),
                            format!(
                                "required execution capability `{:?}` has no venue matrix",
                                capability
                            ),
                        );
                    }
                    return V4ExecutionCapabilityReportEntry {
                        capability,
                        required: is_required,
                        status: V4ExecutionCapabilityStatus::NotDeclared,
                        source: CapabilitySupportSource::Unsupported,
                        supported_modes: Vec::new(),
                        selected_source: None,
                        rejection_reason: Some("venue capability matrix is missing".to_string()),
                    };
                };

                let Some(entry) = matrix.capability_entry(&capability) else {
                    if is_required {
                        push_capability_diagnostic(
                            diagnostics,
                            V4CapabilityReportDiagnosticSeverity::Error,
                            "V4CAP201",
                            format!("execution_capability.{:?}", capability),
                            format!(
                                "required execution capability `{:?}` must be explicitly declared",
                                capability
                            ),
                        );
                    }
                    return V4ExecutionCapabilityReportEntry {
                        capability,
                        required: is_required,
                        status: V4ExecutionCapabilityStatus::NotDeclared,
                        source: CapabilitySupportSource::Unsupported,
                        supported_modes: Vec::new(),
                        selected_source: None,
                        rejection_reason: Some(
                            "capability is not declared in the venue matrix".to_string(),
                        ),
                    };
                };

                match matrix.require_supported_for_mode(
                    &capability,
                    request.runtime_mode,
                    &self.runtime_modes,
                ) {
                    Ok(source) => V4ExecutionCapabilityReportEntry {
                        capability,
                        required: is_required,
                        status: V4ExecutionCapabilityStatus::Supported,
                        source: entry.source,
                        supported_modes: entry.supported_modes.clone(),
                        selected_source: Some(source),
                        rejection_reason: None,
                    },
                    Err(reason) => {
                        let status = if matches!(entry.source, CapabilitySupportSource::Unsupported)
                        {
                            V4ExecutionCapabilityStatus::Unsupported
                        } else {
                            V4ExecutionCapabilityStatus::ModeRejected
                        };
                        if is_required {
                            push_capability_diagnostic(
                                diagnostics,
                                V4CapabilityReportDiagnosticSeverity::Error,
                                "V4CAP202",
                                format!("execution_capability.{:?}", capability),
                                reason.clone(),
                            );
                        }
                        V4ExecutionCapabilityReportEntry {
                            capability,
                            required: is_required,
                            status,
                            source: entry.source,
                            supported_modes: entry.supported_modes.clone(),
                            selected_source: None,
                            rejection_reason: Some(reason),
                        }
                    }
                }
            })
            .collect()
    }

    fn build_plugin_capability_entries(
        &self,
        request: &V4CompileTimeCapabilityRequest,
        diagnostics: &mut Vec<V4CapabilityReportDiagnostic>,
    ) -> Vec<V4PluginCapabilityReportEntry> {
        let required = request
            .required_plugin_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let mut reported = BTreeSet::new();
        let mut entries = Vec::new();

        for manifest in &self.plugin_manifests {
            let is_required = required.contains(manifest.plugin_id.as_str());
            reported.insert(manifest.plugin_id.as_str());
            match self.plugin_governance.validate_plugin_manifest(
                manifest,
                &self.type_system,
                &self.runtime_modes,
            ) {
                Ok(()) => entries.push(V4PluginCapabilityReportEntry {
                    plugin_id: manifest.plugin_id.clone(),
                    required: is_required,
                    kind: Some(manifest.kind),
                    status: V4PluginCapabilityStatus::Accepted,
                    diagnostics: Vec::new(),
                }),
                Err(errors) => {
                    for error in &errors {
                        push_capability_diagnostic(
                            diagnostics,
                            V4CapabilityReportDiagnosticSeverity::Error,
                            "V4CAP300",
                            format!("plugin_manifest.{}", manifest.plugin_id),
                            error.clone(),
                        );
                    }
                    entries.push(V4PluginCapabilityReportEntry {
                        plugin_id: manifest.plugin_id.clone(),
                        required: is_required,
                        kind: Some(manifest.kind),
                        status: V4PluginCapabilityStatus::Rejected,
                        diagnostics: errors,
                    });
                }
            }
        }

        for plugin_id in &request.required_plugin_ids {
            if reported.contains(plugin_id.as_str()) {
                continue;
            }
            push_capability_diagnostic(
                diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP301",
                format!("request.required_plugin_ids.{plugin_id}"),
                format!("required plugin `{plugin_id}` is not declared"),
            );
            entries.push(V4PluginCapabilityReportEntry {
                plugin_id: plugin_id.clone(),
                required: true,
                kind: None,
                status: V4PluginCapabilityStatus::Missing,
                diagnostics: vec![format!("required plugin `{plugin_id}` is not declared")],
            });
        }

        entries
    }
}

impl V4CompileTimeCapabilityReport {
    pub fn validate_for_compile(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION {
            errors.push(format!(
                "compile-time capability report schema_version must be `{}`",
                V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION
            ));
        }
        if self.verdict != V4CapabilityReportVerdict::Accepted {
            errors.push("compile-time capability report verdict must be accepted".to_string());
        }
        if !self.graph_found {
            errors.push("compile-time capability report must resolve graph_id".to_string());
        }
        if !self.venue_found {
            errors.push("compile-time capability report must resolve venue_id".to_string());
        }
        if !self.runtime_mode_found {
            errors.push("compile-time capability report must resolve runtime_mode".to_string());
        }
        if self.execution_submission_attached {
            errors.push(
                "compile-time capability report must not attach execution submission".to_string(),
            );
        }
        for diagnostic in &self.diagnostics {
            if diagnostic.severity == V4CapabilityReportDiagnosticSeverity::Error {
                errors.push(format!(
                    "{} {}: {}",
                    diagnostic.code, diagnostic.target, diagnostic.message
                ));
            }
        }

        let required_capabilities = self
            .request
            .required_execution_capabilities
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for capability in required_capabilities {
            let supported = self.execution_entries.iter().any(|entry| {
                entry.capability == capability
                    && entry.required
                    && entry.status == V4ExecutionCapabilityStatus::Supported
            });
            if !supported {
                errors.push(format!(
                    "required execution capability `{:?}` is not supported by the report",
                    capability
                ));
            }
        }

        for plugin_id in &self.request.required_plugin_ids {
            let accepted = self.plugin_entries.iter().any(|entry| {
                entry.plugin_id == *plugin_id
                    && entry.required
                    && entry.status == V4PluginCapabilityStatus::Accepted
            });
            if !accepted {
                errors.push(format!(
                    "required plugin `{plugin_id}` is not accepted by the report"
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

impl ComplexityMetrics {
    pub fn from_machine_graph(
        graph: &V4MachineGraphContract,
        mode_count: u32,
        plugin_call_count: u32,
    ) -> Self {
        let state_count = graph
            .machines
            .iter()
            .map(|machine| machine.states.len() as u32)
            .sum();
        let transition_count = graph
            .machines
            .iter()
            .map(|machine| machine.transitions.len() as u32)
            .sum();
        let memory_field_count = graph
            .machines
            .iter()
            .map(|machine| machine.memory.len() as u32)
            .sum();
        let event_rate_estimate = graph
            .event_catalog
            .as_ref()
            .map(|catalog| catalog.events.len() as u32)
            .unwrap_or_default()
            .saturating_mul(1_000);
        let estimated_order_paths = graph
            .machines
            .iter()
            .filter(|machine| matches!(machine.template, MachineTemplateKind::Execution))
            .count() as u32;

        Self {
            state_count,
            transition_count,
            memory_field_count,
            plugin_call_count,
            mode_count,
            stale_dependency_count: 0,
            estimated_order_paths,
            event_rate_estimate,
        }
    }
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

    pub fn capability_entry(
        &self,
        capability: &ExecutionCapabilityKind,
    ) -> Option<&VenueCapability> {
        self.capabilities
            .iter()
            .find(|entry| &entry.capability == capability)
    }

    pub fn support_source(&self, capability: &ExecutionCapabilityKind) -> CapabilitySupportSource {
        self.capability_entry(capability)
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

    pub fn require_supported_for_mode(
        &self,
        capability: &ExecutionCapabilityKind,
        mode: RuntimeTradingMode,
        runtime_mode_contract: &RuntimeModeContract,
    ) -> Result<CapabilitySupportSource, String> {
        let mode_spec = runtime_mode_contract
            .mode_spec(mode)
            .ok_or_else(|| format!("runtime mode `{:?}` is not declared", mode))?;
        let entry = self.capability_entry(capability).ok_or_else(|| {
            format!(
                "execution capability `{:?}` is unsupported for venue `{}`",
                capability, self.venue_id
            )
        })?;

        if matches!(entry.source, CapabilitySupportSource::Unsupported) {
            return Err(format!(
                "execution capability `{:?}` is unsupported for venue `{}`",
                capability, self.venue_id
            ));
        }
        if !entry.supported_modes.contains(&mode) {
            return Err(format!(
                "execution capability `{:?}` is not supported in runtime mode `{:?}` for venue `{}`",
                capability, mode, self.venue_id
            ));
        }

        match mode_spec.settlement_authority {
            RuntimeSettlementAuthority::ProviderActual => {
                if !matches!(entry.source, CapabilitySupportSource::ProviderNative) {
                    return Err(format!(
                        "provider_actual mode `{:?}` requires provider_native support for `{:?}`",
                        mode, capability
                    ));
                }
            }
            RuntimeSettlementAuthority::LocalSimulated => {
                if !matches!(entry.source, CapabilitySupportSource::RuntimeSimulated) {
                    return Err(format!(
                        "local_simulated mode `{:?}` requires runtime_simulated support for `{:?}`",
                        mode, capability
                    ));
                }
            }
        }

        Ok(entry.source)
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

fn validate_runtime_mode_spec(spec: &RuntimeTradingModeSpec) -> Vec<String> {
    let mut errors = Vec::new();
    let expected = expected_runtime_mode_spec(spec.mode);

    if spec.account_domain != expected.account_domain {
        errors.push(format!(
            "`{:?}` account_domain must be `{:?}`",
            spec.mode, expected.account_domain
        ));
    }
    if spec.settlement_authority != expected.settlement_authority {
        errors.push(format!(
            "`{:?}` settlement_authority must be `{:?}`",
            spec.mode, expected.settlement_authority
        ));
    }
    if spec.execution_event_source != expected.execution_event_source {
        errors.push(format!(
            "`{:?}` execution_event_source must be `{:?}`",
            spec.mode, expected.execution_event_source
        ));
    }
    if spec.provider_order_submission_allowed != expected.provider_order_submission_allowed {
        errors.push(format!(
            "`{:?}` provider_order_submission_allowed must be {}",
            spec.mode, expected.provider_order_submission_allowed
        ));
    }
    if spec.provider_fill_required != expected.provider_fill_required {
        errors.push(format!(
            "`{:?}` provider_fill_required must be {}",
            spec.mode, expected.provider_fill_required
        ));
    }
    if spec.local_fill_engine_required != expected.local_fill_engine_required {
        errors.push(format!(
            "`{:?}` local_fill_engine_required must be {}",
            spec.mode, expected.local_fill_engine_required
        ));
    }
    if spec.local_ledger_required != expected.local_ledger_required {
        errors.push(format!(
            "`{:?}` local_ledger_required must be {}",
            spec.mode, expected.local_ledger_required
        ));
    }
    if spec.provider_account_context_required != expected.provider_account_context_required {
        errors.push(format!(
            "`{:?}` provider_account_context_required must be {}",
            spec.mode, expected.provider_account_context_required
        ));
    }
    if !spec.risk_plane_required {
        errors.push(format!("`{:?}` must require runtime risk plane", spec.mode));
    }

    let declared_events = spec
        .required_events
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    for event in v4_runtime_execution_events() {
        if !declared_events.contains(event) {
            errors.push(format!(
                "`{:?}` must declare runtime execution event `{:?}`",
                spec.mode, event
            ));
        }
    }

    errors
}

fn required_runtime_trading_modes() -> [RuntimeTradingMode; 4] {
    [
        RuntimeTradingMode::PaperActual,
        RuntimeTradingMode::PaperSimulated,
        RuntimeTradingMode::LiveActual,
        RuntimeTradingMode::LiveSimulated,
    ]
}

fn default_runtime_mode_specs() -> Vec<RuntimeTradingModeSpec> {
    required_runtime_trading_modes()
        .into_iter()
        .map(expected_runtime_mode_spec)
        .collect()
}

fn expected_runtime_mode_spec(mode: RuntimeTradingMode) -> RuntimeTradingModeSpec {
    match mode {
        RuntimeTradingMode::PaperActual => RuntimeTradingModeSpec {
            mode,
            account_domain: RuntimeAccountDomain::Paper,
            settlement_authority: RuntimeSettlementAuthority::ProviderActual,
            execution_event_source: RuntimeExecutionEventSource::ProviderActual,
            provider_order_submission_allowed: true,
            provider_fill_required: true,
            local_fill_engine_required: false,
            local_ledger_required: false,
            provider_account_context_required: true,
            risk_plane_required: true,
            required_events: default_runtime_execution_events(),
        },
        RuntimeTradingMode::PaperSimulated => RuntimeTradingModeSpec {
            mode,
            account_domain: RuntimeAccountDomain::Paper,
            settlement_authority: RuntimeSettlementAuthority::LocalSimulated,
            execution_event_source: RuntimeExecutionEventSource::LocalSimulated,
            provider_order_submission_allowed: false,
            provider_fill_required: false,
            local_fill_engine_required: true,
            local_ledger_required: true,
            provider_account_context_required: false,
            risk_plane_required: true,
            required_events: default_runtime_execution_events(),
        },
        RuntimeTradingMode::LiveActual => RuntimeTradingModeSpec {
            mode,
            account_domain: RuntimeAccountDomain::Live,
            settlement_authority: RuntimeSettlementAuthority::ProviderActual,
            execution_event_source: RuntimeExecutionEventSource::ProviderActual,
            provider_order_submission_allowed: true,
            provider_fill_required: true,
            local_fill_engine_required: false,
            local_ledger_required: false,
            provider_account_context_required: true,
            risk_plane_required: true,
            required_events: default_runtime_execution_events(),
        },
        RuntimeTradingMode::LiveSimulated => RuntimeTradingModeSpec {
            mode,
            account_domain: RuntimeAccountDomain::Live,
            settlement_authority: RuntimeSettlementAuthority::LocalSimulated,
            execution_event_source: RuntimeExecutionEventSource::LocalSimulated,
            provider_order_submission_allowed: false,
            provider_fill_required: false,
            local_fill_engine_required: true,
            local_ledger_required: true,
            provider_account_context_required: true,
            risk_plane_required: true,
            required_events: default_runtime_execution_events(),
        },
    }
}

fn default_machine_contract_version() -> String {
    V4_MACHINE_CONTRACT_VERSION.to_string()
}

fn default_static_contract_bundle_version() -> String {
    V4_STATIC_CONTRACT_BUNDLE_VERSION.to_string()
}

fn default_version_manifest_version() -> String {
    V4_VERSION_MANIFEST_VERSION.to_string()
}

fn default_qs_language_version() -> String {
    "quantpilot/qs-language/v4".to_string()
}

fn default_machine_graph_contract_version() -> String {
    V4_MACHINE_GRAPH_CONTRACT_VERSION.to_string()
}

fn default_machine_event_catalog_version() -> String {
    V4_MACHINE_EVENT_CATALOG_VERSION.to_string()
}

fn default_runtime_mode_contract_version() -> String {
    V4_RUNTIME_MODE_CONTRACT_VERSION.to_string()
}

fn default_qs_type_system_version() -> String {
    V4_QS_TYPE_SYSTEM_VERSION.to_string()
}

fn default_plugin_governance_version() -> String {
    V4_PLUGIN_GOVERNANCE_VERSION.to_string()
}

fn default_reproducibility_contract_version() -> String {
    V4_REPRODUCIBILITY_CONTRACT_VERSION.to_string()
}

fn default_complexity_budget_contract_version() -> String {
    V4_COMPLEXITY_BUDGET_CONTRACT_VERSION.to_string()
}

fn default_learning_pipeline_contract_version() -> String {
    V4_LEARNING_PIPELINE_CONTRACT_VERSION.to_string()
}

fn default_compile_time_capability_request_version() -> String {
    V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION.to_string()
}

fn default_compile_time_capability_report_version() -> String {
    V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION.to_string()
}

fn default_venue_capability_matrix_version() -> String {
    V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string()
}

fn default_qs_state_machine_profile_version() -> String {
    V4_QS_STATE_MACHINE_PROFILE_VERSION.to_string()
}

fn default_qs_machine_templates() -> Vec<MachineTemplateKind> {
    vec![
        MachineTemplateKind::Observation,
        MachineTemplateKind::Decision,
        MachineTemplateKind::Execution,
    ]
}

fn default_qs_scalar_types() -> Vec<QsScalarTypeKind> {
    v4_first_wave_scalar_types().to_vec()
}

fn default_qs_composite_type_specs() -> Vec<QsCompositeTypeSpec> {
    vec![
        QsCompositeTypeSpec {
            kind: QsCompositeTypeKind::Optional,
            max_items_required: false,
            max_items_upper_bound: None,
            replay_safe: true,
        },
        QsCompositeTypeSpec {
            kind: QsCompositeTypeKind::List,
            max_items_required: true,
            max_items_upper_bound: Some(10_000),
            replay_safe: true,
        },
        QsCompositeTypeSpec {
            kind: QsCompositeTypeKind::Map,
            max_items_required: true,
            max_items_upper_bound: Some(10_000),
            replay_safe: true,
        },
        QsCompositeTypeSpec {
            kind: QsCompositeTypeKind::Fresh,
            max_items_required: false,
            max_items_upper_bound: None,
            replay_safe: true,
        },
        QsCompositeTypeSpec {
            kind: QsCompositeTypeKind::Stale,
            max_items_required: false,
            max_items_upper_bound: None,
            replay_safe: true,
        },
    ]
}

fn default_qs_type_max_nesting_depth() -> u8 {
    V4_QS_TYPE_MAX_NESTING_DEPTH
}

fn default_plugin_kinds() -> Vec<PluginKind> {
    vec![PluginKind::Pure, PluginKind::Runtime, PluginKind::Venue]
}

fn default_plugin_required_fields() -> Vec<PluginManifestField> {
    vec![
        PluginManifestField::Name,
        PluginManifestField::Version,
        PluginManifestField::InputSchema,
        PluginManifestField::OutputSchema,
        PluginManifestField::Deterministic,
        PluginManifestField::SideEffect,
        PluginManifestField::RuntimePermission,
        PluginManifestField::NetworkPermission,
        PluginManifestField::CapabilityMatrix,
        PluginManifestField::TestFixture,
    ]
}

fn default_reproducibility_evidence() -> Vec<RunEvidenceKind> {
    vec![
        RunEvidenceKind::StrategyRunId,
        RunEvidenceKind::EventSequence,
        RunEvidenceKind::InputSnapshotId,
        RunEvidenceKind::MemoryChangeLog,
        RunEvidenceKind::CapabilityHash,
        RunEvidenceKind::DeploymentRevision,
        RunEvidenceKind::OrderCapabilitySource,
        RunEvidenceKind::RiskDecisionEvidence,
    ]
}

fn default_event_envelope_fields() -> Vec<EventEnvelopeField> {
    vec![
        EventEnvelopeField::EventId,
        EventEnvelopeField::EventType,
        EventEnvelopeField::EventTime,
        EventEnvelopeField::Source,
        EventEnvelopeField::Payload,
        EventEnvelopeField::Freshness,
        EventEnvelopeField::Sequence,
        EventEnvelopeField::Replayable,
    ]
}

fn default_learning_dir() -> String {
    "markdown/learning/".to_string()
}

fn default_transition_conflict_policy() -> TransitionConflictPolicy {
    TransitionConflictPolicy::Error
}

fn default_machine_graph_edge_activation() -> MachineGraphEdgeActivation {
    MachineGraphEdgeActivation::Always
}

fn default_risk_plane_min_priority() -> i32 {
    V4_RISK_PLANE_MIN_PRIORITY
}

fn default_runtime_execution_events() -> Vec<RuntimeExecutionEventKind> {
    v4_runtime_execution_events().to_vec()
}

fn default_true() -> bool {
    true
}

fn push_capability_diagnostic(
    diagnostics: &mut Vec<V4CapabilityReportDiagnostic>,
    severity: V4CapabilityReportDiagnosticSeverity,
    code: impl Into<String>,
    target: impl Into<String>,
    message: impl Into<String>,
) {
    diagnostics.push(V4CapabilityReportDiagnostic {
        severity,
        code: code.into(),
        target: target.into(),
        message: message.into(),
    });
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

    fn sample_machine_with(
        machine_id: &str,
        template: MachineTemplateKind,
        priority: i32,
    ) -> V4MachineContract {
        let mut machine = sample_machine();
        machine.machine_id = machine_id.to_string();
        machine.template = template;
        machine.priority = priority;
        machine.transitions[0].transition_id = format!("{machine_id}.transition");
        machine
    }

    fn sample_graph_edge(
        source_machine_id: &str,
        target_machine_id: &str,
        event_type: &str,
    ) -> MachineGraphEdge {
        MachineGraphEdge {
            edge_id: format!("{source_machine_id}->{target_machine_id}"),
            source_machine_id: source_machine_id.to_string(),
            target_machine_id: target_machine_id.to_string(),
            event_type: event_type.to_string(),
            activation: MachineGraphEdgeActivation::Always,
            required: true,
            metadata: BTreeMap::new(),
        }
    }

    fn sample_event_spec(
        event_type: &str,
        source_kind: MachineEventSourceKind,
        scope: MachineEventScope,
        allowed_emitters: &[&str],
        allowed_consumers: &[&str],
    ) -> MachineEventTypeSpec {
        MachineEventTypeSpec {
            event_type: event_type.to_string(),
            source_kind,
            scope,
            payload_fields: vec![MachineEventPayloadField {
                name: "symbol".to_string(),
                type_name: "string".to_string(),
                required: true,
                nullable: false,
            }],
            allowed_emitters: allowed_emitters
                .iter()
                .map(|emitter| emitter.to_string())
                .collect(),
            allowed_consumers: allowed_consumers
                .iter()
                .map(|consumer| consumer.to_string())
                .collect(),
            replayable: true,
        }
    }

    fn sample_event_catalog() -> MachineEventCatalog {
        MachineEventCatalog {
            schema_version: V4_MACHINE_EVENT_CATALOG_VERSION.to_string(),
            events: vec![
                sample_event_spec(
                    "market.tick",
                    MachineEventSourceKind::MarketData,
                    MachineEventScope::Runtime,
                    &["market.btc_1m"],
                    &["data.market"],
                ),
                sample_event_spec(
                    "bar_closed",
                    MachineEventSourceKind::Machine,
                    MachineEventScope::Graph,
                    &["data.market"],
                    &["intent.trend"],
                ),
                sample_event_spec(
                    "intent.long",
                    MachineEventSourceKind::Machine,
                    MachineEventScope::Graph,
                    &["intent.trend"],
                    &["risk.guard"],
                ),
                sample_event_spec(
                    "risk.approved",
                    MachineEventSourceKind::RiskPlane,
                    MachineEventScope::Graph,
                    &["risk.guard"],
                    &["execution.router"],
                ),
            ],
            metadata: BTreeMap::new(),
        }
    }

    fn sample_machine_graph() -> V4MachineGraphContract {
        let mut data = sample_machine_with("data.market", MachineTemplateKind::Observation, 8_000);
        data.transitions[0].event.event_type = "market.tick".to_string();
        data.transitions[0].event.source = Some("market.btc_1m".to_string());
        data.transitions[0].action = Some(MachineActionSpec {
            emits: vec!["bar_closed".to_string()],
            memory_writes: vec!["last_signal_at".to_string()],
            diagnostics: vec!["market_bar".to_string()],
        });

        let mut intent = sample_machine_with("intent.trend", MachineTemplateKind::Decision, 5_200);
        intent.transitions[0].event.event_type = "bar_closed".to_string();
        intent.transitions[0].event.source = Some("data.market".to_string());
        intent.transitions[0].action = Some(MachineActionSpec {
            emits: vec!["intent.long".to_string()],
            memory_writes: vec!["last_signal_at".to_string()],
            diagnostics: vec!["trend_score".to_string()],
        });

        let mut risk = sample_machine_with("risk.guard", MachineTemplateKind::Decision, 9_500);
        risk.transitions[0].event.event_type = "intent.long".to_string();
        risk.transitions[0].event.source = Some("intent.trend".to_string());
        risk.transitions[0].action = Some(MachineActionSpec {
            emits: vec!["risk.approved".to_string()],
            memory_writes: vec!["last_signal_at".to_string()],
            diagnostics: vec!["risk_decision".to_string()],
        });

        let mut execution =
            sample_machine_with("execution.router", MachineTemplateKind::Execution, 4_000);
        execution.transitions[0].event.event_type = "risk.approved".to_string();
        execution.transitions[0].event.source = Some("risk.guard".to_string());
        execution.transitions[0].action = Some(MachineActionSpec {
            emits: Vec::new(),
            memory_writes: vec!["last_signal_at".to_string()],
            diagnostics: vec!["route_order".to_string()],
        });

        V4MachineGraphContract {
            schema_version: V4_MACHINE_GRAPH_CONTRACT_VERSION.to_string(),
            graph_id: "strategy.v4.sample".to_string(),
            machines: vec![data, intent, risk, execution],
            edges: vec![
                sample_graph_edge("data.market", "intent.trend", "bar_closed"),
                sample_graph_edge("intent.trend", "risk.guard", "intent.long"),
                sample_graph_edge("risk.guard", "execution.router", "risk.approved"),
            ],
            event_catalog: Some(sample_event_catalog()),
            risk_plane: Some(MachineGraphRiskPlane {
                required: true,
                machine_ids: vec!["risk.guard".to_string()],
                min_priority: V4_RISK_PLANE_MIN_PRIORITY,
            }),
            metadata: BTreeMap::new(),
        }
    }

    fn sample_static_contract_bundle() -> V4StaticContractBundle {
        V4StaticContractBundle {
            machine_graphs: vec![sample_machine_graph()],
            venue_matrices: vec![unsupported_v4_first_wave_matrix("paper-local")],
            ..V4StaticContractBundle::default()
        }
    }

    fn sample_paper_simulated_market_matrix() -> VenueCapabilityMatrix {
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

    fn sample_compile_time_capability_request() -> V4CompileTimeCapabilityRequest {
        V4CompileTimeCapabilityRequest {
            schema_version: V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION.to_string(),
            graph_id: "strategy.v4.sample".to_string(),
            venue_id: "paper-local".to_string(),
            runtime_mode: RuntimeTradingMode::PaperSimulated,
            required_execution_capabilities: vec![ExecutionCapabilityKind::Market],
            required_type_refs: vec![QsTypeRef::Scalar {
                scalar: QsScalarTypeKind::Price,
            }],
            required_plugin_ids: vec!["pure.indicator.zscore".to_string()],
        }
    }

    fn sample_pure_plugin_manifest() -> PluginManifestSpec {
        PluginManifestSpec {
            plugin_id: "pure.indicator.zscore".to_string(),
            name: "ZScore".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::Pure,
            input_schema: Some(QsTypeRef::List {
                item: Box::new(QsTypeRef::Scalar {
                    scalar: QsScalarTypeKind::Price,
                }),
                max_items: 256,
            }),
            output_schema: Some(QsTypeRef::Scalar {
                scalar: QsScalarTypeKind::Decimal,
            }),
            deterministic: true,
            side_effect: PluginSideEffect::None,
            runtime_permission: PluginRuntimePermission::None,
            network_permission: PluginNetworkPermission::None,
            capability_matrix: None,
            test_fixture_id: "fixture.zscore.basic".to_string(),
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
    fn machine_graph_accepts_top_level_dag_with_risk_plane() {
        let graph = sample_machine_graph();

        assert_eq!(graph.validate_static_contract(), Ok(()));
    }

    #[test]
    fn machine_graph_rejects_cycle() {
        let mut graph = sample_machine_graph();
        graph.edges.push(sample_graph_edge(
            "execution.router",
            "intent.trend",
            "risk.approved",
        ));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("machine graph must be acyclic")));
    }

    #[test]
    fn machine_graph_rejects_unknown_edge_target() {
        let mut graph = sample_machine_graph();
        graph.edges[0].target_machine_id = "missing.machine".to_string();

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("unknown target_machine_id")));
    }

    #[test]
    fn machine_graph_requires_risk_plane_for_execution() {
        let mut graph = sample_machine_graph();
        graph.risk_plane = None;

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("dedicated risk_plane")));
    }

    #[test]
    fn machine_graph_rejects_execution_bypass_edge() {
        let mut graph = sample_machine_graph();
        graph.edges.push(sample_graph_edge(
            "intent.trend",
            "execution.router",
            "intent.long",
        ));

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("must originate from risk_plane")));
    }

    #[test]
    fn machine_graph_requires_high_priority_decision_risk_machine() {
        let mut graph = sample_machine_graph();
        let risk = graph
            .machines
            .iter_mut()
            .find(|machine| machine.machine_id == "risk.guard")
            .unwrap();
        risk.priority = 100;

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("below min_priority")));
    }

    #[test]
    fn machine_event_catalog_accepts_strong_events() {
        let catalog = sample_event_catalog();

        assert_eq!(catalog.validate_static_contract(), Ok(()));
    }

    #[test]
    fn machine_event_catalog_rejects_untyped_payload_field() {
        let mut catalog = sample_event_catalog();
        catalog.events[0].payload_fields[0].type_name.clear();

        let errors = catalog.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("must declare a type_name")));
    }

    #[test]
    fn machine_graph_requires_event_catalog_for_events() {
        let mut graph = sample_machine_graph();
        graph.event_catalog = None;

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("must declare event_catalog")));
    }

    #[test]
    fn machine_graph_rejects_unknown_transition_event() {
        let mut graph = sample_machine_graph();
        graph.machines[0].transitions[0].event.event_type = "unknown.event".to_string();

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("must be declared in event_catalog")));
    }

    #[test]
    fn machine_graph_rejects_event_emitter_not_allowed() {
        let mut graph = sample_machine_graph();
        graph
            .event_catalog
            .as_mut()
            .unwrap()
            .events
            .iter_mut()
            .find(|event| event.event_type == "risk.approved")
            .unwrap()
            .allowed_emitters = vec!["other.risk".to_string()];

        let errors = graph.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("not an allowed emitter")));
    }

    #[test]
    fn qs_state_machine_profile_default_is_valid() {
        let profile = default_v4_qs_state_machine_profile();

        assert_eq!(profile.validate_static_contract(), Ok(()));
        assert!(profile.state_policy.allow_state_groups);
        assert!(!profile.state_policy.allow_nested_state_machines);
        assert!(
            profile
                .risk_plane_policy
                .dedicated_high_priority_risk_plane_required
        );
    }

    #[test]
    fn qs_state_machine_profile_requires_all_three_templates() {
        let mut profile = default_v4_qs_state_machine_profile();
        profile
            .allowed_templates
            .retain(|template| !matches!(template, MachineTemplateKind::Execution));

        let errors = profile.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| { message.contains("must allow") && message.contains("Execution") }));
    }

    #[test]
    fn qs_state_machine_profile_rejects_direct_order_submit() {
        let mut profile = default_v4_qs_state_machine_profile();
        profile.action_block_policy.allow_direct_order_submit = true;

        let errors = profile.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("must not submit orders directly")));
    }

    #[test]
    fn qs_state_machine_profile_keeps_nested_state_machines_reserved() {
        let mut profile = default_v4_qs_state_machine_profile();
        profile.state_policy.allow_nested_state_machines = true;

        let errors = profile.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| {
            message.contains("nested state machines") && message.contains("reserved")
        }));
    }

    #[test]
    fn qs_state_machine_profile_requires_high_priority_risk_plane() {
        let mut profile = default_v4_qs_state_machine_profile();
        profile
            .risk_plane_policy
            .dedicated_high_priority_risk_plane_required = false;

        let errors = profile.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("high-priority risk safety plane")));
    }

    #[test]
    fn runtime_mode_contract_default_is_valid() {
        let contract = default_v4_runtime_mode_contract();

        assert_eq!(contract.validate_static_contract(), Ok(()));
        assert_eq!(
            contract.settlement_authority_for(RuntimeTradingMode::LiveSimulated),
            Some(RuntimeSettlementAuthority::LocalSimulated)
        );
    }

    #[test]
    fn runtime_mode_contract_requires_all_four_modes() {
        let mut contract = default_v4_runtime_mode_contract();
        contract
            .modes
            .retain(|spec| spec.mode != RuntimeTradingMode::LiveSimulated);

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("LiveSimulated")));
    }

    #[test]
    fn runtime_mode_contract_rejects_live_simulated_provider_submission() {
        let mut contract = default_v4_runtime_mode_contract();
        let live_simulated = contract
            .modes
            .iter_mut()
            .find(|spec| spec.mode == RuntimeTradingMode::LiveSimulated)
            .unwrap();
        live_simulated.provider_order_submission_allowed = true;

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("provider_order_submission_allowed")));
    }

    #[test]
    fn runtime_mode_contract_requires_execution_events() {
        let mut contract = default_v4_runtime_mode_contract();
        contract.modes[0]
            .required_events
            .retain(|event| *event != RuntimeExecutionEventKind::FeeCharged);

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| message.contains("FeeCharged")));
    }

    #[test]
    fn qs_type_system_contract_default_is_valid() {
        let contract = default_v4_qs_type_system_contract();

        assert_eq!(contract.validate_static_contract(), Ok(()));
        assert_eq!(
            contract.validate_type_ref(&QsTypeRef::Fresh {
                inner: Box::new(QsTypeRef::List {
                    item: Box::new(QsTypeRef::Scalar {
                        scalar: QsScalarTypeKind::Price,
                    }),
                    max_items: 256,
                }),
            }),
            Ok(())
        );
    }

    #[test]
    fn qs_type_system_contract_requires_first_wave_scalar_types() {
        let mut contract = default_v4_qs_type_system_contract();
        contract
            .scalar_types
            .retain(|scalar| *scalar != QsScalarTypeKind::RuntimeMode);

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| message.contains("RuntimeMode")));
    }

    #[test]
    fn qs_type_system_contract_rejects_duplicate_composite_types() {
        let mut contract = default_v4_qs_type_system_contract();
        contract
            .composite_types
            .push(contract.composite_types[0].clone());

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("duplicate composite type")));
    }

    #[test]
    fn qs_type_system_rejects_unbounded_list_ref() {
        let contract = default_v4_qs_type_system_contract();

        let errors = contract
            .validate_type_ref(&QsTypeRef::List {
                item: Box::new(QsTypeRef::Scalar {
                    scalar: QsScalarTypeKind::Symbol,
                }),
                max_items: 0,
            })
            .unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("requires max_items greater than 0")));
    }

    #[test]
    fn qs_type_system_rejects_over_budget_map_ref() {
        let contract = default_v4_qs_type_system_contract();

        let errors = contract
            .validate_type_ref(&QsTypeRef::Map {
                key: QsScalarTypeKind::Symbol,
                value: Box::new(QsTypeRef::Scalar {
                    scalar: QsScalarTypeKind::Decimal,
                }),
                max_items: 10_001,
            })
            .unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("exceeds upper bound")));
    }

    #[test]
    fn qs_type_system_rejects_excessive_nesting() {
        let mut contract = default_v4_qs_type_system_contract();
        contract.max_nesting_depth = 2;

        let errors = contract
            .validate_type_ref(&QsTypeRef::Optional {
                inner: Box::new(QsTypeRef::Fresh {
                    inner: Box::new(QsTypeRef::Scalar {
                        scalar: QsScalarTypeKind::Price,
                    }),
                }),
            })
            .unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("exceeds max_nesting_depth")));
    }

    #[test]
    fn static_contract_bundle_accepts_complete_phase_one_bundle() {
        let bundle = sample_static_contract_bundle();

        assert_eq!(bundle.validate_static_contract(), Ok(()));
    }

    #[test]
    fn version_manifest_requires_schema_bump_for_semantic_change() {
        let manifest = V4VersionManifest {
            semantic_change_requires_schema_bump: false,
            ..V4VersionManifest::default()
        };

        let errors = manifest.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("semantic changes")));
    }

    #[test]
    fn plugin_governance_rejects_pure_plugin_with_network_permission() {
        let governance = PluginGovernanceContract::default();
        let mut manifest = sample_pure_plugin_manifest();
        manifest.network_permission = PluginNetworkPermission::ProviderOnly;

        let errors = governance
            .validate_plugin_manifest(
                &manifest,
                &default_v4_qs_type_system_contract(),
                &default_v4_runtime_mode_contract(),
            )
            .unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("pure plugins must not require network permission")));
    }

    #[test]
    fn reproducibility_contract_requires_risk_decision_evidence() {
        let mut contract = ReproducibilityContract::default();
        contract
            .required_evidence
            .retain(|kind| *kind != RunEvidenceKind::RiskDecisionEvidence);

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors
            .iter()
            .any(|message| message.contains("RiskDecisionEvidence")));
    }

    #[test]
    fn complexity_budget_rejects_over_budget_graph() {
        let budget = ComplexityBudgetContract {
            max_state_count: 1,
            ..ComplexityBudgetContract::default()
        };
        let metrics = ComplexityMetrics::from_machine_graph(&sample_machine_graph(), 4, 0);

        let errors = budget.validate_metrics(&metrics).unwrap_err();
        assert!(errors.iter().any(|message| message.contains("state_count")));
    }

    #[test]
    fn learning_pipeline_contract_keeps_local_records_out_of_git() {
        let contract = DeveloperLearningPipelineContract {
            local_learning_dir_gitignored: false,
            ..DeveloperLearningPipelineContract::default()
        };

        let errors = contract.validate_static_contract().unwrap_err();
        assert!(errors.iter().any(|message| message.contains("gitignored")));
    }

    #[test]
    fn compile_time_capability_report_accepts_supported_phase_two_request() {
        let bundle = V4StaticContractBundle {
            machine_graphs: vec![sample_machine_graph()],
            venue_matrices: vec![sample_paper_simulated_market_matrix()],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };
        let request = sample_compile_time_capability_request();

        let report = bundle.build_compile_time_capability_report(request);

        assert_eq!(report.verdict, V4CapabilityReportVerdict::Accepted);
        assert_eq!(report.validate_for_compile(), Ok(()));
        assert!(!report.execution_submission_attached);
        assert_eq!(
            report
                .execution_entries
                .iter()
                .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
                .unwrap()
                .selected_source,
            Some(CapabilitySupportSource::RuntimeSimulated)
        );
        assert_eq!(
            report.plugin_entries[0].status,
            V4PluginCapabilityStatus::Accepted
        );
    }

    #[test]
    fn compile_time_capability_report_rejects_unsupported_required_capability() {
        let bundle = V4StaticContractBundle {
            machine_graphs: vec![sample_machine_graph()],
            venue_matrices: vec![unsupported_v4_first_wave_matrix("paper-local")],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };
        let request = sample_compile_time_capability_request();

        let report = bundle.build_compile_time_capability_report(request);

        assert_eq!(report.verdict, V4CapabilityReportVerdict::Rejected);
        assert!(report.validate_for_compile().is_err());
        assert_eq!(
            report
                .execution_entries
                .iter()
                .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
                .unwrap()
                .status,
            V4ExecutionCapabilityStatus::Unsupported
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "V4CAP202"));
    }

    #[test]
    fn compile_time_capability_report_rejects_provider_native_for_local_simulated_mode() {
        let mut matrix = unsupported_v4_first_wave_matrix("paper-local");
        let market = matrix
            .capabilities
            .iter_mut()
            .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
            .unwrap();
        market.source = CapabilitySupportSource::ProviderNative;
        market.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
        let bundle = V4StaticContractBundle {
            machine_graphs: vec![sample_machine_graph()],
            venue_matrices: vec![matrix],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };

        let report =
            bundle.build_compile_time_capability_report(sample_compile_time_capability_request());

        assert_eq!(report.verdict, V4CapabilityReportVerdict::Rejected);
        assert_eq!(
            report
                .execution_entries
                .iter()
                .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
                .unwrap()
                .status,
            V4ExecutionCapabilityStatus::ModeRejected
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("requires runtime_simulated")));
    }

    #[test]
    fn compile_time_capability_report_rejects_invalid_required_type_ref() {
        let bundle = V4StaticContractBundle {
            machine_graphs: vec![sample_machine_graph()],
            venue_matrices: vec![sample_paper_simulated_market_matrix()],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };
        let mut request = sample_compile_time_capability_request();
        request.required_type_refs = vec![QsTypeRef::List {
            item: Box::new(QsTypeRef::Scalar {
                scalar: QsScalarTypeKind::Price,
            }),
            max_items: 0,
        }];

        let report = bundle.build_compile_time_capability_report(request);

        assert_eq!(report.verdict, V4CapabilityReportVerdict::Rejected);
        assert_eq!(
            report.type_entries[0].status,
            V4TypeCapabilityStatus::Rejected
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "V4CAP100"));
    }

    #[test]
    fn compile_time_capability_report_rejects_missing_required_plugin() {
        let bundle = V4StaticContractBundle {
            machine_graphs: vec![sample_machine_graph()],
            venue_matrices: vec![sample_paper_simulated_market_matrix()],
            ..V4StaticContractBundle::default()
        };

        let report =
            bundle.build_compile_time_capability_report(sample_compile_time_capability_request());

        assert_eq!(report.verdict, V4CapabilityReportVerdict::Rejected);
        assert_eq!(
            report.plugin_entries[0].status,
            V4PluginCapabilityStatus::Missing
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "V4CAP301"));
    }

    #[test]
    fn venue_matrix_requires_provider_native_for_provider_actual_mode() {
        let contract = default_v4_runtime_mode_contract();
        let matrix = VenueCapabilityMatrix {
            schema_version: V4_VENUE_CAPABILITY_MATRIX_VERSION.to_string(),
            venue_id: "paper-local".to_string(),
            capabilities: vec![VenueCapability {
                capability: ExecutionCapabilityKind::Market,
                source: CapabilitySupportSource::RuntimeSimulated,
                supported_modes: vec![RuntimeTradingMode::LiveActual],
                constraints: BTreeMap::new(),
            }],
            metadata: BTreeMap::new(),
        };

        let error = matrix
            .require_supported_for_mode(
                &ExecutionCapabilityKind::Market,
                RuntimeTradingMode::LiveActual,
                &contract,
            )
            .unwrap_err();
        assert!(error.contains("requires provider_native"));
    }

    #[test]
    fn venue_matrix_requires_runtime_simulated_for_local_simulated_mode() {
        let contract = default_v4_runtime_mode_contract();
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
            matrix.require_supported_for_mode(
                &ExecutionCapabilityKind::Market,
                RuntimeTradingMode::PaperSimulated,
                &contract,
            ),
            Ok(CapabilitySupportSource::RuntimeSimulated)
        );
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
