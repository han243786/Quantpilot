use super::*;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// v1.0.3: 限制同时编译数为 4, 防止 CPU 密集操作撑爆线程池
static COMPILE_SEMAPHORE: std::sync::LazyLock<Arc<Semaphore>> =
    std::sync::LazyLock::new(|| Arc::new(Semaphore::new(4)));

/// QS 管道编译: graph JSON → QS 源码 → parse → lower → RuntimeProtocolCoreConfig (§1.1, §1.3)
pub(super) fn compile_runtime_protocol_via_qs(
    graph_json: &Value,
) -> Result<RuntimeProtocolCoreConfig, (StatusCode, String)> {
    let qs_source = generate_quantscript_from_graph_value(graph_json)
        .map_err(|e| json_bad_request("qs_generation_failed", format!("从图生成 QS 源码失败: {:#}", e)))?;
    let graph_value = parse_graph_quantscript_source(&qs_source)
        .map_err(|e| json_bad_request("qs_parse_failed", format!("QS 解析失败: {:#}", e)))?;
    let script_module = convert_graph_json_to_script_module(&graph_value)
        .map_err(|e| json_bad_request("qs_conversion_failed", format!("QS 模块转换失败: {:#}", e)))?;
    quantscript::lower_script_to_runtime_config(&script_module)
        .map_err(|e| json_bad_request("qs_lowering_failed", format!("QS 下层转换失败: {:#}", e)))
}

pub(super) fn register_compile_routes(router: Router<AppState>) -> Router<AppState> {
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
    let _permit = COMPILE_SEMAPHORE.acquire().await.map_err(|_| {
        (StatusCode::SERVICE_UNAVAILABLE, "编译服务已关闭".to_string())
    })?;
    // 空 intent 保护: 策略必须包含至少一个意图
    if request.runtime_config.intent_generators.is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "策略必须包含至少一个意图。请从左侧面板拖入一个意图节点 (如「双均线」) 并连线",
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

    // QS 管道是唯一编译路径 (§1.1, §1.3)
    let qs_protocol = compile_runtime_protocol_via_qs(&request.graph_json)?;

    let runtime_protocol_counts = (qs_protocol.data_sources.len(), qs_protocol.intents.len(), qs_protocol.agents.len(), qs_protocol.risks.len());
    let runtime_targets = build_compile_runtime_targets_from_graph(&request.graph_json);
    let compiled = compile_runtime_protocol_config(&qs_protocol).map_err(internal_error)?;
    let artifacts = build_compile_artifact_bundle(
        &request.runtime_config.metadata.graph_id,
        &request.runtime_config.metadata.compile_id,
        &request.runtime_config.metadata.name,
        &request.runtime_config.metadata.mode,
        StrategyArtifactSourceKind::FrontendGraph,
        &request.runtime_config.metadata.graph_id,
        BTreeMap::new(),
        &compiled,
    )
    .map_err(internal_error)?;
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
    let protocol_name = compiled.protocol_name.clone();
    let config_hash = compiled.config_hash.clone();
    let core_ir = compiled.core_ir.clone();

    Ok(Json(CompileRuntimeResponse {
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
    }))
}

async fn compile_strategy_ir_request(
    Json(request): Json<CompileStrategyIrRequest>,
) -> Result<Json<CompileStrategyIrResponse>, (StatusCode, String)> {
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

async fn compile_formal_quantscript_request(
    Json(request): Json<CompileFormalQuantScriptRequest>,
) -> Result<Json<CompileRuntimeResponse>, (StatusCode, String)> {
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
            if diagnostic.code != "QPQSLOW999" {
                return json_bad_request_with_details_and_partial(
                    "quantscript_lowering_failed",
                    "形式化 QuantScript 下层转换失败",
                    vec![api_error_detail_from_compile_diagnostic(&diagnostic)],
                    partial_authoring_view.clone(),
                );
            }
            json_bad_request_with_partial(
                "quantscript_lowering_failed",
                format!("形式化 QuantScript 下层转换失败: {message}"),
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
