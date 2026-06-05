use std::collections::BTreeSet;

use super::{MachineCachePolicy, MachineRecoveryPolicy, MachineSilencePolicy, V4MachineContract};
use crate::v4::V4_MACHINE_CONTRACT_VERSION;

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

        for transition in &self.transitions {
            if let Some(action) = &transition.action {
                for memory_name in &action.memory_writes {
                    if !memory_names.contains(memory_name.as_str()) {
                        errors.push(format!(
                            "transition `{}` writes undeclared memory field `{}`",
                            transition.transition_id, memory_name
                        ));
                    }
                }
            }
        }

        let mut child_machine_ids = BTreeSet::new();
        for state in &self.states {
            let Some(child_machine) = state.child_machine.as_ref() else {
                continue;
            };
            if child_machine.machine_id.trim().is_empty() {
                errors.push(format!(
                    "state `{}` child_machine must declare machine_id",
                    state.state_id
                ));
            } else if child_machine.machine_id == self.machine_id {
                errors.push(format!(
                    "state `{}` child_machine `{}` must not reuse parent machine_id",
                    state.state_id, child_machine.machine_id
                ));
            } else if !child_machine_ids.insert(child_machine.machine_id.as_str()) {
                errors.push(format!(
                    "duplicate child_machine `{}` under machine `{}`",
                    child_machine.machine_id, self.machine_id
                ));
            }
            if child_machine.template != self.template {
                errors.push(format!(
                    "state `{}` child_machine `{}` template must match parent machine template",
                    state.state_id, child_machine.machine_id
                ));
            }
            if child_machine
                .states
                .iter()
                .any(|child_state| child_state.child_machine.is_some())
            {
                errors.push(format!(
                    "state `{}` child_machine `{}` exceeds max nested machine depth 2",
                    state.state_id, child_machine.machine_id
                ));
            }
            if let Err(child_errors) = child_machine.validate_static_contract() {
                for child_error in child_errors {
                    errors.push(format!(
                        "state `{}` child_machine `{}` failed static contract: {}",
                        state.state_id, child_machine.machine_id, child_error
                    ));
                }
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
