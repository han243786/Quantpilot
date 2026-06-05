use std::collections::{BTreeMap, BTreeSet};

use crate::v4::{
    MachineTemplateKind, V4MachineContract, V4MachineGraphContract, V4_RISK_PLANE_MIN_PRIORITY,
};

impl V4MachineGraphContract {
    pub(super) fn validate_risk_plane(
        &self,
        machines_by_id: &BTreeMap<&str, &V4MachineContract>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        let execution_machine_ids = self
            .machines
            .iter()
            .filter(|machine| matches!(machine.template, MachineTemplateKind::Execution))
            .map(|machine| machine.machine_id.as_str())
            .collect::<BTreeSet<_>>();

        let Some(risk_plane) = &self.risk_plane else {
            if !execution_machine_ids.is_empty() {
                errors.push(
                    "execution machine graphs must declare a dedicated risk_plane".to_string(),
                );
            }
            return errors;
        };

        if !execution_machine_ids.is_empty() && !risk_plane.required {
            errors.push("execution machine graphs must require the risk_plane".to_string());
        }
        if risk_plane.required && risk_plane.machine_ids.is_empty() {
            errors.push("required risk_plane must list at least one machine_id".to_string());
        }
        if risk_plane.min_priority < V4_RISK_PLANE_MIN_PRIORITY {
            errors.push(format!(
                "risk_plane min_priority must be at least {}",
                V4_RISK_PLANE_MIN_PRIORITY
            ));
        }

        let risk_machine_ids = risk_plane
            .machine_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();

        for machine_id in &risk_plane.machine_ids {
            match machines_by_id.get(machine_id.as_str()) {
                Some(machine) => {
                    if !matches!(machine.template, MachineTemplateKind::Decision) {
                        errors.push(format!(
                            "risk_plane machine `{}` must use Decision template",
                            machine_id
                        ));
                    }
                    if machine.priority < risk_plane.min_priority {
                        errors.push(format!(
                            "risk_plane machine `{}` priority {} is below min_priority {}",
                            machine_id, machine.priority, risk_plane.min_priority
                        ));
                    }
                }
                None => errors.push(format!(
                    "risk_plane references unknown machine `{}`",
                    machine_id
                )),
            }
        }

        for execution_machine_id in &execution_machine_ids {
            let mut has_risk_inbound_edge = false;
            for edge in self
                .edges
                .iter()
                .filter(|edge| edge.target_machine_id == *execution_machine_id)
            {
                if risk_machine_ids.contains(edge.source_machine_id.as_str()) {
                    has_risk_inbound_edge = true;
                } else {
                    errors.push(format!(
                        "execution machine `{}` inbound edge `{}` must originate from risk_plane",
                        execution_machine_id, edge.edge_id
                    ));
                }
            }
            if !has_risk_inbound_edge {
                errors.push(format!(
                    "execution machine `{}` must have an inbound edge from risk_plane",
                    execution_machine_id
                ));
            }
        }

        errors
    }
}
