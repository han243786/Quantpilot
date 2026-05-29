use super::*;

// v2.4.0: 单机桌面应用, 所有记录属于本机用户, 无需多用户隔离。
// 如未来部署为服务端多用户, 需按 UserId 过滤 + 存储路径前缀隔离。
pub(crate) async fn list_backtests(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<BacktestListItem>>, (StatusCode, String)> {
    let records = list_backtest_records(state.backtest_store_dir.as_ref())
        .await
        .map_err(io_error)?;
    let mut items = records
        .into_iter()
        .map(backtest_list_item_from_record)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    Ok(Json(paginate(items, pagination)))
}

pub(crate) async fn get_backtest_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
) -> Result<Json<BacktestDetailResponse>, (StatusCode, String)> {
    let record = load_backtest_record_from_state(&state, &user_id, &backtest_id).await?;
    Ok(Json(backtest_detail_response_from_record(record)))
}

pub(crate) async fn save_backtest_record(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
) -> Result<Json<BacktestDetailResponse>, (StatusCode, String)> {
    let mut record = load_backtest_record_from_state(&state, &user_id, &backtest_id).await?;
    let views = persist_backtest_record(state.backtest_store_dir.as_ref(), &record)
        .await
        .map_err(io_error)?;
    delete_transient_backtest_record(state.transient_backtest_store_dir.as_ref(), &backtest_id)
        .await
        .map_err(io_error)?;
    record.backtest_artifacts = Some(views);
    state
        .backtests
        .write()
        .await
        .insert(auth::scoped_key(&user_id, &backtest_id), record.clone());

    if let Some(actor) = &record.actor {
        persist_graph_audit_entry(
            &state.audit_store_dir,
            &build_graph_audit_entry(
                &record.graph_id,
                actor,
                GraphAuditAction::BacktestCreated,
                Some(backtest_id),
                format!("Saved backtest {}", record.backtest_id),
            ),
        )
        .await
        .map_err(io_error)?;
    }

    Ok(Json(backtest_detail_response_from_record(record)))
}

pub(crate) async fn discard_backtest_record(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
) -> Result<Json<DiscardRuntimeArtifactResponse>, (StatusCode, String)> {
    // v1.1.2: 路径遍历防护
    let safe_id = sanitize_storage_path_segment(&backtest_id);
    let dir = state.backtest_store_dir.join(&safe_id);
    if fs::try_exists(&dir).await.map_err(io_error)? {
        return Err((
            StatusCode::CONFLICT,
            format!("回测 `{}` 已保存, 无法丢弃", backtest_id),
        ));
    }

    let scoped_backtest_id = auth::scoped_key(&user_id, &backtest_id);
    let removed_memory = state
        .backtests
        .write()
        .await
        .remove(&scoped_backtest_id)
        .is_some();
    let removed_transient =
        delete_transient_backtest_record(state.transient_backtest_store_dir.as_ref(), &backtest_id)
            .await
            .map_err(io_error)?;
    if !removed_memory && !removed_transient {
        return Err((
            StatusCode::NOT_FOUND,
            format!("回测 `{}` 不存在", backtest_id),
        ));
    }

    Ok(Json(DiscardRuntimeArtifactResponse {
        discarded_id: backtest_id,
        discarded_kind: "backtest".to_string(),
    }))
}
