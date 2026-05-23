use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const V4_MACHINE_CONTRACT_VERSION: &str = "quantpilot/machine-contract/v1";
pub const V4_VENUE_CAPABILITY_MATRIX_VERSION: &str = "quantpilot/venue-capability-matrix/v1";
pub const V4_QS_STATE_MACHINE_PROFILE_VERSION: &str = "quantpilot/qs-state-machine-profile/v1";
pub const V4_MACHINE_GRAPH_CONTRACT_VERSION: &str = "quantpilot/machine-graph-contract/v1";
pub const V4_RISK_PLANE_MIN_PRIORITY: i32 = 9_000;

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
pub struct V4MachineGraphContract {
    #[serde(default = "default_machine_graph_contract_version")]
    pub schema_version: String,
    pub graph_id: String,
    #[serde(default)]
    pub machines: Vec<V4MachineContract>,
    #[serde(default)]
    pub edges: Vec<MachineGraphEdge>,
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

fn default_machine_graph_contract_version() -> String {
    V4_MACHINE_GRAPH_CONTRACT_VERSION.to_string()
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

fn default_transition_conflict_policy() -> TransitionConflictPolicy {
    TransitionConflictPolicy::Error
}

fn default_machine_graph_edge_activation() -> MachineGraphEdgeActivation {
    MachineGraphEdgeActivation::Always
}

fn default_risk_plane_min_priority() -> i32 {
    V4_RISK_PLANE_MIN_PRIORITY
}

fn default_true() -> bool {
    true
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

    fn sample_graph_edge(source_machine_id: &str, target_machine_id: &str) -> MachineGraphEdge {
        MachineGraphEdge {
            edge_id: format!("{source_machine_id}->{target_machine_id}"),
            source_machine_id: source_machine_id.to_string(),
            target_machine_id: target_machine_id.to_string(),
            event_type: "machine_event".to_string(),
            activation: MachineGraphEdgeActivation::Always,
            required: true,
            metadata: BTreeMap::new(),
        }
    }

    fn sample_machine_graph() -> V4MachineGraphContract {
        V4MachineGraphContract {
            schema_version: V4_MACHINE_GRAPH_CONTRACT_VERSION.to_string(),
            graph_id: "strategy.v4.sample".to_string(),
            machines: vec![
                sample_machine_with("data.market", MachineTemplateKind::Observation, 8_000),
                sample_machine_with("intent.trend", MachineTemplateKind::Decision, 5_200),
                sample_machine_with("risk.guard", MachineTemplateKind::Decision, 9_500),
                sample_machine_with("execution.router", MachineTemplateKind::Execution, 4_000),
            ],
            edges: vec![
                sample_graph_edge("data.market", "intent.trend"),
                sample_graph_edge("intent.trend", "risk.guard"),
                sample_graph_edge("risk.guard", "execution.router"),
            ],
            risk_plane: Some(MachineGraphRiskPlane {
                required: true,
                machine_ids: vec!["risk.guard".to_string()],
                min_priority: V4_RISK_PLANE_MIN_PRIORITY,
            }),
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
    fn machine_graph_accepts_top_level_dag_with_risk_plane() {
        let graph = sample_machine_graph();

        assert_eq!(graph.validate_static_contract(), Ok(()));
    }

    #[test]
    fn machine_graph_rejects_cycle() {
        let mut graph = sample_machine_graph();
        graph
            .edges
            .push(sample_graph_edge("execution.router", "intent.trend"));

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
        graph
            .edges
            .push(sample_graph_edge("intent.trend", "execution.router"));

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
