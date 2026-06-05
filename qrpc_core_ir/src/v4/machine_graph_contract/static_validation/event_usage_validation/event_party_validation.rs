use std::collections::BTreeMap;

use super::super::machine_event_party_allowed;
use crate::v4::{MachineEventTypeSpec, V4MachineContract, V4MachineGraphContract};

impl V4MachineGraphContract {
    pub(super) fn validate_event_parties(
        &self,
        all_machines: &[&V4MachineContract],
        event_specs: &BTreeMap<&str, &MachineEventTypeSpec>,
        machines_by_id: &BTreeMap<&str, &V4MachineContract>,
    ) -> Vec<String> {
        let mut errors = Vec::new();

        for machine in all_machines {
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
}
