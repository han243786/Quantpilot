use crate::Diagnostic;
use qrpc_core_ir::v4::{
    V4CompileTimeCapabilityRequest, V4MachineGraphContract,
    V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION, V4_MACHINE_GRAPH_CONTRACT_VERSION,
};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{
    derive_event_catalog, diag, parse_edge, parse_execution_capability, parse_machine_block,
    parse_qs_type_ref, parse_risk_plane, parse_runtime_mode, prepare_lines, split_words,
    V4_DEFAULT_MARKET_DATA_SOURCE,
};

pub(super) struct ParsedV4QsStaticDocument {
    pub(super) graph: V4MachineGraphContract,
    pub(super) request: V4CompileTimeCapabilityRequest,
}

pub(super) fn parse_v4_static_document(
    input: &str,
) -> Result<ParsedV4QsStaticDocument, Vec<Diagnostic>> {
    let lines = prepare_lines(input);
    let mut diagnostics = Vec::new();
    let Some(header) = lines.first() else {
        return Err(vec![diag(
            "QSV4000",
            "v4 QS 静态审计需要 v4_strategy 顶层块",
            1,
        )]);
    };

    let header_parts = split_words(&header.text);
    if header_parts.len() != 3 || header_parts[0] != "v4_strategy" || header_parts[2] != "{" {
        return Err(vec![diag(
            "QSV4001",
            "v4 QS 顶层语法必须是 `v4_strategy <graph_id> {`",
            header.number,
        )]);
    }
    let graph_id = header_parts[1].to_string();
    let mut venue_id = None;
    let mut runtime_mode = None;
    let mut required_execution_capabilities = Vec::new();
    let mut required_type_refs = Vec::new();
    let mut required_plugin_ids = Vec::new();
    let mut machines = Vec::new();
    let mut edges = Vec::new();
    let mut risk_plane = None;

    let mut index = 1;
    let mut edge_index = 0usize;
    while index < lines.len() {
        let line = &lines[index];
        if line.text == "}" {
            index += 1;
            break;
        }

        if let Some(rest) = line.text.strip_prefix("venue ") {
            venue_id = Some(rest.trim().to_string());
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("mode ") {
            match parse_runtime_mode(rest.trim()) {
                Ok(mode) => runtime_mode = Some(mode),
                Err(message) => diagnostics.push(diag("QSV4002", message, line.number)),
            }
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("require capability ") {
            match parse_execution_capability(rest.trim()) {
                Ok(capability) => required_execution_capabilities.push(capability),
                Err(message) => diagnostics.push(diag("QSV4003", message, line.number)),
            }
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("require type ") {
            match parse_qs_type_ref(rest.trim()) {
                Ok(type_ref) => required_type_refs.push(type_ref),
                Err(message) => diagnostics.push(diag("QSV4004", message, line.number)),
            }
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("require plugin ") {
            let plugin_id = rest.trim();
            if plugin_id.is_empty() {
                diagnostics.push(diag(
                    "QSV4005",
                    "require plugin 必须声明 plugin id",
                    line.number,
                ));
            } else {
                required_plugin_ids.push(plugin_id.to_string());
            }
            index += 1;
            continue;
        }
        if line.text.starts_with("machine ") {
            match parse_machine_block(&lines, index, 1, None) {
                Ok((machine, next_index)) => {
                    machines.push(machine.machine);
                    index = next_index;
                }
                Err((errors, next_index)) => {
                    diagnostics.extend(errors);
                    index = next_index;
                }
            }
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("edge ") {
            match parse_edge(rest, edge_index, line.number) {
                Ok(edge) => {
                    edges.push(edge);
                    edge_index += 1;
                }
                Err(error) => diagnostics.push(error),
            }
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("risk_plane ") {
            match parse_risk_plane(rest, line.number) {
                Ok(parsed_risk_plane) => risk_plane = Some(parsed_risk_plane),
                Err(error) => diagnostics.push(error),
            }
            index += 1;
            continue;
        }

        diagnostics.push(diag(
            "QSV4006",
            format!("v4 QS 顶层不支持的语句: {}", line.text),
            line.number,
        ));
        index += 1;
    }

    if index < lines.len() {
        diagnostics.push(diag(
            "QSV4007",
            "v4_strategy 顶层块结束后不能继续声明内容",
            lines[index].number,
        ));
    }
    if venue_id.as_deref().unwrap_or_default().is_empty() {
        diagnostics.push(diag("QSV4008", "v4 QS 必须声明 venue", header.number));
    }
    if runtime_mode.is_none() {
        diagnostics.push(diag("QSV4009", "v4 QS 必须声明 mode", header.number));
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let resolved_venue_id = venue_id.unwrap_or_default();
    let mut graph_metadata = BTreeMap::new();
    graph_metadata.insert(
        "default_venue_id".to_string(),
        Value::String(resolved_venue_id.clone()),
    );
    graph_metadata.insert(
        "market_event_source".to_string(),
        Value::String(V4_DEFAULT_MARKET_DATA_SOURCE.to_string()),
    );

    let mut graph = V4MachineGraphContract {
        schema_version: V4_MACHINE_GRAPH_CONTRACT_VERSION.to_string(),
        graph_id: graph_id.clone(),
        machines,
        edges,
        event_catalog: None,
        risk_plane,
        metadata: graph_metadata,
    };
    graph.event_catalog = Some(derive_event_catalog(&graph));

    Ok(ParsedV4QsStaticDocument {
        request: V4CompileTimeCapabilityRequest {
            schema_version: V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION.to_string(),
            graph_id,
            venue_id: resolved_venue_id,
            runtime_mode: runtime_mode
                .expect("runtime_mode is validated before graph construction"),
            required_execution_capabilities,
            required_type_refs,
            required_plugin_ids,
        },
        graph,
    })
}
