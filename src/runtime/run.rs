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
