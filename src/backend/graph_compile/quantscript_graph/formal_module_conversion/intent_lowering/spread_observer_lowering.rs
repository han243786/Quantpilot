use serde_json::Value;

pub(super) fn append_spread_observer_lowering_lines(
    node: &Value,
    edges: &[Value],
    cfg: &Value,
    node_id: &str,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
) {
    let _ = node;
    let upstream_sources = edges
        .iter()
        .filter(|e| e.get("target_node_id").and_then(Value::as_str) == Some(node_id))
        .filter_map(|e| e.get("source_node_id").and_then(Value::as_str))
        .map(|source| source.replace(['-', '.'], "_"))
        .collect::<Vec<_>>();
    let left_source = upstream_sources
        .first()
        .cloned()
        .unwrap_or_else(|| source_var.to_string());
    let right_source = upstream_sources
        .get(1)
        .cloned()
        .unwrap_or_else(|| left_source.clone());
    let tolerance_ms = cfg
        .get("max_time_diff_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(5000);
    let output = match cfg.get("spread_output_code").and_then(|v| v.as_u64()) {
        Some(1) => "bps",
        _ => "ratio",
    };
    let threshold = cfg
        .get("comparison_threshold")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let op = match cfg.get("comparison_op_code").and_then(|v| v.as_u64()) {
        Some(3) => ">=",
        _ => ">",
    };
    qs_lines.push(format!(
        "    let {}_left = align_asof(field({}, name=\"bid\"), direction=\"backward\", tolerance_ms={})",
        node_id, left_source, tolerance_ms
    ));
    qs_lines.push(format!(
        "    let {}_right = align_asof(field({}, name=\"ask\"), direction=\"backward\", tolerance_ms={})",
        node_id, right_source, tolerance_ms
    ));
    qs_lines.push(format!(
        "    let {}_signal = spread({}_left, {}_right, output=\"{}\")",
        node_id, node_id, node_id, output
    ));
    qs_lines.push(format!("    if {}_signal {} {} {{", node_id, op, threshold));
    qs_lines.push(format!(
        "        emit Intent(\"BUY\", instrument=\"{}\", quantity=1.0)",
        instrument
    ));
    qs_lines.push("    }".to_string());
}
