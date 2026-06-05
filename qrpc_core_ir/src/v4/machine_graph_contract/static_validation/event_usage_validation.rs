mod event_party_validation;
mod event_reference_resolution;

use std::collections::BTreeMap;

use crate::v4::{V4MachineContract, V4MachineGraphContract};

impl V4MachineGraphContract {
    pub(super) fn validate_event_catalog(
        &self,
        machines_by_id: &BTreeMap<&str, &V4MachineContract>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        let event_references = self.resolve_event_references();
        errors.extend(event_references.errors);

        let Some(catalog) = &self.event_catalog else {
            if !event_references.referenced_events.is_empty() {
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

        for event_type in event_references.referenced_events {
            if !event_specs.contains_key(event_type) {
                errors.push(format!(
                    "event_type `{}` must be declared in event_catalog",
                    event_type
                ));
            }
        }

        errors.extend(self.validate_event_parties(
            &event_references.all_machines,
            &event_specs,
            machines_by_id,
        ));

        errors
    }
}
