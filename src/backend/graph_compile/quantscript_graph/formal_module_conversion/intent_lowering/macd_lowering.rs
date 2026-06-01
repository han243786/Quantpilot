use serde_json::Value;

pub(super) fn append_macd_lowering_lines(
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
) {
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
