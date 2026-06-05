use std::collections::{BTreeMap, BTreeSet};

use super::{
    collect_machine_family, MachineTemplateKind, V4MachineContract, V4MachineGraphContract,
};
use crate::v4::{V4_MACHINE_GRAPH_CONTRACT_VERSION, V4_RISK_PLANE_MIN_PRIORITY};

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

        errors.extend(self.validate_graph_acyclic().err().unwrap_or_default());
        errors.extend(self.validate_event_catalog(&all_machines_by_id));
        errors.extend(self.validate_risk_plane(&machines_by_id));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_graph_acyclic(&self) -> Result<(), Vec<String>> {
        let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for edge in &self.edges {
            adjacency
                .entry(edge.source_machine_id.as_str())
                .or_default()
                .push(edge.target_machine_id.as_str());
        }

        let mut visited = BTreeSet::new();
        let mut in_stack = BTreeSet::new();
        let mut cycle_path = Vec::new();

        fn dfs<'a>(
            node: &'a str,
            adjacency: &BTreeMap<&'a str, Vec<&'a str>>,
            visited: &mut BTreeSet<&'a str>,
            in_stack: &mut BTreeSet<&'a str>,
            cycle_path: &mut Vec<String>,
        ) -> bool {
            visited.insert(node);
            in_stack.insert(node);
            if let Some(neighbors) = adjacency.get(node) {
                for &next in neighbors {
                    if !visited.contains(next) {
                        if dfs(next, adjacency, visited, in_stack, cycle_path) {
                            cycle_path.push(node.to_string());
                            return true;
                        }
                    } else if in_stack.contains(next) {
                        cycle_path.push(next.to_string());
                        cycle_path.push(node.to_string());
                        return true;
                    }
                }
            }
            in_stack.remove(node);
            false
        }

        for edge in &self.edges {
            let source = edge.source_machine_id.as_str();
            if !visited.contains(source)
                && dfs(
                    source,
                    &adjacency,
                    &mut visited,
                    &mut in_stack,
                    &mut cycle_path,
                )
            {
                cycle_path.reverse();
                return Err(vec![format!(
                    "machine graph must be acyclic, cycle: {}",
                    cycle_path.join(" -> ")
                )]);
            }
        }

        Ok(())
    }

    fn validate_event_catalog(
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

        for machine in &all_machines {
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

    fn validate_risk_plane(
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

fn machine_event_party_allowed(allowed_parties: &[String], party: &str) -> bool {
    allowed_parties.is_empty() || allowed_parties.iter().any(|allowed| allowed == party)
}
