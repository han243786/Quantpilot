use serde_json::Value;

pub(super) fn append_data_source_lowering_lines(nodes: &[Value], qs_lines: &mut Vec<String>) {
    for node in nodes {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        if node_type == "data" {
            append_data_source_line(node, qs_lines);
        }
    }
}

fn append_data_source_line(node: &Value, qs_lines: &mut Vec<String>) {
    let cfg = node.get("config").unwrap_or(&Value::Null);
    let exchange = cfg
        .get("exchange")
        .and_then(Value::as_str)
        .unwrap_or("binance");
    let instrument = cfg
        .get("instrument")
        .and_then(Value::as_str)
        .unwrap_or("BTCUSDT");
    let interval = cfg.get("timeframe").and_then(Value::as_str).unwrap_or("1d");
    let lookback = cfg
        .get("window_size")
        .and_then(|v| v.as_f64())
        .filter(|&n| n >= 1.0)
        .map(|n| n as u64)
        .unwrap_or(200);
    let mut fetch_args = vec![
        format!("exchange=\"{}\"", exchange),
        format!("interval=\"{}\"", interval),
        format!("lookback={}", lookback),
    ];
    if let Some(ping_enabled) = cfg.get("ping_enabled").and_then(Value::as_bool) {
        fetch_args.push(format!("ping_enabled={}", ping_enabled));
    }
    if let Some(request_interval_ms) = cfg.get("request_interval_ms").and_then(Value::as_u64) {
        fetch_args.push(format!("request_interval_ms={}", request_interval_ms));
    }
    let node_id = node.get("id").and_then(Value::as_str).unwrap_or("data");
    let var_name = node_id.replace(['-', '.'], "_");
    qs_lines.push(format!(
        "    let {} = fetch(\"{}\", {})?",
        var_name,
        instrument,
        fetch_args.join(", ")
    ));
}

#[cfg(test)]
mod tests {
    use super::append_data_source_lowering_lines;
    use serde_json::json;

    #[test]
    fn data_source_lowering_renders_fetch_line_with_existing_defaults_and_options() {
        let nodes = vec![json!({
            "id": "data-1.2",
            "type": "data",
            "config": {
                "exchange": "okx",
                "instrument": "ETHUSDT",
                "timeframe": "1h",
                "window_size": 50,
                "ping_enabled": true,
                "request_interval_ms": 1000
            }
        })];
        let mut qs_lines = Vec::new();

        append_data_source_lowering_lines(&nodes, &mut qs_lines);

        assert_eq!(
            qs_lines,
            vec![
                "    let data_1_2 = fetch(\"ETHUSDT\", exchange=\"okx\", interval=\"1h\", lookback=50, ping_enabled=true, request_interval_ms=1000)?"
            ]
        );
    }
}
