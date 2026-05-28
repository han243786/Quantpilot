use super::*;

pub(crate) async fn start_test_run(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<FrontendRunRequest>,
) -> Result<Json<RunStartResponse>, (StatusCode, String)> {
    // v1.0.3: 运行互斥 — 同一时间只允许一个 Paper 运行
    // v1.3.5: RAII guard 确保运行结束后自动复位
    if state
        .run_in_progress
        .swap(true, std::sync::atomic::Ordering::AcqRel)
    {
        return Err((
            StatusCode::CONFLICT,
            "已有运行在进行中, 请先停止当前运行后再启动新的运行".to_string(),
        ));
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
    let graph_json = request.graph_json.as_ref().ok_or_else(|| {
        json_bad_request(
            "bad_request",
            "运行时请求必须包含 graph_json，请从图编辑器发起",
        )
    })?;
    let qs_protocol = compile_runtime_protocol_via_qs(graph_json)?;
    let compiled = compile_runtime_protocol_config(&qs_protocol).map_err(internal_error)?;
    let now_ms = current_time_ms();
    // v2.3.3: 沙盒操作可能阻塞 (HTTP请求/sleep)，移至 spawn_blocking 避免阻塞 tokio 线程
    let (_sandbox, session) = tokio::task::spawn_blocking(move || {
        let mut sandbox = RealTimeSandbox::new(RuntimeCoordinator::new(compiled));
        sandbox
            .start()
            .map_err(|e| internal_error(anyhow::anyhow!(e)))?;
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
    let _collaboration =
        collaboration_with_run_actor(&state.graph_store_dir, &graph_id, &actor).await?;

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

    state
        .runs
        .write()
        .await
        .insert(auth::scoped_key(&user_id, &run_id), record);

    Ok(Json(run_start_response(
        run_id,
        graph_id,
        compile_id,
        events.len(),
    )))
}
