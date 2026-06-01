use serde_json::Value;

pub(super) fn parse_strategy_graph_source(source: &str, now: u64) -> anyhow::Result<Value> {
    let lines = source
        .lines()
        .map(|line| line.replace('\t', "    "))
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect::<Vec<_>>();

    let header = lines
        .first()
        .ok_or_else(|| anyhow::anyhow!("QuantScript 源码为空"))?;
    let header = header.trim();
    let header = header
        .strip_prefix("strategy_graph ")
        .ok_or_else(|| anyhow::anyhow!("策略图源码必须以 strategy_graph 开头"))?;
    let (graph_id, _) = header
        .split_once(" {")
        .ok_or_else(|| anyhow::anyhow!("无效的策略图源码头部"))?;

    let mut name = "Imported Strategy".to_string();
    let mut version = "1.0.0".to_string();
    let mut mode = "paper".to_string();
    let mut nodes = Vec::<Value>::new();
    let mut edges = Vec::<Value>::new();
    let mut index = 1usize;

    while index < lines.len() {
        let line = lines[index].trim();
        if line == "nodes:" {
            index += 1;
            break;
        }
        if let Some(value) = line.strip_prefix("name:") {
            name = parse_qs_scalar(value)
                .as_str()
                .unwrap_or("Imported Strategy")
                .to_string();
        } else if let Some(value) = line.strip_prefix("version:") {
            version = parse_qs_scalar(value)
                .as_str()
                .unwrap_or("1.0.0")
                .to_string();
        } else if let Some(value) = line.strip_prefix("mode:") {
            mode = parse_qs_scalar(value)
                .as_str()
                .unwrap_or("paper")
                .to_string();
        }
        index += 1;
    }

    while index < lines.len() {
        let line = lines[index].trim();
        if line == "graph:" {
            index += 1;
            break;
        }
        if let Some((kind, node_id, module_key)) = parse_qs_node_header(line) {
            let mut node_name = node_id.to_string();
            let mut node_type = match kind {
                "runtime" => "runtime".to_string(),
                "execution" => "execution".to_string(),
                _ => "data".to_string(),
            };
            let mut config = serde_json::Map::new();
            index += 1;
            while index < lines.len() {
                let raw = &lines[index];
                let trimmed = raw.trim();
                let indent = raw.chars().take_while(|ch| ch.is_whitespace()).count();
                if trimmed == "graph:" || parse_qs_node_header(trimmed).is_some() || indent <= 2 {
                    break;
                }
                if let Some(value) = trimmed.strip_prefix("name:") {
                    node_name = parse_qs_scalar(value)
                        .as_str()
                        .unwrap_or(node_id)
                        .to_string();
                } else if let Some(value) = trimmed.strip_prefix("category:") {
                    node_type = parse_qs_scalar(value)
                        .as_str()
                        .unwrap_or(&node_type)
                        .to_string();
                } else if trimmed == "config:" {
                    index += 1;
                    while index < lines.len() {
                        let raw = &lines[index];
                        let trimmed = raw.trim();
                        let indent = raw.chars().take_while(|ch| ch.is_whitespace()).count();
                        if trimmed.is_empty() || indent <= 4 {
                            break;
                        }
                        if let Some((key, value)) = trimmed.split_once(':') {
                            config.insert(key.trim().to_string(), parse_qs_scalar(value));
                        }
                        index += 1;
                    }
                    continue;
                }
                index += 1;
            }
            if node_type == "runtime" {
                config.insert("mode".to_string(), Value::String(mode.clone()));
            }
            nodes.push(serde_json::json!({
                "id": node_id,
                "type": node_type,
                "module_key": module_key,
                "name": node_name,
                "position": {
                    "x": 120 + (nodes.len() as i64 * 80),
                    "y": 120 + (nodes.len() as i64 * 40)
                },
                "config": Value::Object(config),
                "ui_state": { "collapsed": false },
                "runtime_state": {
                    "status": "idle",
                    "last_event_type": Value::Null,
                    "last_event_time": Value::Null,
                    "last_message": "",
                    "metrics": {},
                    "error": Value::Null
                }
            }));
            continue;
        }
        index += 1;
    }

    while index < lines.len() {
        let line = lines[index].trim();
        if line == "}" || line == "# no connections" {
            index += 1;
            continue;
        }
        if let Some((source_node_id, source_port, target_node_id, target_port)) =
            parse_qs_connect(line)
        {
            edges.push(serde_json::json!({
                "id": format!("edge_{}_{}_{}_{}", source_node_id, target_node_id, source_port, target_port),
                "source_node_id": source_node_id,
                "source_port": source_port,
                "target_node_id": target_node_id,
                "target_port": target_port,
                "edge_type": format!("{}-{}", source_node_id, target_node_id)
            }));
        }
        index += 1;
    }

    Ok(serde_json::json!({
        "metadata": {
            "graph_id": graph_id,
            "name": name,
            "description": "Imported from strategy_graph source",
            "version": version,
            "created_at": now,
            "updated_at": now,
            "runtime_binding": {
                "current_run_id": Value::Null,
                "last_compile_id": Value::Null
            },
            "editor": {
                "viewport": { "x": 0, "y": 0, "zoom": 0.8 }
            },
            "source_mode": "graph",
            "artifacts": {}
        },
        "nodes": nodes,
        "edges": edges,
        "validation_state": {
            "is_valid": false,
            "is_runnable": false,
            "node_issues": {},
            "edge_issues": {},
            "graph_issues": [],
            "issue_counts": { "error": 0, "warning": 0, "info": 0 },
            "last_validated_at": Value::Null
        },
        "compile_summary": {
            "compilable": false,
            "last_compile_id": Value::Null,
            "last_compile_at": Value::Null,
            "topology_order": [],
            "outputs": {
                "data_sources": 0,
                "intent_generators": 0,
                "agents": 0,
                "risk_controls": 0,
                "executions": 0
            },
            "warnings": [],
            "errors": []
        }
    }))
}

fn parse_qs_scalar(input: &str) -> Value {
    let trimmed = input.trim();
    if trimmed.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    if trimmed.eq_ignore_ascii_case("null") {
        return Value::Null;
    }
    if let Ok(number) = trimmed.parse::<i64>() {
        return Value::Number(number.into());
    }
    if let Ok(number) = trimmed.parse::<f64>() {
        if let Some(number) = serde_json::Number::from_f64(number) {
            return Value::Number(number);
        }
    }
    if ((trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
        && trimmed.len() >= 2
    {
        return Value::String(trimmed[1..trimmed.len() - 1].to_string());
    }
    Value::String(trimmed.to_string())
}

fn parse_qs_node_header(input: &str) -> Option<(&str, &str, &str)> {
    let mut parts = input.split_whitespace();
    let kind = parts.next()?;
    if kind != "runtime" && kind != "execution" && kind != "plugin" {
        return None;
    }
    let node_id = parts.next()?;
    if parts.next()? != "uses" {
        return None;
    }
    let module_key = parts.next()?;
    Some((kind, node_id, module_key))
}

fn parse_qs_connect(input: &str) -> Option<(&str, &str, &str, &str)> {
    let body = input.strip_prefix("connect ")?;
    let (left, right) = body.split_once(" -> ")?;
    let (source_node_id, source_port) = left.split_once('.')?;
    let (target_node_id, target_port) = right.split_once('.')?;
    Some((source_node_id, source_port, target_node_id, target_port))
}

#[cfg(test)]
mod tests {
    use super::{parse_qs_connect, parse_qs_node_header, parse_qs_scalar};
    use serde_json::Value;

    #[test]
    fn parse_qs_scalar_keeps_basic_literals() {
        assert_eq!(parse_qs_scalar("true"), Value::Bool(true));
        assert_eq!(parse_qs_scalar("false"), Value::Bool(false));
        assert_eq!(parse_qs_scalar("null"), Value::Null);
        assert_eq!(parse_qs_scalar("42"), Value::Number(42.into()));
        assert_eq!(parse_qs_scalar("1.5"), serde_json::json!(1.5));
        assert_eq!(
            parse_qs_scalar("'BTCUSDT'"),
            Value::String("BTCUSDT".to_string())
        );
    }

    #[test]
    fn parse_qs_node_header_keeps_supported_kinds_only() {
        assert_eq!(
            parse_qs_node_header("runtime run uses builtin.runtime.control"),
            Some(("runtime", "run", "builtin.runtime.control"))
        );
        assert_eq!(
            parse_qs_node_header("execution exec uses builtin.execution.paper"),
            Some(("execution", "exec", "builtin.execution.paper"))
        );
        assert_eq!(
            parse_qs_node_header("plugin feed uses builtin.data.kline"),
            Some(("plugin", "feed", "builtin.data.kline"))
        );
        assert_eq!(
            parse_qs_node_header("risk r uses builtin.risk.global"),
            None
        );
    }

    #[test]
    fn parse_qs_connect_keeps_port_mapping() {
        assert_eq!(
            parse_qs_connect("connect data.out -> intent.input"),
            Some(("data", "out", "intent", "input"))
        );
        assert_eq!(parse_qs_connect("data.out -> intent.input"), None);
    }
}
