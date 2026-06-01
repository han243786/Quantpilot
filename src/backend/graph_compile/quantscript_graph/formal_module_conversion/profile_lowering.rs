use serde_json::Value;

pub(super) fn append_profile_lowering_line(node: &Value, qs_lines: &mut Vec<String>) -> bool {
    let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
    let cfg = node.get("config").unwrap_or(&Value::Null);
    match node_type {
        "risk" => {
            append_risk_profile_line(cfg, qs_lines);
            true
        }
        "execution" => {
            append_execution_profile_line(cfg, qs_lines);
            true
        }
        _ => false,
    }
}

fn append_risk_profile_line(cfg: &Value, qs_lines: &mut Vec<String>) {
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

fn append_execution_profile_line(cfg: &Value, qs_lines: &mut Vec<String>) {
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

#[cfg(test)]
mod tests {
    use super::append_profile_lowering_line;
    use serde_json::json;

    #[test]
    fn profile_lowering_renders_risk_and_execution_lines_with_existing_defaults() {
        let risk_node = json!({
            "type": "risk",
            "config": {
                "profile_name": "intraday",
                "max_position": 0.4,
                "max_total_leverage": 2.5,
                "max_exchange_leverage": 1.5,
                "min_action_interval_ms": 250
            }
        });
        let execution_node = json!({
            "type": "execution",
            "config": {
                "mode": "paper",
                "fee_bps": 8.0,
                "slippage_bps": 3.0
            }
        });
        let ignored_node = json!({ "type": "agent", "config": {} });
        let mut qs_lines = Vec::new();

        assert!(append_profile_lowering_line(&risk_node, &mut qs_lines));
        assert!(append_profile_lowering_line(&execution_node, &mut qs_lines));
        assert!(!append_profile_lowering_line(&ignored_node, &mut qs_lines));

        assert_eq!(
            qs_lines,
            vec![
                "    risk.profile(\"intraday\", max_position=0.4, max_total_leverage=2.5, max_exchange_leverage=1.5, min_action_interval_ms=250)",
                "    execution.profile(\"paper\", fee_bps=8, slippage_bps=3)",
            ]
        );
    }
}
