mod event_party_validation;

use std::collections::{BTreeMap, BTreeSet};

use super::super::collect_machine_family;
use crate::v4::{V4MachineContract, V4MachineGraphContract};

impl V4MachineGraphContract {
    pub(super) fn validate_event_catalog(
        &self,
        machines_by_id: &BTreeMap<&str, &V4MachineContract>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        let mut referenced_events = BTreeSet::new();

        let all_machines = self
            .machines
            .iter()
            .flat_map(|machine| {
                let mut family = Vec::new();
                collect_machine_family(machine, &mut family);
                family
            })
            .collect::<Vec<_>>();

        for machine in &all_machines {
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

        errors.extend(self.validate_event_parties(&all_machines, &event_specs, machines_by_id));

        errors
    }
}
