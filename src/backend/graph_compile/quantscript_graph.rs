use crate::{
    current_time_ms, internal_error, json_bad_request, not_found_io_error, validate_graph_id,
    AppState, CompileRuntimeTargets, ParseGraphQuantScriptRequest,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use tokio::fs;

mod formal_module_conversion;
mod graph_to_qs_generation;
pub(crate) use formal_module_conversion::convert_graph_json_to_script_module;
pub(crate) use graph_to_qs_generation::generate_quantscript_from_graph_value;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
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

async fn load_graph_quantscript(
    State(state): State<AppState>,
    Path(graph_id): Path<String>,
) -> Result<String, (StatusCode, String)> {
    validate_graph_id(&graph_id).map_err(internal_error)?;
    let source_path = state.graph_store_dir.join(format!("{}.qs", graph_id));
    fs::read_to_string(&source_path)
        .await
        .map_err(not_found_io_error)
}

async fn parse_graph_quantscript(
    Json(request): Json<ParseGraphQuantScriptRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    parse_graph_quantscript_source(&request.source)
        .map(Json)
        .map_err(|error| {
            json_bad_request(
                "bad_request",
                format!("strategy_graph QuantScript 解析失败: {error:#}"),
            )
        })
}

pub(crate) fn attach_quantscript_artifacts(
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
            let source =
                graph_to_qs_generation::generate_node_quantscript(node, &nodes, &edges).ok()?;
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
                if let Some(script_source_id) = script_data_source_id_from_graph_node(&node) {
                    source_to_node.insert(script_source_id, Value::String(node_id.to_string()));
                }
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
            "runtime" | "runtime_control" => runtime_node_id = Value::String(node_id.to_string()),
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
    // v2.4.0 G1: 反序列化失败时记录警告而非静默降级为空结构体
    match serde_json::from_value(targets_value) {
        Ok(targets) => targets,
        Err(e) => {
            safe_eprintln!(
                "[runtime_targets] 反序列化 CompileRuntimeTargets 失败: {}, 降级为空映射",
                e
            );
            CompileRuntimeTargets::default()
        }
    }
}

fn script_data_source_id_from_graph_node(node: &Value) -> Option<String> {
    let config = node.get("config")?;
    let exchange = config
        .get("exchange")
        .and_then(Value::as_str)
        .unwrap_or("binance");
    let instrument = config
        .get("instrument")
        .and_then(Value::as_str)
        .unwrap_or("BTCUSDT");
    let interval = config
        .get("timeframe")
        .or_else(|| config.get("interval"))
        .and_then(Value::as_str)
        .unwrap_or("1d");

    Some(format!(
        "script_{}_{}_{}",
        sanitize_quantscript_source_segment(exchange),
        sanitize_quantscript_source_segment(instrument),
        sanitize_quantscript_source_segment(interval)
    ))
}

fn sanitize_quantscript_source_segment(value: &str) -> String {
    let sanitized = value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
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

pub(crate) fn parse_graph_quantscript_source(source: &str) -> anyhow::Result<Value> {
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
