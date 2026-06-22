use std::collections::BTreeSet;

use super::super::super::collect_machine_family;
use crate::v4::{V4MachineContract, V4MachineGraphContract};

pub(super) struct EventReferenceResolution<'a> {
    pub(super) all_machines: Vec<&'a V4MachineContract>,
    pub(super) referenced_events: BTreeSet<&'a str>,
    pub(super) reference_contexts: Vec<EventReferenceContext<'a>>,
    pub(super) errors: Vec<String>,
}

pub(super) struct EventReferenceContext<'a> {
    pub(super) event_type: &'a str,
    pub(super) context: String,
}

impl V4MachineGraphContract {
    pub(super) fn resolve_event_references(&self) -> EventReferenceResolution<'_> {
        let mut errors = Vec::new();
        let mut referenced_events = BTreeSet::new();
        let mut reference_contexts = Vec::new();

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
                    reference_contexts.push(EventReferenceContext {
                        event_type: transition.event.event_type.as_str(),
                        context: format!(
                            "machine `{}` transition `{}`",
                            machine.machine_id, transition.transition_id
                        ),
                    });
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
                            reference_contexts.push(EventReferenceContext {
                                event_type: event_type.as_str(),
                                context: format!(
                                    "machine `{}` transition `{}` action emit",
                                    machine.machine_id, transition.transition_id
                                ),
                            });
                        }
                    }
                }
            }
        }
        for edge in &self.edges {
            if !edge.event_type.trim().is_empty() {
                referenced_events.insert(edge.event_type.as_str());
                reference_contexts.push(EventReferenceContext {
                    event_type: edge.event_type.as_str(),
                    context: format!("edge `{}`", edge.edge_id),
                });
            }
        }

        EventReferenceResolution {
            all_machines,
            referenced_events,
            reference_contexts,
            errors,
        }
    }
}
