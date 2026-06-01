use crate::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// v1.0.3: 限制同时编译数为 4, 防止 CPU 密集操作撑爆线程池
static COMPILE_SEMAPHORE: std::sync::LazyLock<Arc<Semaphore>> =
    std::sync::LazyLock::new(|| Arc::new(Semaphore::new(4)));

// v3.5.0: 编译产物缓存 — 同图同参数跳过QS管道
const COMPILE_CACHE_MAX: usize = 50;
static COMPILE_CACHE: std::sync::LazyLock<std::sync::Mutex<HashMap<String, CompileCacheEntry>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

#[derive(Clone)]
struct CompileCacheEntry {
    response: CompileRuntimeResponse,
    inserted_at_ms: u64,
}

fn compute_compile_cache_key(graph_json: &Value, runtime_config: &FrontendRuntimeConfig) -> String {
    use ring::digest::{digest, SHA256};
    let cache_payload = serde_json::json!({
        "graph_json": graph_json,
        "runtime_config": runtime_config,
    });
    let graph_bytes = serde_json::to_vec(&cache_payload).unwrap_or_default();
    let hash = digest(&SHA256, &graph_bytes);
    hash.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
}

/// QS 管道编译: graph JSON → QS 源码 → parse → lower → RuntimeProtocolCoreConfig (§1.1, §1.3)
pub(crate) fn compile_runtime_protocol_via_qs(
    graph_json: &Value,
) -> Result<RuntimeProtocolCoreConfig, (StatusCode, String)> {
    let qs_source = generate_quantscript_from_graph_value(graph_json).map_err(|e| {
        json_bad_request_with_code(
            "qs_generation_failed",
            crate::error_codes::ERR_QS_GENERATION_FAILED,
            format!("从图生成 QS 源码失败: {}", e),
        )
    })?;
    let graph_value = parse_graph_quantscript_source(&qs_source).map_err(|e| {
        json_bad_request_with_code(
            "qs_parse_failed",
            crate::error_codes::ERR_QS_PARSE_FAILED,
            format!("QS 解析失败: {}", e),
        )
    })?;
    let script_module = convert_graph_json_to_script_module(&graph_value).map_err(|e| {
        json_bad_request_with_code(
            "qs_conversion_failed",
            crate::error_codes::ERR_QS_LOWER_FAILED,
            format!("QS 模块转换失败: {}", e),
        )
    })?;
    quantscript::lower_script_to_runtime_config(&script_module).map_err(|e| {
        json_bad_request_with_code(
            "qs_lowering_failed",
            crate::error_codes::ERR_QS_LOWER_FAILED,
            format!("QS 下层转换失败: {}", e),
        )
    })
}

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/runtime/compile", post(compile_runtime_request))
        .route(
            "/api/strategy-ir/compile",
            post(compile_strategy_ir_request),
        )
        .route(
            "/api/quantscript/formal/compile",
            post(compile_formal_quantscript_request),
        )
}
async fn compile_runtime_request(
    Json(request): Json<CompileRuntimeRequest>,
) -> Result<Json<CompileRuntimeResponse>, (StatusCode, String)> {
    let graph_json = graph_json_from_runtime_config(&request.runtime_config);
    // v3.5.0: 编译缓存 — 同图同参数跳过整个QS管道
    let cache_key = compute_compile_cache_key(&graph_json, &request.runtime_config);
    {
        let cache = COMPILE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = cache.get(&cache_key) {
            return Ok(Json(entry.response.clone()));
        }
    }

    // v1.1.10: 编译信号量超时保护，防止请求永久挂起
    let _permit = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        COMPILE_SEMAPHORE.acquire(),
    )
    .await
    .map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            serde_json::json!({
                "error": "service_unavailable",
                "error_code": crate::error_codes::ERR_COMPILE_BUSY,
                "message": "编译服务繁忙，请稍后重试"
            })
            .to_string(),
        )
    })?
    .map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            serde_json::json!({
                "error": "service_unavailable",
                "error_code": crate::error_codes::ERR_COMPILE_BUSY,
                "message": "编译服务已关闭"
            })
            .to_string(),
        )
    })?;
    // 空 intent 保护: 策略必须包含至少一个意图
    if request.runtime_config.intent_generators.is_empty() {
        return Err(json_bad_request_with_code(
            "bad_request",
            crate::error_codes::ERR_QSC_EMPTY_INTENT,
            "策略必须包含至少一个意图。请从左侧面板拖入一个意图节点 (如「双均线」) 并连线",
        ));
    }
    // v3.7.x: 节点上限早期检查 (编译前拒绝, 避免浪费CPU)
    const MAX_COMPILE_NODES: usize = 500;
    let node_count = graph_json
        .get("nodes")
        .and_then(|n| n.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if node_count > MAX_COMPILE_NODES {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "策略图节点数 ({}) 超过上限 ({})",
                node_count, MAX_COMPILE_NODES
            ),
        ));
    }
    validate_runtime_config_capabilities(&request.runtime_config).map_err(|details| {
        json_bad_request_with_details(
            "capability_gated",
            "运行时配置使用了当前 Beta 版本未启用的能力。请检查所有节点的 module_key 是否在 /api/capabilities 白名单中",
            details,
        )
    })?;
    let contract_diagnostics =
        collect_runtime_compile_contract_diagnostics(&request.runtime_config);
    if !contract_diagnostics.is_empty() {
        return Err(json_bad_request_with_details(
            "runtime_compile_failed",
            "运行时图编译合约校验失败。请检查所有节点已正确连线且 graph_id 不含非法字符",
            contract_diagnostics
                .iter()
                .map(api_error_detail_from_compile_diagnostic)
                .collect(),
        ));
    }

    // v2.1.3: CPU密集编译移至 spawn_blocking，不阻塞 tokio runtime (P2-8)
    let (qs_protocol, runtime_targets, compiled, artifacts) = {
        let graph_json = graph_json.clone();
        let metadata = request.runtime_config.metadata.clone();
        let request_graph_json = graph_json.clone();
        let join_result =
            tokio::task::spawn_blocking(move || -> Result<_, (StatusCode, String)> {
                // QS 管道是唯一编译路径 (§1.1, §1.3)
                let qs = compile_runtime_protocol_via_qs(&graph_json)?;
                let targets = build_compile_runtime_targets_from_graph(&request_graph_json);
                let comp = compile_runtime_protocol_config(&qs).map_err(internal_error)?;
                let arts = build_compile_artifact_bundle(
                    &metadata.graph_id,
                    &metadata.compile_id,
                    &metadata.name,
                    &metadata.mode,
                    StrategyArtifactSourceKind::FrontendGraph,
                    &metadata.graph_id,
                    BTreeMap::new(),
                    &comp,
                )
                .map_err(internal_error)?;
                Ok((qs, targets, comp, arts))
            })
            .await;
        match join_result {
            Ok(Ok(data)) => data,
            Ok(Err(e)) => return Err(e),
            // v2.4.0 G4: 记录 panic payload, 否则只能看到 "编译任务被取消"
            Err(join_err) => {
                let panic_msg = join_err
                    .try_into_panic()
                    .map(|payload| {
                        payload
                            .downcast_ref::<String>()
                            .map(|s| s.as_str())
                            .or_else(|| payload.downcast_ref::<&str>().copied())
                            .unwrap_or("未知 panic")
                            .to_string()
                    })
                    .unwrap_or_else(|_| "非 panic 导致的 JoinError".to_string());
                safe_eprintln!("[compile] 编译任务 panic: {}", panic_msg);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "编译任务被取消".to_string(),
                ));
            }
        }
    };

    // v2.0.1: graph→QS→graph 往返完整性检查
    let edge_count = graph_json
        .get("edges")
        .and_then(|e| e.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let runtime_protocol_counts = (
        qs_protocol.data_sources.len(),
        qs_protocol.intents.len(),
        qs_protocol.agents.len(),
        qs_protocol.risks.len(),
    );
    let mut diagnostics = collect_compile_diagnostics(&request.runtime_config);
    diagnostics.push(CompileDiagnostic {
        code: "QSPIPELINE".to_string(),
        severity: CompileDiagnosticSeverity::Warning,
        message: format!(
            "QS 管道编译通过: {} 个数据源, {} 个意图, {} 个代理, {} 个风控",
            qs_protocol.data_sources.len(),
            qs_protocol.intents.len(),
            qs_protocol.agents.len(),
            qs_protocol.risks.len(),
        ),
        target: None,
        span_label: None,
        hint: None,
    });
    // v2.0.1: graph→QS→graph 往返节点/边计数完整性诊断
    let intent_count = qs_protocol.intents.len();
    if intent_count == 0 && node_count > 0 {
        diagnostics.push(CompileDiagnostic {
            code: "QSPIPELINE".to_string(),
            severity: CompileDiagnosticSeverity::Warning,
            message: format!(
                "graph→QS 往返: {} 个节点, {} 条边, 但编译后生成 0 个意图。可能部分模块键未被 QS 生成器识别。",
                node_count, edge_count
            ),
            target: None,
            span_label: None,
            hint: Some(
                "检查前端模块键是否与 backend.graph_compile.quantscript_graph 的 QS 生成分支一致。"
                    .to_string(),
            ),
        });
    }
    let protocol_name = compiled.protocol_name.clone();
    let config_hash = compiled.config_hash.clone();
    let core_ir = compiled.core_ir.clone();

    safe_eprintln!(
        "[audit] 编译完成 — graph={} compile={} protocol={} intents={} agents={}",
        request.runtime_config.metadata.graph_id,
        request.runtime_config.metadata.compile_id,
        protocol_name,
        runtime_protocol_counts.1,
        runtime_protocol_counts.2
    );

    let response = CompileRuntimeResponse {
        graph_id: request.runtime_config.metadata.graph_id.clone(),
        compile_id: request.runtime_config.metadata.compile_id.clone(),
        compilable: true,
        protocol_name,
        config_hash,
        core_ir,
        artifacts,
        counts: CompileCounts {
            data_sources: runtime_protocol_counts.0,
            intent_generators: runtime_protocol_counts.1,
            agents: runtime_protocol_counts.2,
            risk_controls: runtime_protocol_counts.3,
            executions: request.runtime_config.executions.len(),
        },
        diagnostics,
        runtime_config: request.runtime_config.clone(),
        runtime_targets,
    };

    // v3.5.1: 缓存编译产物 (FIFO淘汰, 双检锁防重复编译)
    {
        let mut cache = COMPILE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        // 双检: 编译期间其他线程可能已插入相同key, 避免重复写入
        if cache.contains_key(&cache_key) {
            return Ok(Json(response));
        }
        // FIFO淘汰: 超过上限时移除最旧的半数条目 (不drain全量防panic丢缓存)
        if cache.len() >= COMPILE_CACHE_MAX {
            let remove_count = cache.len() - (COMPILE_CACHE_MAX / 2).max(1);
            let mut oldest: Vec<_> = cache
                .iter()
                .map(|(k, v)| (v.inserted_at_ms, k.clone()))
                .collect();
            oldest.sort_by_key(|(ts, _)| *ts);
            for (_, key) in oldest.iter().take(remove_count) {
                cache.remove(key);
            }
        }
        cache.insert(
            cache_key,
            CompileCacheEntry {
                response: response.clone(),
                inserted_at_ms: current_time_ms(),
            },
        );
    }

    Ok(Json(response))
}

fn graph_json_from_runtime_config(runtime_config: &FrontendRuntimeConfig) -> Value {
    let runtime_value = serde_json::to_value(runtime_config).unwrap_or_else(|_| Value::Null);
    let mut nodes = Vec::<Value>::new();
    let mut edges = Vec::<Value>::new();

    for node in runtime_value
        .get("data_sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        nodes.push(serde_json::json!({
            "id": node["id"].clone(),
            "type": "data",
            "module_key": node["module_key"].clone(),
            "name": node["name"].clone(),
            "config": node["config"].clone(),
        }));
    }

    for node in runtime_value
        .get("intent_generators")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        nodes.push(serde_json::json!({
            "id": node["id"].clone(),
            "type": "intent",
            "module_key": node["module_key"].clone(),
            "name": node["name"].clone(),
            "config": node["config"].clone(),
        }));
        for input_ref in node
            .get("input_refs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            edges.push(serde_json::json!({
                "source_node_id": input_ref["source_id"].clone(),
                "source_port": input_ref["source_port"].clone(),
                "target_node_id": node["id"].clone(),
                "target_port": input_ref["target_port"].clone(),
            }));
        }
    }

    for node in runtime_value
        .get("agents")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        nodes.push(serde_json::json!({
            "id": node["id"].clone(),
            "type": "agent",
            "module_key": node["module_key"].clone(),
            "name": node["name"].clone(),
            "config": node["config"].clone(),
        }));
        for intent_ref in node
            .get("intent_refs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            edges.push(serde_json::json!({
                "source_node_id": intent_ref.clone(),
                "source_port": "intent_out",
                "target_node_id": node["id"].clone(),
                "target_port": "intent_input",
            }));
        }
    }

    for node in runtime_value
        .get("risk_controls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        nodes.push(serde_json::json!({
            "id": node["id"].clone(),
            "type": "risk",
            "module_key": node["module_key"].clone(),
            "name": node["name"].clone(),
            "config": node["config"].clone(),
        }));
        for agent_ref in node
            .get("agent_refs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            edges.push(serde_json::json!({
                "source_node_id": agent_ref.clone(),
                "source_port": "agent_out",
                "target_node_id": node["id"].clone(),
                "target_port": "agent_input",
            }));
        }
    }

    for node in runtime_value
        .get("executions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        nodes.push(serde_json::json!({
            "id": node["id"].clone(),
            "type": "execution",
            "module_key": node["module_key"].clone(),
            "name": node["name"].clone(),
            "config": node["config"].clone(),
        }));
        if !node.get("risk_ref").unwrap_or(&Value::Null).is_null() {
            edges.push(serde_json::json!({
                "source_node_id": node["risk_ref"].clone(),
                "source_port": "risk_out",
                "target_node_id": node["id"].clone(),
                "target_port": "risk_input",
            }));
        }
    }

    if let Some(node) = runtime_value
        .get("runtime_control")
        .filter(|node| !node.is_null())
    {
        nodes.push(serde_json::json!({
            "id": node["id"].clone(),
            "type": "runtime",
            "module_key": node["module_key"].clone(),
            "name": node["name"].clone(),
            "config": node["config"].clone(),
        }));
        if let Some(execution_node) = runtime_value
            .get("executions")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
        {
            edges.push(serde_json::json!({
                "source_node_id": execution_node["id"].clone(),
                "source_port": "execution_out",
                "target_node_id": node["id"].clone(),
                "target_port": "execution_input",
            }));
        }
    }

    serde_json::json!({
        "metadata": {
            "graph_id": runtime_config.metadata.graph_id.clone(),
            "name": runtime_config.metadata.name.clone(),
            "version": runtime_config.metadata.version.clone(),
        },
        "nodes": nodes,
        "edges": edges,
    })
}

async fn compile_strategy_ir_request(
    Json(request): Json<CompileStrategyIrRequest>,
) -> Result<Json<CompileStrategyIrResponse>, (StatusCode, String)> {
    // v2.1.1: 统一使用编译信号量限流 (P2-7)
    let _permit = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        COMPILE_SEMAPHORE.acquire(),
    )
    .await
    .map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            serde_json::json!({
                "error": "service_unavailable",
                "error_code": crate::error_codes::ERR_COMPILE_BUSY,
                "message": "编译服务繁忙，请稍后重试"
            })
            .to_string(),
        )
    })?
    .map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            serde_json::json!({
                "error": "service_unavailable",
                "error_code": crate::error_codes::ERR_COMPILE_BUSY,
                "message": "编译服务已关闭"
            })
            .to_string(),
        )
    })?;
    let strategy_ir =
        serde_json::from_value::<StrategyIr>(request.strategy_ir).map_err(|error| {
            json_bad_request_with_details(
                "strategy_ir_compile_failed",
                "无效的 Strategy IR JSON 负载",
                vec![ApiErrorDetail {
                    code: "QPSTRATJSON001".to_string(),
                    target: Some("strategy_ir".to_string()),
                    message: error.to_string(),
                    span_label: None,
                    reason: None,
                }],
            )
        })?;

    if let Err(validation_error) = strategy_ir.validate() {
        let diagnostics = validation_error
            .errors
            .iter()
            .map(|message| strategy_ir_diagnostic_from_validation_message(message))
            .collect::<Vec<_>>();
        return Err(json_bad_request_with_details(
            "strategy_ir_compile_failed",
            "Strategy IR 验证失败",
            diagnostics
                .iter()
                .map(api_error_detail_from_compile_diagnostic)
                .collect(),
        ));
    }

    let core_ir = lower_strategy_ir_to_core_ir(&strategy_ir).map_err(|error| {
        let diagnostics = vec![strategy_ir_diagnostic_from_lowering_error(
            &error.to_string(),
        )];
        json_bad_request_with_details(
            "strategy_ir_compile_failed",
            "Strategy IR 下层转换失败",
            diagnostics
                .iter()
                .map(api_error_detail_from_compile_diagnostic)
                .collect(),
        )
    })?;

    Ok(Json(CompileStrategyIrResponse {
        graph_id: request.graph_id,
        compile_id: request.compile_id,
        compilable: true,
        core_ir,
        diagnostics: Vec::new(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn runtime_config(mode: &str) -> FrontendRuntimeConfig {
        FrontendRuntimeConfig {
            metadata: FrontendMetadata {
                graph_id: "graph_cache".to_string(),
                compile_id: "compile_cache".to_string(),
                name: "Cache".to_string(),
                version: "1.0.0".to_string(),
                mode: mode.to_string(),
            },
            data_sources: vec![],
            intent_generators: vec![],
            agents: vec![],
            risk_controls: vec![],
            executions: vec![],
            runtime_control: None,
        }
    }

    #[test]
    fn compile_cache_key_includes_runtime_config() {
        let graph = json!({"metadata": {"graph_id": "same_graph"}});
        let paper_key = compute_compile_cache_key(&graph, &runtime_config("paper"));
        let live_key = compute_compile_cache_key(&graph, &runtime_config("live"));

        assert_ne!(paper_key, live_key);
    }
}

async fn compile_formal_quantscript_request(
    Json(request): Json<CompileFormalQuantScriptRequest>,
) -> Result<Json<CompileRuntimeResponse>, (StatusCode, String)> {
    // v2.1.1: 统一使用编译信号量限流 (P2-7)
    let _permit = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        COMPILE_SEMAPHORE.acquire(),
    )
    .await
    .map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            serde_json::json!({
                "error": "service_unavailable",
                "message": "编译服务繁忙，请稍后重试"
            })
            .to_string(),
        )
    })?
    .map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            serde_json::json!({
                "error": "service_unavailable",
                "message": "编译服务已关闭"
            })
            .to_string(),
        )
    })?;
    let module = parse_formal_quant_script_module(&request.source).map_err(internal_error)?;
    let resolved = lower_formal_script_to_typed_hir(&module);
    let analysis = analyze_formal_script_module(&module, &resolved);
    let lowering_context = FormalLoweringContext {
        universe_snapshot: request.universe_snapshot.clone(),
    };
    let partial_authoring_view =
        build_quantscript_authoring_view(&request.source, &module, &resolved, &lowering_context)
            .ok();

    let mut diagnostics = resolved
        .diagnostics
        .iter()
        .map(compile_diagnostic_from_script_diagnostic)
        .collect::<Vec<_>>();
    diagnostics.extend(
        analysis
            .diagnostics
            .iter()
            .map(compile_diagnostic_from_script_diagnostic),
    );

    let mut error_details = resolved
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == quantscript::DiagnosticSeverity::Error)
        .map(api_error_detail_from_script_diagnostic)
        .collect::<Vec<_>>();
    error_details.extend(
        analysis
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == quantscript::DiagnosticSeverity::Error)
            .map(api_error_detail_from_script_diagnostic),
    );

    if !error_details.is_empty() {
        return Err(json_bad_request_with_details_and_partial(
            "quantscript_compile_failed",
            "形式化 QuantScript 语义分析失败",
            error_details,
            partial_authoring_view.clone(),
        ));
    }

    let lowering_error_details = collect_formal_quantscript_pre_lowering_diagnostics(&module)
        .into_iter()
        .map(|diagnostic| api_error_detail_from_compile_diagnostic(&diagnostic))
        .collect::<Vec<_>>();
    if !lowering_error_details.is_empty() {
        return Err(json_bad_request_with_details_and_partial(
            "quantscript_lowering_failed",
            "形式化 QuantScript 下层转换失败",
            lowering_error_details,
            partial_authoring_view.clone(),
        ));
    }
    let runtime_config = lower_formal_script_to_runtime_config(&module, &lowering_context)
        .map_err(|error| {
            let message = format!("{error:#}");
            let diagnostic = formal_quantscript_diagnostic_from_lowering_error(&message);
            json_bad_request_with_details_and_partial(
                "quantscript_lowering_failed",
                if diagnostic.code == "QPQSLOW999" {
                    "QuantScript 编译失败：遇到未预期的内部错误，请检查策略语法或联系支持"
                } else {
                    "QuantScript 编译失败"
                },
                vec![api_error_detail_from_compile_diagnostic(&diagnostic)],
                partial_authoring_view.clone(),
            )
        })?;
    let compiled = compile_runtime_protocol_config_with_metadata(
        &runtime_config,
        CoreMetadata {
            strategy_id: request.graph_id.clone(),
            name: request.runtime_template.metadata.name.clone(),
            source_kind: CoreSourceKind::FormalQuantScript,
        },
    )
    .map_err(internal_error)?;
    let artifacts = build_compile_artifact_bundle(
        &request.graph_id,
        &request.compile_id,
        &request.runtime_template.metadata.name,
        &request.runtime_template.metadata.mode,
        StrategyArtifactSourceKind::FormalQuantScript,
        &request.graph_id,
        build_formal_quantscript_strategy_metadata(
            &request.source,
            &module,
            &resolved,
            &lowering_context,
        )
        .map_err(internal_error)?,
        &compiled,
    )
    .map_err(internal_error)?;
    let protocol_name = compiled.protocol_name.clone();
    let config_hash = compiled.config_hash.clone();
    let core_ir = compiled.core_ir.clone();
    let frontend_runtime_config = frontend_runtime_config_from_core_with_template(
        &runtime_config,
        &request.runtime_template,
        &request.runtime_targets,
        &request.graph_id,
        &request.compile_id,
    );

    Ok(Json(CompileRuntimeResponse {
        graph_id: request.graph_id,
        compile_id: request.compile_id,
        compilable: true,
        protocol_name,
        config_hash,
        core_ir,
        artifacts,
        counts: CompileCounts {
            data_sources: runtime_config.data_sources.len(),
            intent_generators: runtime_config.intents.len(),
            agents: runtime_config.agents.len(),
            risk_controls: runtime_config.risks.len(),
            executions: 1,
        },
        diagnostics,
        runtime_config: frontend_runtime_config,
        runtime_targets: request.runtime_targets,
    }))
}
