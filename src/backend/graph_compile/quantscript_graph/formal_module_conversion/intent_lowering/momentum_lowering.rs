use serde_json::Value;

pub(super) fn append_momentum_lowering_lines(
    node_id: &str,
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
) {
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
