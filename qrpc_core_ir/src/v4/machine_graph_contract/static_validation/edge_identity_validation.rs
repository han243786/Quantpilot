use std::collections::{BTreeMap, BTreeSet};

use crate::v4::{V4MachineContract, V4MachineGraphContract};

impl V4MachineGraphContract {
    pub(super) fn validate_edge_identity(
        &self,
        machines_by_id: &BTreeMap<&str, &V4MachineContract>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
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

        errors
    }
}
