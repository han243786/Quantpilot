use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use super::{default_true, MachineTemplateKind, V4_QS_STATE_MACHINE_PROFILE_VERSION};

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
            allow_nested_state_machines: true,
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
        if !self.state_policy.allow_nested_state_machines {
            errors.push("QS state machine profile must allow v4 nested state machines".to_string());
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
