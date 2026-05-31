use crate::{
    auth, build_graph_audit_entry, io_error, json_bad_request, list_run_records,
    load_run_record_from_state, paginate, persist_graph_audit_entry, persist_run_record,
    run_detail_response_from_record, run_list_item_from_record,
    runtime::DiscardRuntimeArtifactResponse, sanitize_storage_path_segment, AppState,
    GraphAuditAction, PaginatedResponse, PaginationQuery, RunDetailResponse, RunListItem,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use tokio::fs;

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
