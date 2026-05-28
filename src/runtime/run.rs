pub(crate) async fn start_test_run(
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
pub(crate) async fn list_runs(
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
pub(crate) async fn get_run_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunDetailResponse>, (StatusCode, String)> {
    let record = load_run_record_from_state(&state, &user_id, &run_id).await?;
    Ok(Json(run_detail_response_from_record(record)))
}

pub(crate) async fn save_run_record(
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

pub(crate) async fn discard_run_record(
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

pub(crate) async fn get_run_replay(
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
pub(crate) async fn stream_run_events(
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

pub(crate) async fn get_run_status(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunStatusResponse>, (StatusCode, String)> {
    let record = load_run_record_from_state(&state, &user_id, &run_id).await?;
    Ok(Json(run_status_response_from_record(record)))
}

// ── Block 5: 审批流引擎 ──

#[derive(Debug, Deserialize)]
pub(crate) struct RuntimeApprovalListQuery {
    #[serde(default)]
    review_state: Option<String>,
}

// ── Block 5: 合并记录 API ──

#[derive(Debug, Serialize)]
pub(crate) struct MergeRecordsResponse {
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
