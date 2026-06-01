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

    // Add fetch calls for data nodes
    for node in nodes {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        if node_type == "data" {
            let cfg = node.get("config").unwrap_or(&Value::Null);
            let exchange = cfg
                .get("exchange")
                .and_then(Value::as_str)
                .unwrap_or("binance");
            let instrument = cfg
                .get("instrument")
                .and_then(Value::as_str)
                .unwrap_or("BTCUSDT");
            let interval = cfg.get("timeframe").and_then(Value::as_str).unwrap_or("1d");
            // v1.3.6: 拒绝负数窗口大小，防止静默回退默认值
            let lookback = cfg
                .get("window_size")
                .and_then(|v| v.as_f64())
                .filter(|&n| n >= 1.0)
                .map(|n| n as u64)
                .unwrap_or(200);
            let mut fetch_args = vec![
                format!("exchange=\"{}\"", exchange),
                format!("interval=\"{}\"", interval),
                format!("lookback={}", lookback),
            ];
            if let Some(ping_enabled) = cfg.get("ping_enabled").and_then(Value::as_bool) {
                fetch_args.push(format!("ping_enabled={}", ping_enabled));
            }
            if let Some(request_interval_ms) =
                cfg.get("request_interval_ms").and_then(Value::as_u64)
            {
                fetch_args.push(format!("request_interval_ms={}", request_interval_ms));
            }
            let node_id = node.get("id").and_then(Value::as_str).unwrap_or("data");
            let var_name = node_id.replace(['-', '.'], "_");
            qs_lines.push(format!(
                "    let {} = fetch(\"{}\", {})?",
                var_name,
                instrument,
                fetch_args.join(", ")
            ));
        }
    }

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
