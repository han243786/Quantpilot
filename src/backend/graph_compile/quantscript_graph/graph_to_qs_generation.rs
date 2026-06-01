use serde_json::Value;

pub(crate) fn generate_quantscript_from_graph_value(graph: &Value) -> anyhow::Result<String> {
    let metadata = graph
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("graph.metadata 必须是对象"))?;
    let graph_id = metadata
        .get("graph_id")
        .and_then(Value::as_str)
        .unwrap_or("graph");
    let name = metadata
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Untitled Strategy");
    let version = metadata
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("1.0.0");
    let nodes = graph
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("graph.nodes 必须是数组"))?;
    let edges = graph
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("graph.edges 必须是数组"))?;
    let mode = nodes
        .iter()
        .find(|node| node.get("type").and_then(Value::as_str) == Some("runtime"))
        .and_then(|node| node.get("config"))
        .and_then(|config| config.get("mode"))
        .and_then(Value::as_str)
        .unwrap_or("paper");

    let mut lines = vec![
        format!("strategy_graph {} {{", graph_id),
        format!("  name: {}", quoted(name)),
        format!("  version: {}", quoted(version)),
        format!("  mode: {}", quoted(mode)),
        String::new(),
        "  nodes:".to_string(),
    ];

    for node in nodes {
        for line in generate_node_quantscript(node, nodes, edges)?.lines() {
            lines.push(format!("    {}", line));
        }
        lines.push(String::new());
    }

    lines.push("  graph:".to_string());
    if edges.is_empty() {
        lines.push("    # no connections".to_string());
    } else {
        for edge in edges {
            let source_node_id = edge
                .get("source_node_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown_source");
            let source_port = edge
                .get("source_port")
                .and_then(Value::as_str)
                .unwrap_or("out");
            let target_node_id = edge
                .get("target_node_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown_target");
            let target_port = edge
                .get("target_port")
                .and_then(Value::as_str)
                .unwrap_or("in");
            lines.push(format!(
                "    connect {}.{} -> {}.{}",
                source_node_id, source_port, target_node_id, target_port
            ));
        }
    }
    lines.push("}".to_string());
    Ok(lines.join("\n"))
}

pub(super) fn generate_node_quantscript(
    node: &Value,
    nodes: &[Value],
    edges: &[Value],
) -> anyhow::Result<String> {
    let node_id = node.get("id").and_then(Value::as_str).unwrap_or("node");
    let node_type = node.get("type").and_then(Value::as_str).unwrap_or("plugin");
    let module_key = node
        .get("module_key")
        .and_then(Value::as_str)
        .or_else(|| node.get("type").and_then(Value::as_str))
        .unwrap_or("unknown.module");
    let name = node.get("name").and_then(Value::as_str).unwrap_or(node_id);
    let kind = match node_type {
        "runtime" => "runtime",
        "execution" => "execution",
        _ => "plugin",
    };
    let mut lines = vec![
        format!("{} {} uses {}", kind, node_id, module_key),
        format!("  name: {}", quoted(name)),
        format!("  category: {}", quoted(node_type)),
    ];

    if let Some(config) = node.get("config").and_then(Value::as_object) {
        if !config.is_empty() {
            lines.push("  config:".to_string());
            for (key, value) in config {
                lines.push(format!("    {}: {}", key, render_json_scalar(value)));
            }
        }
    }

    let inputs = edges
        .iter()
        .filter(|edge| edge.get("target_node_id").and_then(Value::as_str) == Some(node_id))
        .filter_map(|edge| {
            let source_node_id = edge.get("source_node_id").and_then(Value::as_str)?;
            let source_port = edge
                .get("source_port")
                .and_then(Value::as_str)
                .unwrap_or("out");
            let target_port = edge
                .get("target_port")
                .and_then(Value::as_str)
                .unwrap_or("in");
            let _source_node = nodes.iter().find(|candidate| {
                candidate.get("id").and_then(Value::as_str) == Some(source_node_id)
            })?;
            Some((
                source_node_id.to_string(),
                source_port.to_string(),
                target_port.to_string(),
            ))
        })
        .collect::<Vec<_>>();

    if !inputs.is_empty() {
        lines.push("  inputs:".to_string());
        for (source_id, source_port, target_port) in inputs {
            lines.push(format!("    - from: {}.{}", source_id, source_port));
            lines.push(format!("      to: {}.{}", node_id, target_port));
        }
    }

    Ok(lines.join("\n"))
}

fn quoted(input: &str) -> String {
    serde_json::to_string(input).unwrap_or_else(|_| format!("\"{}\"", input))
}

fn render_json_scalar(value: &Value) -> String {
    match value {
        Value::String(value) => quoted(value),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Null => "null".to_string(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()),
    }
}
