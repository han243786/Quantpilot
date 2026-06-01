mod data_source_lowering;
mod intent_lowering;
mod profile_lowering;

use quantscript::{parse_quant_script_module, ScriptModule};
use serde_json::Value;

pub(crate) fn convert_graph_json_to_script_module(
    graph_value: &Value,
) -> anyhow::Result<ScriptModule> {
    let nodes = graph_value
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("graph.nodes 必须是数组"))?;
    let edges = graph_value
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("graph.edges 必须是数组"))?;

    // Build a minimal QS source from the graph
    let mut qs_lines: Vec<String> = vec!["fn strategy() {".to_string()];

    data_source_lowering::append_data_source_lowering_lines(nodes, &mut qs_lines);

    // Agent/runtime nodes are represented by the inferred runtime graph and metadata, not calls.
    for node in nodes {
        if profile_lowering::append_profile_lowering_line(node, &mut qs_lines) {
            continue;
        }

        let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        match node_type {
            "data" | "intent" | "agent" | "runtime" | "runtime_control" => {}
            _ => {
                safe_eprintln!("[graph->QS] 未知节点类型 '{}', 跳过 QS 生成", node_type);
            }
        }
    }

    intent_lowering::append_intent_lowering_lines(nodes, edges, &mut qs_lines)?;

    qs_lines.push("}".to_string());
    let qs_source = qs_lines.join("\n");

    // Parse the generated QS source into a ScriptModule
    parse_quant_script_module(&qs_source)
}
