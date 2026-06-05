use std::collections::BTreeSet;

use super::super::super::collect_machine_family;
use crate::v4::{V4MachineContract, V4MachineGraphContract};

pub(super) struct EventReferenceResolution<'a> {
    pub(super) all_machines: Vec<&'a V4MachineContract>,
    pub(super) referenced_events: BTreeSet<&'a str>,
    pub(super) errors: Vec<String>,
}

impl V4MachineGraphContract {
    pub(super) fn resolve_event_references(&self) -> EventReferenceResolution<'_> {
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

        EventReferenceResolution {
            all_machines,
            referenced_events,
            errors,
        }
    }
}
