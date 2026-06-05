mod event_usage_validation;
mod graph_acyclic_validation;
mod machine_identity_validation;
mod risk_plane_validation;

use std::collections::BTreeSet;

use super::V4MachineGraphContract;
use crate::v4::V4_MACHINE_GRAPH_CONTRACT_VERSION;

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

        let machine_identity = self.validate_machine_identity();
        errors.extend(machine_identity.errors);

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
            if !machine_identity
                .machines_by_id
                .contains_key(edge.source_machine_id.as_str())
            {
                errors.push(format!(
                    "edge `{}` references unknown source_machine_id `{}`",
                    edge.edge_id, edge.source_machine_id
                ));
            }
            if !machine_identity
                .machines_by_id
                .contains_key(edge.target_machine_id.as_str())
            {
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
        errors.extend(self.validate_event_catalog(&machine_identity.all_machines_by_id));
        errors.extend(self.validate_risk_plane(&machine_identity.machines_by_id));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn machine_event_party_allowed(allowed_parties: &[String], party: &str) -> bool {
    allowed_parties.is_empty() || allowed_parties.iter().any(|allowed| allowed == party)
}
