use qrpc_core_ir::v4::{
    MachineEventCatalog, MachineEventPayloadField, MachineEventScope, MachineEventSourceKind,
    MachineEventTypeSpec, V4MachineContract, V4MachineGraphContract,
    V4_MACHINE_EVENT_CATALOG_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn derive_event_catalog(graph: &V4MachineGraphContract) -> MachineEventCatalog {
    let risk_machine_ids = graph
        .risk_plane
        .as_ref()
        .map(|risk_plane| {
            risk_plane
                .machine_ids
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let mut emitters: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut consumers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for machine in graph.machines.iter().flat_map(machine_family) {
        for transition in &machine.transitions {
            consumers
                .entry(transition.event.event_type.clone())
                .or_default()
                .insert(machine.machine_id.clone());
            if let Some(action) = &transition.action {
                for event_type in &action.emits {
                    emitters
                        .entry(event_type.clone())
                        .or_default()
                        .insert(machine.machine_id.clone());
                }
            }
        }
    }
    for edge in &graph.edges {
        emitters
            .entry(edge.event_type.clone())
            .or_default()
            .insert(edge.source_machine_id.clone());
        consumers
            .entry(edge.event_type.clone())
            .or_default()
            .insert(edge.target_machine_id.clone());
    }

    let event_types = emitters
        .keys()
        .chain(consumers.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let events = event_types
        .into_iter()
        .map(|event_type| {
            let allowed_emitters = emitters
                .remove(&event_type)
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();
            let allowed_consumers = consumers
                .remove(&event_type)
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();
            let source_kind = if allowed_emitters
                .iter()
                .any(|machine_id| risk_machine_ids.contains(machine_id.as_str()))
                || event_type.starts_with("risk.")
            {
                MachineEventSourceKind::RiskPlane
            } else if allowed_emitters.is_empty() && event_type.starts_with("market.") {
                MachineEventSourceKind::MarketData
            } else {
                MachineEventSourceKind::Machine
            };
            MachineEventTypeSpec {
                event_type,
                source_kind,
                scope: MachineEventScope::Graph,
                payload_fields: Vec::<MachineEventPayloadField>::new(),
                allowed_emitters,
                allowed_consumers,
                replayable: true,
            }
        })
        .collect();

    MachineEventCatalog {
        schema_version: V4_MACHINE_EVENT_CATALOG_VERSION.to_string(),
        events,
        metadata: BTreeMap::new(),
    }
}

fn machine_family(machine: &V4MachineContract) -> Vec<&V4MachineContract> {
    let mut machines = vec![machine];
    for state in &machine.states {
        if let Some(child_machine) = state.child_machine.as_deref() {
            machines.extend(machine_family(child_machine));
        }
    }
    machines
}
