use anyhow::{anyhow, Result};
use qrpc_core_ir::v4::{MachineGraphEdge, V4MachineContract, V4MachineGraphContract};
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub fn expand_v4_graph_for_symbols(
    graph: &V4MachineGraphContract,
    symbols: &[String],
) -> Result<V4MachineGraphContract> {
    let normalized_symbols = normalize_v4_backtest_symbols(symbols);
    if normalized_symbols.len() <= 1 {
        let mut single = graph.clone();
        single.metadata.insert(
            "symbols".to_string(),
            json!(normalized_symbols
                .first()
                .cloned()
                .into_iter()
                .collect::<Vec<_>>()),
        );
        return Ok(single);
    }

    let machine_ids = collect_v4_machine_family_ids(&graph.machines);
    let mut expanded = graph.clone();
    expanded.graph_id = format!(
        "{}::universe_{}",
        graph.graph_id,
        normalized_symbols
            .iter()
            .map(|symbol| sanitize_v4_symbol_for_id(symbol))
            .collect::<Vec<_>>()
            .join("_")
    );
    expanded
        .metadata
        .insert("symbols".to_string(), json!(normalized_symbols.clone()));
    expanded
        .metadata
        .insert("universe_expanded".to_string(), Value::Bool(true));

    expanded.machines.clear();
    for symbol in &normalized_symbols {
        for machine in &graph.machines {
            let mut cloned = machine.clone();
            let original_machine_id = machine.machine_id.clone();
            prefix_v4_machine_family_for_symbol(&mut cloned, symbol, &machine_ids);
            cloned
                .metadata
                .insert("symbol".to_string(), Value::String(symbol.clone()));
            cloned.metadata.insert(
                "base_machine_id".to_string(),
                Value::String(original_machine_id.clone()),
            );
            expanded.machines.push(cloned);
        }
    }

    expanded.edges.clear();
    for symbol in &normalized_symbols {
        for edge in &graph.edges {
            expanded.edges.push(MachineGraphEdge {
                edge_id: expanded_v4_machine_id(symbol, &edge.edge_id),
                source_machine_id: expanded_v4_machine_id(symbol, &edge.source_machine_id),
                target_machine_id: expanded_v4_machine_id(symbol, &edge.target_machine_id),
                event_type: edge.event_type.clone(),
                activation: edge.activation.clone(),
                required: edge.required,
                metadata: {
                    let mut metadata = edge.metadata.clone();
                    metadata.insert("symbol".to_string(), Value::String(symbol.clone()));
                    metadata
                },
            });
        }
    }

    if let Some(catalog) = expanded.event_catalog.as_mut() {
        for event in &mut catalog.events {
            event.allowed_emitters = expand_v4_event_party_list(
                &event.allowed_emitters,
                &machine_ids,
                &normalized_symbols,
            );
            event.allowed_consumers = expand_v4_event_party_list(
                &event.allowed_consumers,
                &machine_ids,
                &normalized_symbols,
            );
        }
        catalog.metadata.insert(
            "universe_symbols".to_string(),
            json!(normalized_symbols.clone()),
        );
    }

    if let Some(risk_plane) = expanded.risk_plane.as_mut() {
        risk_plane.machine_ids = graph
            .risk_plane
            .as_ref()
            .map(|plane| {
                normalized_symbols
                    .iter()
                    .flat_map(|symbol| {
                        plane
                            .machine_ids
                            .iter()
                            .map(move |machine_id| expanded_v4_machine_id(symbol, machine_id))
                    })
                    .collect()
            })
            .unwrap_or_default();
    }

    expanded.validate_static_contract().map_err(|errors| {
        anyhow!(
            "expanded v4 multi-symbol graph failed static validation: {:?}",
            errors
        )
    })?;
    Ok(expanded)
}

pub fn normalize_v4_backtest_symbols(symbols: &[String]) -> Vec<String> {
    let mut normalized = symbols
        .iter()
        .map(|symbol| symbol.trim().to_ascii_uppercase())
        .filter(|symbol| !symbol.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        normalized.push("BTCUSDT".to_string());
    }
    normalized
}

fn expanded_v4_machine_id(symbol: &str, base: &str) -> String {
    format!("{}::{}", sanitize_v4_symbol_for_id(symbol), base)
}

fn collect_v4_machine_family_ids(machines: &[V4MachineContract]) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for machine in machines {
        collect_v4_machine_family_ids_inner(machine, &mut ids);
    }
    ids
}

fn collect_v4_machine_family_ids_inner(machine: &V4MachineContract, ids: &mut BTreeSet<String>) {
    ids.insert(machine.machine_id.clone());
    for state in &machine.states {
        if let Some(child_machine) = state.child_machine.as_deref() {
            collect_v4_machine_family_ids_inner(child_machine, ids);
        }
    }
}

fn prefix_v4_machine_family_for_symbol(
    machine: &mut V4MachineContract,
    symbol: &str,
    machine_ids: &BTreeSet<String>,
) {
    machine.machine_id = expanded_v4_machine_id(symbol, &machine.machine_id);
    for transition in &mut machine.transitions {
        transition.transition_id = expanded_v4_machine_id(symbol, &transition.transition_id);
        if let Some(source) = transition.event.source.as_mut() {
            if machine_ids.contains(source.as_str()) {
                *source = expanded_v4_machine_id(symbol, source);
            }
        }
    }
    for state in &mut machine.states {
        if let Some(child_machine) = state.child_machine.as_deref_mut() {
            prefix_v4_machine_family_for_symbol(child_machine, symbol, machine_ids);
        }
    }
}

fn sanitize_v4_symbol_for_id(symbol: &str) -> String {
    symbol
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn expand_v4_event_party_list(
    parties: &[String],
    machine_ids: &BTreeSet<String>,
    symbols: &[String],
) -> Vec<String> {
    if parties.is_empty() {
        return Vec::new();
    }
    let mut expanded = BTreeSet::new();
    for party in parties {
        if machine_ids.contains(party.as_str()) {
            for symbol in symbols {
                expanded.insert(expanded_v4_machine_id(symbol, party));
            }
        } else {
            expanded.insert(party.clone());
        }
    }
    expanded.into_iter().collect()
}

pub(super) fn symbol_for_machine_id(machine_id: &str) -> Option<String> {
    machine_id
        .split_once("::")
        .map(|(symbol, _)| symbol.to_ascii_uppercase())
}
