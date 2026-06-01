mod double_ma_lowering;
mod ma_deviation_lowering;
mod macd_lowering;
mod rsi_lowering;
mod spread_observer_lowering;

use serde_json::Value;

pub(super) fn append_intent_lowering_lines(
    nodes: &[Value],
    edges: &[Value],
    qs_lines: &mut Vec<String>,
) -> anyhow::Result<()> {
    // Add indicator/emit calls for intent nodes
    for node in nodes {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        if node_type == "intent" {
            let module_key = node.get("module_key").and_then(Value::as_str).unwrap_or("");
            let cfg = node.get("config").unwrap_or(&Value::Null);
            let instrument = cfg
                .get("instrument")
                .and_then(Value::as_str)
                .unwrap_or("BTCUSDT");

            // Find upstream data node
            let node_id = node.get("id").and_then(Value::as_str).unwrap_or("");
            let upstream_edge = edges
                .iter()
                .find(|e| e.get("target_node_id").and_then(Value::as_str) == Some(node_id));
            let source_id = upstream_edge
                .and_then(|e| e.get("source_node_id").and_then(Value::as_str))
                .unwrap_or("data");
            let source_var = source_id.replace(['-', '.'], "_");

            match module_key {
                "builtin.intent.double_ma" => {
                    double_ma_lowering::append_double_ma_lowering_lines(
                        cfg,
                        &source_var,
                        instrument,
                        qs_lines,
                    );
                }
                "builtin.intent.rsi" => {
                    rsi_lowering::append_rsi_lowering_lines(
                        node_id,
                        cfg,
                        &source_var,
                        instrument,
                        qs_lines,
                    );
                }
                "builtin.intent.ma_deviation" => {
                    ma_deviation_lowering::append_ma_deviation_lowering_lines(
                        cfg,
                        &source_var,
                        instrument,
                        qs_lines,
                    );
                }
                "builtin.intent.macd" => {
                    macd_lowering::append_macd_lowering_lines(
                        cfg,
                        &source_var,
                        instrument,
                        qs_lines,
                    );
                }
                "builtin.intent.momentum" => {
                    let lookback = cfg.get("lookback").and_then(|v| v.as_u64()).unwrap_or(10);
                    let threshold = cfg
                        .get("threshold_ratio")
                        .or_else(|| cfg.get("threshold"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.02);
                    qs_lines.push(format!(
                        "    let {}_signal = momentum({}, {})",
                        node_id, source_var, lookback
                    ));
                    qs_lines.push(format!("    if {}_signal > {} {{", node_id, threshold));
                    qs_lines.push(format!(
                        "        emit Intent(\"BUY\", instrument=\"{}\", quantity=1.0)",
                        instrument
                    ));
                    qs_lines.push("    }".to_string());
                }
                "builtin.intent.zscore" => {
                    let window = cfg.get("window").and_then(|v| v.as_u64()).unwrap_or(20);
                    let entry_z = cfg.get("entry_z").and_then(|v| v.as_f64()).unwrap_or(2.0);
                    qs_lines.push(format!(
                        "    let {}_signal = zscore({}, {})",
                        node_id, source_var, window
                    ));
                    qs_lines.push(format!("    if {}_signal < -{} {{", node_id, entry_z.abs()));
                    qs_lines.push(format!(
                        "        emit Intent(\"BUY\", instrument=\"{}\", quantity=1.0)",
                        instrument
                    ));
                    qs_lines.push("    }".to_string());
                }
                "builtin.intent.spread_observer" => {
                    spread_observer_lowering::append_spread_observer_lowering_lines(
                        node,
                        edges,
                        cfg,
                        node_id,
                        &source_var,
                        instrument,
                        qs_lines,
                    );
                }
                _ => {
                    // v2.3.3 修复 S0-5: 未知意图模块键不再静默丢弃, 返回明确错误
                    let supported =
                        "double_ma/ma_deviation/rsi/macd/momentum/zscore/spread_observer";
                    anyhow::bail!(
                        "不支持的意图模块 '{}': 当前版本仅支持 {}。请升级到支持该模块的版本。",
                        module_key,
                        supported
                    );
                }
            }
        }
    }

    Ok(())
}
