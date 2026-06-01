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
                    let fast = cfg
                        .get("fast_period")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(20);
                    let slow = cfg
                        .get("slow_period")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(50);
                    qs_lines.push(format!("    let fast = sma({}, {})", source_var, fast));
                    qs_lines.push(format!("    let slow = sma({}, {})", source_var, slow));
                    qs_lines.push("    if fast > slow {".to_string());
                    qs_lines.push(format!(
                        "        emit Intent(\"BUY\", instrument=\"{}\", quantity=1.0)",
                        instrument
                    ));
                    qs_lines.push("    }".to_string());
                }
                "builtin.intent.rsi" => {
                    let period = cfg.get("period").and_then(|v| v.as_u64()).unwrap_or(14);
                    let oversold = cfg
                        .get("oversold_threshold")
                        .or_else(|| cfg.get("oversold"))
                        .and_then(Value::as_f64)
                        .unwrap_or(30.0);
                    qs_lines.push(format!(
                        "    let {}_signal = rsi({}, {})",
                        node_id, source_var, period
                    ));
                    qs_lines.push(format!("    if {}_signal < {} {{", node_id, oversold));
                    qs_lines.push(format!(
                        "        emit Intent(\"BUY\", instrument=\"{}\", quantity=1.0)",
                        instrument
                    ));
                    qs_lines.push("    }".to_string());
                }
                "builtin.intent.ma_deviation" => {
                    let lookback = cfg.get("lookback").and_then(|v| v.as_u64()).unwrap_or(15);
                    let baseline = cfg
                        .get("baseline_period")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(150);
                    qs_lines.push(format!(
                        "    let ma_dev = sma({}, {}) / sma({}, {})",
                        source_var, lookback, source_var, baseline
                    ));
                    qs_lines.push("    if ma_dev > 1 {".to_string());
                    qs_lines.push(format!(
                        "        emit Intent(\"SELL\", instrument=\"{}\", quantity=1.0)",
                        instrument
                    ));
                    qs_lines.push("    }".to_string());
                }
                "builtin.intent.macd" => {
                    let fast = cfg
                        .get("fast_period")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(12);
                    let slow = cfg
                        .get("slow_period")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(26);
                    let signal_period = cfg
                        .get("signal_period")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(9);
                    qs_lines.push(format!(
                        "    let macd_val = macd({}, {}, {}, {})",
                        source_var, fast, slow, signal_period
                    ));
                    qs_lines.push("    if macd_val > 0 {".to_string());
                    qs_lines.push(format!(
                        "        emit Intent(\"BUY\", instrument=\"{}\", quantity=1.0)",
                        instrument
                    ));
                    qs_lines.push("    } else if macd_val < 0 {".to_string());
                    qs_lines.push(format!(
                        "        emit Intent(\"SELL\", instrument=\"{}\", quantity=1.0)",
                        instrument
                    ));
                    qs_lines.push("    }".to_string());
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
