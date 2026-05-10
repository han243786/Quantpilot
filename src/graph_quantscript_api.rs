use super::*;
use quantscript::{parse_quant_script_module, ScriptModule};

pub(super) fn register_graph_quantscript_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/graphs/:graph_id/quantscript",
            get(load_graph_quantscript),
        )
        .route(
            "/api/quantscript/graph/parse",
            post(parse_graph_quantscript),
        )
}

pub(super) async fn load_graph_quantscript(
    State(state): State<AppState>,
    Path(graph_id): Path<String>,
) -> Result<String, (StatusCode, String)> {
    validate_graph_id(&graph_id).map_err(internal_error)?;
    let source_path = state.graph_store_dir.join(format!("{}.qs", graph_id));
    fs::read_to_string(&source_path)
        .await
        .map_err(not_found_io_error)
}

pub(super) async fn parse_graph_quantscript(
    Json(request): Json<ParseGraphQuantScriptRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    parse_graph_quantscript_source(&request.source)
        .map(Json)
        .map_err(internal_error)
}

pub(crate) fn generate_quantscript_from_graph_value(graph: &Value) -> anyhow::Result<String> {
    let metadata = graph
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("graph.metadata 必须是对象"))?;
    let graph_id = metadata
        .get("graph_id")
        .and_then(Value::as_str)
        .unwrap_or("graph");
    let name = metadata
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Untitled Strategy");
    let version = metadata
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("1.0.0");
    let nodes = graph
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("graph.nodes 必须是数组"))?;
    let edges = graph
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("graph.edges 必须是数组"))?;
    let mode = nodes
        .iter()
        .find(|node| node.get("type").and_then(Value::as_str) == Some("runtime"))
        .and_then(|node| node.get("config"))
        .and_then(|config| config.get("mode"))
        .and_then(Value::as_str)
        .unwrap_or("paper");

    let mut lines = vec![
        format!("strategy_graph {} {{", graph_id),
        format!("  name: {}", quoted(name)),
        format!("  version: {}", quoted(version)),
        format!("  mode: {}", quoted(mode)),
        String::new(),
        "  nodes:".to_string(),
    ];

    for node in nodes {
        for line in generate_node_quantscript(node, nodes, edges)?.lines() {
            lines.push(format!("    {}", line));
        }
        lines.push(String::new());
    }

    lines.push("  graph:".to_string());
    if edges.is_empty() {
        lines.push("    # no connections".to_string());
    } else {
        for edge in edges {
            let source_node_id = edge
                .get("source_node_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown_source");
            let source_port = edge
                .get("source_port")
                .and_then(Value::as_str)
                .unwrap_or("out");
            let target_node_id = edge
                .get("target_node_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown_target");
            let target_port = edge
                .get("target_port")
                .and_then(Value::as_str)
                .unwrap_or("in");
            lines.push(format!(
                "    connect {}.{} -> {}.{}",
                source_node_id, source_port, target_node_id, target_port
            ));
        }
    }
    lines.push("}".to_string());
    Ok(lines.join("\n"))
}

pub(crate) fn convert_graph_json_to_script_module(graph_value: &Value) -> anyhow::Result<ScriptModule> {
    let nodes = graph_value
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("graph.nodes 必须是数组"))?;
    let edges = graph_value
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("graph.edges 必须是数组"))?;

    // Build a minimal QS source from the graph
    let mut qs_lines: Vec<String> = vec!["fn strategy() {".to_string()];

    // Add fetch calls for data nodes
    for node in nodes {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        if node_type == "data" {
            let cfg = node.get("config").unwrap_or(&Value::Null);
            let exchange = cfg
                .get("exchange")
                .and_then(Value::as_str)
                .unwrap_or("binance");
            let instrument = cfg
                .get("instrument")
                .and_then(Value::as_str)
                .unwrap_or("BTCUSDT");
            let interval = cfg
                .get("timeframe")
                .and_then(Value::as_str)
                .unwrap_or("1d");
            let lookback = cfg
                .get("window_size")
                .and_then(|v| v.as_u64())
                .unwrap_or(200);
            let node_id = node.get("id").and_then(Value::as_str).unwrap_or("data");
            let var_name = node_id.replace('-', "_").replace('.', "_");
            qs_lines.push(format!(
                "    let {} = fetch(\"{}\", exchange=\"{}\", interval=\"{}\", lookback={})?",
                var_name, instrument, exchange, interval, lookback
            ));
        }
    }

    // Add indicator/emit calls for intent nodes
    for node in nodes {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        if node_type == "intent" {
            let module_key = node
                .get("module_key")
                .and_then(Value::as_str)
                .unwrap_or("");
            let cfg = node.get("config").unwrap_or(&Value::Null);
            let instrument = cfg
                .get("instrument")
                .and_then(Value::as_str)
                .unwrap_or("BTCUSDT");

            // Find upstream data node
            let node_id = node.get("id").and_then(Value::as_str).unwrap_or("");
            let upstream_edge = edges.iter().find(|e| {
                e.get("target_node_id").and_then(Value::as_str) == Some(node_id)
            });
            let source_id = upstream_edge
                .and_then(|e| e.get("source_node_id").and_then(Value::as_str))
                .unwrap_or("data");
            let source_var = source_id.replace('-', "_").replace('.', "_");

            match module_key {
                "builtin.intent.double_ma" => {
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
                "builtin.intent.rsi" => {
                    let period = cfg
                        .get("period")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(14);
                    qs_lines.push(format!("    let rsi_val = rsi({}, {})", source_var, period));
                    qs_lines.push("    if rsi_val < 30 {".to_string());
                    qs_lines.push(format!(
                        "        emit Intent(\"BUY\", instrument=\"{}\", quantity=1.0)",
                        instrument
                    ));
                    qs_lines.push("    } else if rsi_val > 70 {".to_string());
                    qs_lines.push(format!(
                        "        emit Intent(\"SELL\", instrument=\"{}\", quantity=1.0)",
                        instrument
                    ));
                    qs_lines.push("    }".to_string());
                }
                _ => {
                    // Generic intent — use sma crossover as fallback
                    qs_lines.push(format!("    let fast = sma({}, 20)", source_var));
                    qs_lines.push(format!("    let slow = sma({}, 50)", source_var));
                    qs_lines.push("    if fast > slow {".to_string());
                    qs_lines.push(format!(
                        "        emit Intent(\"BUY\", instrument=\"{}\", quantity=1.0)",
                        instrument
                    ));
                    qs_lines.push("    }".to_string());
                }
            }
        }
    }

    qs_lines.push("}".to_string());
    let qs_source = qs_lines.join("\n");

    // Parse the generated QS source into a ScriptModule
    parse_quant_script_module(&qs_source)
}

fn generate_node_quantscript(
    node: &Value,
    nodes: &[Value],
    edges: &[Value],
) -> anyhow::Result<String> {
    let node_id = node.get("id").and_then(Value::as_str).unwrap_or("node");
    let node_type = node.get("type").and_then(Value::as_str).unwrap_or("plugin");
    let module_key = node
        .get("module_key")
        .and_then(Value::as_str)
        .or_else(|| node.get("type").and_then(Value::as_str))
        .unwrap_or("unknown.module");
    let name = node.get("name").and_then(Value::as_str).unwrap_or(node_id);
    let kind = match node_type {
        "runtime" => "runtime",
        "execution" => "execution",
        _ => "plugin",
    };
    let mut lines = vec![
        format!("{} {} uses {}", kind, node_id, module_key),
        format!("  name: {}", quoted(name)),
        format!("  category: {}", quoted(node_type)),
    ];

    if let Some(config) = node.get("config").and_then(Value::as_object) {
        if !config.is_empty() {
            lines.push("  config:".to_string());
            for (key, value) in config {
                lines.push(format!("    {}: {}", key, render_json_scalar(value)));
            }
        }
    }

    let inputs = edges
        .iter()
        .filter(|edge| edge.get("target_node_id").and_then(Value::as_str) == Some(node_id))
        .filter_map(|edge| {
            let source_node_id = edge.get("source_node_id").and_then(Value::as_str)?;
            let source_port = edge
                .get("source_port")
                .and_then(Value::as_str)
                .unwrap_or("out");
            let target_port = edge
                .get("target_port")
                .and_then(Value::as_str)
                .unwrap_or("in");
            let _source_node = nodes.iter().find(|candidate| {
                candidate.get("id").and_then(Value::as_str) == Some(source_node_id)
            })?;
            Some((
                source_node_id.to_string(),
                source_port.to_string(),
                target_port.to_string(),
            ))
        })
        .collect::<Vec<_>>();

    if !inputs.is_empty() {
        lines.push("  inputs:".to_string());
        for (source_id, source_port, target_port) in inputs {
            lines.push(format!("    - from: {}.{}", source_id, source_port));
            lines.push(format!("      to: {}.{}", node_id, target_port));
        }
    }

    Ok(lines.join("\n"))
}

pub(super) fn attach_quantscript_artifacts(
    graph: &mut Value,
    quantscript: &str,
    generated_at: u64,
    quantscript_path: &std::path::Path,
) {
    let node_sources = build_quantscript_node_sources(graph);
    let label_targets = build_quantscript_label_targets(graph);
    let runtime_targets = build_quantscript_runtime_targets(graph);
    let formal_source = graph
        .get("metadata")
        .and_then(|metadata| metadata.get("artifacts"))
        .and_then(|artifacts| artifacts.get("quantscript"))
        .and_then(|quantscript| quantscript.get("formal_source"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let Some(root) = graph.as_object_mut() else {
        return;
    };
    let metadata = root
        .entry("metadata")
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let Some(metadata) = metadata.as_object_mut() else {
        return;
    };
    let existing_source_mode = metadata
        .get("source_mode")
        .and_then(Value::as_str)
        .unwrap_or("graph");
    metadata.insert(
        "source_mode".to_string(),
        Value::String(existing_source_mode.to_string()),
    );
    let artifacts = metadata
        .entry("artifacts")
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let Some(artifacts) = artifacts.as_object_mut() else {
        return;
    };
    let mut quantscript_value = serde_json::Map::new();
    quantscript_value.insert(
        "graph_source".to_string(),
        Value::String(quantscript.to_string()),
    );
    quantscript_value.insert("formal_source".to_string(), Value::String(formal_source));
    quantscript_value.insert("node_sources".to_string(), Value::Object(node_sources));
    quantscript_value.insert("label_targets".to_string(), Value::Object(label_targets));
    quantscript_value.insert("runtime_targets".to_string(), runtime_targets);
    quantscript_value.insert(
        "generated_at".to_string(),
        Value::Number(serde_json::Number::from(generated_at)),
    );
    quantscript_value.insert(
        "saved_path".to_string(),
        Value::String(quantscript_path.to_string_lossy().to_string()),
    );
    artifacts.insert("quantscript".to_string(), Value::Object(quantscript_value));
}

fn build_quantscript_node_sources(graph: &Value) -> serde_json::Map<String, Value> {
    let nodes = graph
        .get("nodes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let edges = graph
        .get("edges")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    nodes
        .iter()
        .filter_map(|node| {
            let node_id = node.get("id").and_then(Value::as_str)?.to_string();
            let source = generate_node_quantscript(node, &nodes, &edges).ok()?;
            Some((node_id, Value::String(source)))
        })
        .collect()
}

fn build_quantscript_label_targets(graph: &Value) -> serde_json::Map<String, Value> {
    let nodes = graph
        .get("nodes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut targets = serde_json::Map::new();

    for node in nodes {
        let Some(node_id) = node.get("id").and_then(Value::as_str) else {
            continue;
        };
        let node_name = node
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(node_id)
            .to_string();

        let base_target = diagnostic_target_value(
            "node",
            Some(node_id.to_string()),
            None,
            None,
            Some(node_name.clone()),
        );
        insert_label_target(&mut targets, node_id, base_target.clone());
        insert_label_target(&mut targets, &node_name, base_target);

        let name_target = diagnostic_target_value(
            "node",
            Some(node_id.to_string()),
            None,
            Some("name".to_string()),
            Some(format!("{}.name", node_name)),
        );
        insert_label_target(
            &mut targets,
            &format!("{node_id}.name"),
            name_target.clone(),
        );
        insert_label_target(&mut targets, &format!("{}.name", node_name), name_target);

        let Some(config) = node.get("config").and_then(Value::as_object) else {
            continue;
        };
        for field in config.keys() {
            let field_target = diagnostic_target_value(
                "node",
                Some(node_id.to_string()),
                None,
                Some(field.clone()),
                Some(format!("{}.{}", node_name, field)),
            );
            insert_label_target(&mut targets, field, field_target.clone());
            insert_label_target(
                &mut targets,
                &format!("{node_id}.{field}"),
                field_target.clone(),
            );
            insert_label_target(
                &mut targets,
                &format!("{}.{}", node_name, field),
                field_target,
            );
        }
    }

    targets
}

pub(crate) fn build_quantscript_runtime_targets(graph: &Value) -> Value {
    let nodes = graph
        .get("nodes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut source_to_node = serde_json::Map::new();
    let mut runtime_node_id = Value::Null;
    let mut execution_node_id = Value::Null;

    for node in nodes {
        let Some(node_id) = node.get("id").and_then(Value::as_str) else {
            continue;
        };
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or_default();
        let sanitized = sanitize_quantscript_runtime_id(node_id);
        match node_type {
            "data" => {
                source_to_node.insert(
                    format!("data_{sanitized}"),
                    Value::String(node_id.to_string()),
                );
            }
            "intent" => {
                source_to_node.insert(
                    format!("intent_{sanitized}"),
                    Value::String(node_id.to_string()),
                );
            }
            "agent" => {
                source_to_node.insert(
                    "agent_script_main".to_string(),
                    Value::String(node_id.to_string()),
                );
            }
            "risk" => {
                source_to_node.insert(
                    "risk_script_global".to_string(),
                    Value::String(node_id.to_string()),
                );
            }
            "runtime" => runtime_node_id = Value::String(node_id.to_string()),
            "execution" => execution_node_id = Value::String(node_id.to_string()),
            _ => {}
        }
    }

    serde_json::json!({
        "source_to_node": source_to_node,
        "runtime_node_id": runtime_node_id,
        "execution_node_id": execution_node_id
    })
}

pub(crate) fn build_compile_runtime_targets_from_graph(graph: &Value) -> CompileRuntimeTargets {
    let targets_value = build_quantscript_runtime_targets(graph);
    serde_json::from_value(targets_value).unwrap_or_default()
}

fn sanitize_quantscript_runtime_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn insert_label_target(targets: &mut serde_json::Map<String, Value>, label: &str, target: Value) {
    if label.is_empty() || targets.contains_key(label) {
        return;
    }
    targets.insert(label.to_string(), target);
}

fn diagnostic_target_value(
    scope: &str,
    node_id: Option<String>,
    edge_id: Option<String>,
    field: Option<String>,
    label: Option<String>,
) -> Value {
    serde_json::json!({
        "scope": scope,
        "node_id": node_id,
        "edge_id": edge_id,
        "field": field,
        "label": label
    })
}

fn quoted(input: &str) -> String {
    serde_json::to_string(input).unwrap_or_else(|_| format!("\"{}\"", input))
}

fn render_json_scalar(value: &Value) -> String {
    match value {
        Value::String(value) => quoted(value),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Null => "null".to_string(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()),
    }
}

pub(super) fn parse_graph_quantscript_source(source: &str) -> anyhow::Result<Value> {
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

    let now = current_time_ms();
    let mut graph = serde_json::json!({
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
    });
    attach_quantscript_artifacts(&mut graph, source, now, std::path::Path::new(""));
    Ok(graph)
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
