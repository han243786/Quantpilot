use crate::{
    auth, build_graph_audit_entry, delete_transient_backtest_record,
    experiment_detail_response_from_record, experiment_list_item_from_record, io_error,
    list_experiment_records, load_backtest_record_from_state, load_experiment_record_from_state,
    paginate, persist_backtest_record, persist_experiment_record, persist_graph_audit_entry,
    runtime::DiscardRuntimeArtifactResponse, sanitize_storage_path_segment, AppState,
    ExperimentDetailResponse, ExperimentListItem, GraphAuditAction, PaginatedResponse,
    PaginationQuery,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use tokio::fs;

pub(crate) async fn list_experiments(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ExperimentListItem>>, (StatusCode, String)> {
    let records = list_experiment_records(state.experiment_store_dir.as_ref())
        .await
        .map_err(io_error)?;
    let mut items = records
        .into_iter()
        .map(experiment_list_item_from_record)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    Ok(Json(paginate(items, pagination)))
}

pub(crate) async fn get_experiment_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<ExperimentDetailResponse>, (StatusCode, String)> {
    let record = load_experiment_record_from_state(&state, &user_id, &experiment_id).await?;
    Ok(Json(experiment_detail_response_from_record(record)))
}

pub(crate) async fn save_experiment_record(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<ExperimentDetailResponse>, (StatusCode, String)> {
    let mut record = load_experiment_record_from_state(&state, &user_id, &experiment_id).await?;

    for variant in &record.variants {
        let variant_record =
            load_backtest_record_from_state(&state, &user_id, &variant.backtest_id).await?;
        persist_backtest_record(state.backtest_store_dir.as_ref(), &variant_record)
            .await
            .map_err(io_error)?;
        delete_transient_backtest_record(
            state.transient_backtest_store_dir.as_ref(),
            &variant.backtest_id,
        )
        .await
        .map_err(io_error)?;
    }

    record.saved = true;
    persist_experiment_record(state.experiment_store_dir.as_ref(), &record)
        .await
        .map_err(io_error)?;
    state
        .experiments
        .write()
        .await
        .insert(auth::scoped_key(&user_id, &experiment_id), record.clone());

    if let Some(actor) = &record.actor {
        persist_graph_audit_entry(
            &state.audit_store_dir,
            &build_graph_audit_entry(
                &record.graph_id,
                actor,
                GraphAuditAction::ExperimentCreated,
                Some(experiment_id),
                format!("Saved backtest sweep {}", record.experiment_id),
            ),
        )
        .await
        .map_err(io_error)?;
    }

    Ok(Json(experiment_detail_response_from_record(record)))
}

pub(crate) async fn discard_experiment_record(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(experiment_id): Path<String>,
) -> Result<Json<DiscardRuntimeArtifactResponse>, (StatusCode, String)> {
    let record = load_experiment_record_from_state(&state, &user_id, &experiment_id).await?;
    if record.saved {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "experiment `{}` is already saved and cannot be discarded",
                experiment_id
            ),
        ));
    }

    // v1.1.9: 路径遍历防护
    let safe_id = sanitize_storage_path_segment(&experiment_id);
    let path = state.experiment_store_dir.join(format!("{}.json", safe_id));

    let scoped_experiment_id = auth::scoped_key(&user_id, &experiment_id);
    state
        .experiments
        .write()
        .await
        .remove(&scoped_experiment_id);
    if fs::try_exists(&path).await.map_err(io_error)? {
        fs::remove_file(&path).await.map_err(io_error)?;
    }

    let mut transient_variant_ids = Vec::new();
    for variant in &record.variants {
        let dir = state.backtest_store_dir.join(&variant.backtest_id);
        if !fs::try_exists(&dir).await.map_err(io_error)? {
            transient_variant_ids.push(variant.backtest_id.clone());
        }
    }

    if !transient_variant_ids.is_empty() {
        let mut backtests = state.backtests.write().await;
        for backtest_id in &transient_variant_ids {
            let scoped = auth::scoped_key(&user_id, backtest_id);
            backtests.remove(&scoped);
        }
    }
    for backtest_id in transient_variant_ids {
        delete_transient_backtest_record(state.transient_backtest_store_dir.as_ref(), &backtest_id)
            .await
            .map_err(io_error)?;
    }

    Ok(Json(DiscardRuntimeArtifactResponse {
        discarded_id: experiment_id,
        discarded_kind: "experiment".to_string(),
    }))
}
