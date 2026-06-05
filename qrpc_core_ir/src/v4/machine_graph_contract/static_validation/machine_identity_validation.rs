use std::collections::BTreeMap;

use super::super::collect_machine_family;
use crate::v4::{V4MachineContract, V4MachineGraphContract};

pub(super) struct MachineIdentityValidation<'a> {
    pub(super) machines_by_id: BTreeMap<&'a str, &'a V4MachineContract>,
    pub(super) all_machines_by_id: BTreeMap<&'a str, &'a V4MachineContract>,
    pub(super) errors: Vec<String>,
}

impl V4MachineGraphContract {
    pub(super) fn validate_machine_identity(&self) -> MachineIdentityValidation<'_> {
        let mut errors = Vec::new();
        let mut machines_by_id = BTreeMap::new();
        let mut all_machines_by_id = BTreeMap::new();

        for machine in &self.machines {
            if machine.machine_id.trim().is_empty() {
                errors.push("machine_id is required".to_string());
            } else if machines_by_id
                .insert(machine.machine_id.as_str(), machine)
                .is_some()
            {
                errors.push(format!("duplicate machine `{}`", machine.machine_id));
            }
            let mut family = Vec::new();
            collect_machine_family(machine, &mut family);
            for family_machine in family {
                if family_machine.machine_id.trim().is_empty() {
                    continue;
                }
                if all_machines_by_id
                    .insert(family_machine.machine_id.as_str(), family_machine)
                    .is_some()
                {
                    errors.push(format!(
                        "duplicate machine `{}` across top-level and nested machines",
                        family_machine.machine_id
                    ));
                }
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

        MachineIdentityValidation {
            machines_by_id,
            all_machines_by_id,
            errors,
        }
    }
}
