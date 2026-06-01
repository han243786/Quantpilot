use serde_json::Value;

pub(super) fn append_ma_deviation_lowering_lines(
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
) {
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
