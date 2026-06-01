use serde_json::Value;

pub(super) fn append_rsi_lowering_lines(
    node_id: &str,
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
) {
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
