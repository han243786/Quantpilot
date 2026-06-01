mod data_source_lowering;
mod intent_lowering;

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

    // Add top-level risk/execution declarations supported by formal QuantScript lowering.
    // Agent/runtime nodes are represented by the inferred runtime graph and metadata, not calls.
    for node in nodes {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        let cfg = node.get("config").unwrap_or(&Value::Null);
        match node_type {
            "risk" => {
                let profile = cfg
                    .get("profile_id")
                    .or_else(|| cfg.get("profile_name"))
                    .and_then(Value::as_str)
                    .unwrap_or("global");
                let max_pos = cfg
                    .get("max_position")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.2);
                let max_lev = cfg
                    .get("max_total_leverage")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(3.0);
                let max_exchange_lev = cfg
                    .get("max_exchange_leverage")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(3.0);
                let min_interval = cfg
                    .get("min_action_interval_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100);
                qs_lines.push(format!(
                    "    risk.profile(\"{}\", max_position={}, max_total_leverage={}, max_exchange_leverage={}, min_action_interval_ms={})",
                    profile, max_pos, max_lev, max_exchange_lev, min_interval
                ));
            }
            "execution" => {
                let profile = cfg
                    .get("profile_id")
                    .or_else(|| cfg.get("profile_name"))
                    .or_else(|| cfg.get("mode"))
                    .and_then(Value::as_str)
                    .unwrap_or("paper");
                let fee = cfg.get("fee_bps").and_then(|v| v.as_f64()).unwrap_or(10.0);
                let slip = cfg
                    .get("slippage_bps")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(5.0);
                qs_lines.push(format!(
                    "    execution.profile(\"{}\", fee_bps={}, slippage_bps={})",
                    profile, fee, slip
                ));
            }
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
