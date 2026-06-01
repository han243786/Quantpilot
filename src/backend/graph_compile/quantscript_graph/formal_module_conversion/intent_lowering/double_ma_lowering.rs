use serde_json::Value;

pub(super) fn append_double_ma_lowering_lines(
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
) {
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
