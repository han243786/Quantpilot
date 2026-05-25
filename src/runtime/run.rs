#[derive(Debug, Deserialize)]
struct V4RuntimeRunRequest {
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    graph: Option<qrpc_core_ir::v4::V4MachineGraphContract>,
    #[serde(default)]
    initial_event: Option<qrpc_runtime::V4RuntimeInputEvent>,
}

#[derive(Debug, Serialize)]
struct V4RuntimeRunDiagnostic {
    severity: String,
    code: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct V4RuntimeRunResponse {
    run_id: String,
    graph_id: String,
    event_count: usize,
    output: qrpc_runtime::V4PaperSimulatedRunOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    handoff: Option<V4RuntimeRunHandoff>,
    diagnostics: Vec<V4RuntimeRunDiagnostic>,
}

#[derive(Debug, Serialize)]
struct V4RuntimeRunHandoff {
    schema_version: String,
    accepted_for_runtime_handoff: bool,
    graph_id: Option<String>,
    venue_id: Option<String>,
    runtime_mode: Option<qrpc_core_ir::v4::RuntimeTradingMode>,
    paper_simulated_start_allowed: bool,
    provider_order_submission_attached: bool,
    runtime_attached: bool,
    lowering_attached: bool,
    diagnostics: Vec<String>,
}

async fn start_v4_runtime_run(
    State(state): State<AppState>,
    Json(request): Json<V4RuntimeRunRequest>,
) -> Result<Json<V4RuntimeRunResponse>, (StatusCode, String)> {
    if state
        .run_in_progress
        .swap(true, std::sync::atomic::Ordering::AcqRel)
    {
        return Err((
            StatusCode::CONFLICT,
            serde_json::to_string(&json!({
                "error": "runtime_busy",
                "error_code": crate::error_codes::ERR_QSC_CAPABILITY_GATED,
                "message": "runtime already has an active run; stop or wait before starting v4 simulation",
                "details": []
            }))
            .unwrap(),
        ));
    }
    let _run_guard = RunInProgressGuard(&state.run_in_progress);

    let now_ms = current_time_ms();
    let (graph, handoff, diagnostics, initial_event_override) =
        resolve_v4_runtime_run_graph(request)?;
    let graph_id = graph.graph_id.clone();
    let initial_event =
        initial_event_override.unwrap_or(handoff_initial_event(handoff.as_ref(), &graph, now_ms)?);

    let mut runtime = qrpc_runtime::V4PaperSimulatedRuntime::new_with_execution_capabilities(
        graph,
        runtime_simulated_v4_matrix("paper-local"),
        vec![qrpc_core_ir::v4::ExecutionCapabilityKind::Market],
    )
    .map_err(internal_error)?;
    let output = runtime.submit_event(initial_event).map_err(internal_error)?;

    Ok(Json(V4RuntimeRunResponse {
        run_id: format!("v4_run_{}", now_ms),
        graph_id,
        event_count: output.events.len(),
        output,
        handoff: handoff.as_ref().map(v4_runtime_handoff_response),
        diagnostics,
    }))
}

fn resolve_v4_runtime_run_graph(
    request: V4RuntimeRunRequest,
) -> Result<
    (
        qrpc_core_ir::v4::V4MachineGraphContract,
        Option<quantscript::V4QsRuntimeHandoffReport>,
        Vec<V4RuntimeRunDiagnostic>,
        Option<qrpc_runtime::V4RuntimeInputEvent>,
    ),
    (StatusCode, String),
> {
    let initial_event = request.initial_event;
    if let Some(graph) = request.graph {
        graph.validate_static_contract().map_err(|errors| {
            json_bad_request_with_code(
                "v4_graph_invalid",
                crate::error_codes::ERR_QSC_CONTRACT_INVALID,
                format!("v4 machine graph failed static validation: {}", errors.join("; ")),
            )
        })?;
        return Ok((graph, None, Vec::new(), initial_event));
    }

    let Some(source) = request
        .source
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Err(json_bad_request_with_code(
            "v4_source_missing",
            crate::error_codes::ERR_QSC_EMPTY_INTENT,
            "v4 runtime run requires `source` or `graph`",
        ));
    };

    let audit = quantscript::audit_v4_quant_script_static(&source, &runtime_v4_static_bundle());
    let diagnostics = audit
        .diagnostics
        .iter()
        .map(|diagnostic| V4RuntimeRunDiagnostic {
            severity: format!("{:?}", diagnostic.severity).to_ascii_lowercase(),
            code: diagnostic.code.to_string(),
            message: diagnostic.message.clone(),
        })
        .collect::<Vec<_>>();
    let handoff = quantscript::build_v4_qs_runtime_handoff(&audit);
    if !handoff.accepted_for_runtime_handoff {
        return Err(json_bad_request_with_code(
            "v4_runtime_handoff_rejected",
            crate::error_codes::ERR_QSC_CONTRACT_INVALID,
            format!(
                "v4 QS runtime handoff rejected: {}",
                handoff.diagnostics.join("; ")
            ),
        ));
    }
    let graph = audit.parsed_graph.ok_or_else(|| {
        json_bad_request_with_code(
            "v4_graph_missing",
            crate::error_codes::ERR_QSC_CONTRACT_INVALID,
            "v4 QS static audit did not produce a machine graph",
        )
    })?;

    Ok((graph, Some(handoff), diagnostics, initial_event))
}

fn handoff_initial_event(
    _handoff: Option<&quantscript::V4QsRuntimeHandoffReport>,
    graph: &qrpc_core_ir::v4::V4MachineGraphContract,
    ts_ms: u64,
) -> Result<qrpc_runtime::V4RuntimeInputEvent, (StatusCode, String)> {
    let spec = graph
        .event_catalog
        .as_ref()
        .and_then(|catalog| {
            catalog
                .events
                .iter()
                .find(|event| {
                    event.source_kind == qrpc_core_ir::v4::MachineEventSourceKind::Runtime
                })
                .or_else(|| catalog.events.first())
        })
        .ok_or_else(|| {
            json_bad_request_with_code(
                "v4_event_catalog_missing",
                crate::error_codes::ERR_QSC_CONTRACT_INVALID,
                "v4 runtime run requires at least one declared event in MachineEventCatalog",
            )
        })?;
    let mut payload = serde_json::Map::new();
    for field in &spec.payload_fields {
        payload.insert(
            field.name.clone(),
            default_v4_payload_value(field, graph.graph_id.as_str()),
        );
    }
    Ok(qrpc_runtime::V4RuntimeInputEvent {
        event_type: spec.event_type.clone(),
        source: "runtime".to_string(),
        payload: Value::Object(payload),
        ts_ms,
    })
}

fn v4_runtime_handoff_response(
    handoff: &quantscript::V4QsRuntimeHandoffReport,
) -> V4RuntimeRunHandoff {
    V4RuntimeRunHandoff {
        schema_version: handoff.schema_version.clone(),
        accepted_for_runtime_handoff: handoff.accepted_for_runtime_handoff,
        graph_id: handoff.graph_id.clone(),
        venue_id: handoff.venue_id.clone(),
        runtime_mode: handoff.runtime_mode,
        paper_simulated_start_allowed: handoff.paper_simulated_start_allowed,
        provider_order_submission_attached: handoff.provider_order_submission_attached,
        runtime_attached: handoff.runtime_attached,
        lowering_attached: handoff.lowering_attached,
        diagnostics: handoff.diagnostics.clone(),
    }
}

fn default_v4_payload_value(
    field: &qrpc_core_ir::v4::MachineEventPayloadField,
    graph_id: &str,
) -> Value {
    match field.type_name.trim().to_ascii_lowercase().as_str() {
        "string" | "symbol" | "venue" | "account" | "side" | "position_side" | "order_type"
        | "time_in_force" | "freshness" | "runtime_mode" | "order_permission" => {
            if field.name == "strategy_id" {
                Value::String(graph_id.to_string())
            } else {
                Value::String(field.name.clone())
            }
        }
        "bool" | "boolean" => Value::Bool(true),
        "u64" | "uint" => Value::Number(serde_json::Number::from(0_u64)),
        "i64" | "int" | "integer" => Value::Number(serde_json::Number::from(0_i64)),
        "f64" | "decimal" | "number" | "price" | "quantity" | "notional" | "percent"
        | "ratio" | "fee" | "slippage" | "leverage" => serde_json::Number::from_f64(0.0)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        "object" | "map" => json!({}),
        "array" | "list" => json!([]),
        _ if field.nullable => Value::Null,
        _ => Value::String(field.name.clone()),
    }
}

fn runtime_v4_static_bundle() -> qrpc_core_ir::v4::V4StaticContractBundle {
    qrpc_core_ir::v4::V4StaticContractBundle {
        venue_matrices: vec![runtime_simulated_v4_matrix("paper-local")],
        ..qrpc_core_ir::v4::V4StaticContractBundle::default()
    }
}

fn runtime_simulated_v4_matrix(venue_id: impl Into<String>) -> qrpc_core_ir::v4::VenueCapabilityMatrix {
    let mut matrix = qrpc_core_ir::v4::unsupported_v4_first_wave_matrix(venue_id);
    for entry in &mut matrix.capabilities {
        if matches!(
            entry.capability,
            qrpc_core_ir::v4::ExecutionCapabilityKind::Market
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Limit
                | qrpc_core_ir::v4::ExecutionCapabilityKind::StopMarket
                | qrpc_core_ir::v4::ExecutionCapabilityKind::StopLimit
                | qrpc_core_ir::v4::ExecutionCapabilityKind::TakeProfitMarket
                | qrpc_core_ir::v4::ExecutionCapabilityKind::TakeProfitLimit
                | qrpc_core_ir::v4::ExecutionCapabilityKind::OcoBracket
                | qrpc_core_ir::v4::ExecutionCapabilityKind::TrailingStop
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Gtc
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Ioc
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Fok
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Day
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Gtd
                | qrpc_core_ir::v4::ExecutionCapabilityKind::PostOnly
                | qrpc_core_ir::v4::ExecutionCapabilityKind::ReduceOnly
                | qrpc_core_ir::v4::ExecutionCapabilityKind::CloseOnly
                | qrpc_core_ir::v4::ExecutionCapabilityKind::ClientOrderId
                | qrpc_core_ir::v4::ExecutionCapabilityKind::OpenLong
                | qrpc_core_ir::v4::ExecutionCapabilityKind::CloseLong
                | qrpc_core_ir::v4::ExecutionCapabilityKind::OpenShort
                | qrpc_core_ir::v4::ExecutionCapabilityKind::CloseShort
                | qrpc_core_ir::v4::ExecutionCapabilityKind::CancelReplaceAmend
        ) {
            entry.source = qrpc_core_ir::v4::CapabilitySupportSource::RuntimeSimulated;
            entry.supported_modes = vec![qrpc_core_ir::v4::RuntimeTradingMode::PaperSimulated];
        }
    }
    matrix
}

async fn start_test_run(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<FrontendRunRequest>,
) -> Result<Json<RunStartResponse>, (StatusCode, String)> {
    // v1.0.3: 运行互斥 — 同一时间只允许一个 Paper 运行
    // v1.3.5: RAII guard 确保运行结束后自动复位
    if state.run_in_progress.swap(true, std::sync::atomic::Ordering::AcqRel) {
        return Err((StatusCode::CONFLICT, "已有运行在进行中, 请先停止当前运行后再启动新的运行".to_string()));
    }
    let _run_guard = RunInProgressGuard(&state.run_in_progress);
    validate_runtime_capability_guard(request.capability_context.as_ref()).map_err(|details| {
        json_bad_request_with_details(
            "capability_boundary_violation",
            "运行时写入需要当前的能力哈希和权限边界",
            details,
        )
    })?;
    validate_runtime_config_capabilities(&request.runtime_config).map_err(|details| {
        json_bad_request_with_details(
            "capability_gated",
            "运行时配置使用了当前 Beta 版本未启用的能力",
            details,
        )
    })?;
    let graph_json = request.graph_json.as_ref()
        .ok_or_else(|| json_bad_request("bad_request", "运行时请求必须包含 graph_json，请从图编辑器发起"))?;
    let qs_protocol = compile_runtime_protocol_via_qs(graph_json)?;
    let compiled =
        compile_runtime_protocol_config(&qs_protocol).map_err(internal_error)?;
    let now_ms = current_time_ms();
    // v2.3.3: 沙盒操作可能阻塞 (HTTP请求/sleep)，移至 spawn_blocking 避免阻塞 tokio 线程
    let (_sandbox, session) = tokio::task::spawn_blocking(move || {
        let mut sandbox = RealTimeSandbox::new(RuntimeCoordinator::new(compiled));
        sandbox.start().map_err(|e| internal_error(anyhow::anyhow!(e)))?;
        let session = sandbox
            .run_session(now_ms, now_ms + RUN_WINDOW_MS)
            .map_err(|e| internal_error(anyhow::anyhow!(e)))?;
        Ok::<_, (StatusCode, String)>((sandbox, session))
    })
    .await
    .map_err(|e| internal_error(anyhow::anyhow!("运行任务被取消: {}", e)))??;
    let run_id = format!("run_{}", now_ms);
    let graph_targets = build_compile_runtime_targets_from_graph(graph_json);
    let runtime_targets = merge_runtime_targets(&request.runtime_targets, &graph_targets);
    let governance = runtime_governance_snapshot(&request.runtime_config.metadata, None);
    let mut events = collect_frontend_events(&session, &runtime_targets);
    prepend_capability_snapshot_event(
        &mut events,
        &run_id,
        &request.runtime_config.metadata.mode,
        now_ms,
        &governance,
    );
    attach_runtime_event_envelopes(
        &mut events,
        &run_id,
        &request.runtime_config.metadata.mode,
        &governance,
    );
    validate_runtime_event_envelopes(&events, &run_id, &governance)
        .map_err(|message| internal_error(anyhow::anyhow!(message)))?;
    let account = account_summary(&session);
    let graph_id = request.runtime_config.metadata.graph_id.clone();
    let compile_id = request.runtime_config.metadata.compile_id.clone();
    let actor = normalize_actor_identity(request.actor);
    let _collaboration = collaboration_with_run_actor(&state.graph_store_dir, &graph_id, &actor).await?;

    let record = RunRecord {
        run_id: run_id.clone(),
        graph_id: graph_id.clone(),
        compile_id: compile_id.clone(),
        created_at_ms: now_ms,
        events: events.clone(),
        account: account.clone(),
        session,
        governance,
        actor: Some(actor.clone()),
    };

    state.runs.write().await.insert(auth::scoped_key(&user_id, &run_id), record);

    Ok(Json(run_start_response(
        run_id,
        graph_id,
        compile_id,
        events.len(),
    )))
}
// v2.4.0: 单机桌面应用, 所有记录属于本机用户, 无需多用户隔离。
// 如未来部署为服务端多用户, 需按 UserId 过滤 + 存储路径前缀隔离。
async fn list_runs(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<RunListItem>>, (StatusCode, String)> {
    let records = list_run_records(state.run_store_dir.as_ref())
        .await
        .map_err(io_error)?;
    let mut items = records
        .into_iter()
        .map(run_list_item_from_record)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    Ok(Json(paginate(items, pagination)))
}
async fn get_run_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunDetailResponse>, (StatusCode, String)> {
    let record = load_run_record_from_state(&state, &user_id, &run_id).await?;
    Ok(Json(run_detail_response_from_record(record)))
}

async fn save_run_record(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunDetailResponse>, (StatusCode, String)> {
    let record = load_run_record_from_state(&state, &user_id, &run_id).await?;
    persist_run_record(state.run_store_dir.as_ref(), &record)
        .await
        .map_err(io_error)?;

    if let Some(actor) = &record.actor {
        persist_graph_audit_entry(
            &state.audit_store_dir,
            &build_graph_audit_entry(
                &record.graph_id,
                actor,
                GraphAuditAction::RunCreated,
                Some(run_id),
                format!("Saved runtime simulation {}", record.run_id),
            ),
        )
        .await
        .map_err(io_error)?;
    }

    Ok(Json(run_detail_response_from_record(record)))
}

async fn discard_run_record(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<DiscardRuntimeArtifactResponse>, (StatusCode, String)> {
    // v1.1.2: 路径遍历防护
    let safe_id = sanitize_storage_path_segment(&run_id);
    let path = state.run_store_dir.join(format!("{}.json", safe_id));
    if fs::try_exists(&path).await.map_err(io_error)? {
        return Err((
            StatusCode::CONFLICT,
            format!("运行记录 '{}' 已保存，无法丢弃", run_id),
        ));
    }

    let scoped_run_id = auth::scoped_key(&user_id, &run_id);
    let removed = state.runs.write().await.remove(&scoped_run_id);
    if removed.is_none() {
        return Err(json_bad_request(
            "not_found",
            format!("运行记录 '{}' 不存在", run_id),
        ));
    }

    Ok(Json(DiscardRuntimeArtifactResponse {
        discarded_id: run_id,
        discarded_kind: "run".to_string(),
    }))
}

async fn get_run_replay(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(query): Query<RuntimeReplayQuery>,
) -> Result<Json<RuntimeReplayResponse>, (StatusCode, String)> {
    let started = Instant::now();
    let options = normalized_replay_options(query);
    let record = load_run_record_from_state(&state, &user_id, &run_id).await?;
    let response = run_replay_response_from_record(record, options)
        .map_err(|message| json_bad_request("bad_replay_cursor", message))?;
    state
        .evidence_metrics
        .record_replay_page(started.elapsed().as_millis() as u64);
    Ok(Json(response))
}
async fn stream_run_events(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)>
{
    let record = load_run_record_from_state(&state, &user_id, &run_id).await?;
    let event_count = record.events.len();

    let stream = stream! {
        yield Ok(json_sse_event("run_started", serde_json::json!({
            "run_id": record.run_id,
            "graph_id": record.graph_id,
            "compile_id": record.compile_id,
            "status": "started"
        })));

        for event in record.events {
            yield Ok(json_sse_event("runtime_event", &event));
            sleep(Duration::from_millis(SSE_EVENT_DELAY_MS)).await;
        }

        yield Ok(json_sse_event("account", &record.account));

        yield Ok(json_sse_event("run_completed", serde_json::json!({
            "run_id": record.run_id,
            "status": "completed",
            "event_count": event_count,
        })));
    };

    // v2.4.0 NOTE: SSE 超时保护需要 tokio-stream 依赖或 stream 级别 timeout wrapper,
    // Axum 0.7 的 Sse 类型不提供 max_age。当前由 TCP keepalive + 浏览器端超时处理。
    // 计划 v2.5.0 添加 tokio-stream 依赖后实现。
    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(5))
            .text("keepalive"),
    ))
}

async fn get_run_status(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunStatusResponse>, (StatusCode, String)> {
    let record = load_run_record_from_state(&state, &user_id, &run_id).await?;
    Ok(Json(run_status_response_from_record(record)))
}

// ── Block 5: 审批流引擎 ──

#[derive(Debug, Deserialize)]
struct RuntimeApprovalListQuery {
    #[serde(default)]
    review_state: Option<String>,
}

// ── Block 5: 合并记录 API ──

#[derive(Debug, Serialize)]
struct MergeRecordsResponse {
    records: Vec<MergeRecordEntry>,
    total_conflicts: usize,
    total_suppressed: usize,
}

#[derive(Debug, Serialize)]
struct MergeRecordEntry {
    cycle_name: String,
    input_count: usize,
    output_count: usize,
    conflicts: usize,
    suppressed: usize,
    merge_policy: String,
}
