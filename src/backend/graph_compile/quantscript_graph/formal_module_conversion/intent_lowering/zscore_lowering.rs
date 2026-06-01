use serde_json::Value;

pub(super) fn append_zscore_lowering_lines(
    node_id: &str,
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
) {
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
