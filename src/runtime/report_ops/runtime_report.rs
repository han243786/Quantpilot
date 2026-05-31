use crate::{
    auth, current_time_ms, io_error, list_runtime_report_records, load_backtest_record_from_state,
    load_run_record_from_state, load_runtime_report_record, paginate,
    persist_runtime_report_record, runtime_report_artifact_from_record,
    runtime_report_record_from_backtest_record, runtime_report_record_from_run_record, AppState,
    CreateRuntimeReportRequest, PaginatedResponse, PaginationQuery, RuntimeEvidenceReportArtifact,
    RuntimeEvidenceReportRecord, RuntimeEvidenceSourceKind, RuntimeReportFailureMetadata,
    RuntimeReportLifecycleStatus,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

pub(crate) async fn create_runtime_report(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<CreateRuntimeReportRequest>,
) -> Result<Json<RuntimeEvidenceReportRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let report = match request.source_kind {
        RuntimeEvidenceSourceKind::Run => {
            let record = load_run_record_from_state(&state, &user_id, &request.source_id).await?;
            runtime_report_record_from_run_record(record, now_ms, request.generation_policy)
        }
        RuntimeEvidenceSourceKind::Backtest => {
            let record =
                load_backtest_record_from_state(&state, &user_id, &request.source_id).await?;
            runtime_report_record_from_backtest_record(record, now_ms, request.generation_policy)
        }
    };

    match load_runtime_report_record(state.report_store_dir.as_ref(), &report.report_id).await {
        Ok(existing) => return Ok(Json(existing)),
        Err((StatusCode::NOT_FOUND, _)) => {}
        Err(error) => return Err(error),
    }

    state.evidence_metrics.record_report_generation(&report);
    persist_runtime_report_record(state.report_store_dir.as_ref(), &report)
        .await
        .map_err(io_error)?;
    Ok(Json(report))
}

fn report_source_metadata_matches(
    saved: &RuntimeEvidenceReportRecord,
    current: &RuntimeEvidenceReportRecord,
) -> bool {
    saved.graph_id == current.graph_id
        && saved.source_sequence_range == current.source_sequence_range
        && saved.source_event_count == current.source_event_count
        && saved.retained_event_count == current.retained_event_count
        && saved.governance == current.governance
        && saved.generation_policy == current.generation_policy
}

fn source_changed_report(
    mut record: RuntimeEvidenceReportRecord,
    reason_code: &str,
    message: impl Into<String>,
) -> RuntimeEvidenceReportRecord {
    let message = message.into();
    record.status = RuntimeReportLifecycleStatus::SourceChanged;
    record.failure_reason = Some(message.clone());
    record.failure = Some(RuntimeReportFailureMetadata {
        reason_code: reason_code.to_string(),
        message,
        retry_eligible: true,
    });
    record.artifacts.clear();
    record.updated_at_ms = current_time_ms();
    record
}

async fn current_report_for_saved_source(
    state: &AppState,
    user_id: &auth::UserId,
    record: &RuntimeEvidenceReportRecord,
) -> Result<Option<RuntimeEvidenceReportRecord>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    match record.source_kind {
        RuntimeEvidenceSourceKind::Run => {
            match load_run_record_from_state(state, user_id, &record.source_id).await {
                Ok(source) => Ok(Some(runtime_report_record_from_run_record(
                    source,
                    now_ms,
                    Some(record.generation_policy.clone()),
                ))),
                Err((StatusCode::NOT_FOUND, _)) => Ok(None),
                Err(error) => Err(error),
            }
        }
        RuntimeEvidenceSourceKind::Backtest => {
            match load_backtest_record_from_state(state, user_id, &record.source_id).await {
                Ok(source) => Ok(Some(runtime_report_record_from_backtest_record(
                    source,
                    now_ms,
                    Some(record.generation_policy.clone()),
                ))),
                Err((StatusCode::NOT_FOUND, _)) => Ok(None),
                Err(error) => Err(error),
            }
        }
    }
}

async fn materialize_runtime_report_record(
    state: &AppState,
    user_id: &auth::UserId,
    record: RuntimeEvidenceReportRecord,
) -> Result<RuntimeEvidenceReportRecord, (StatusCode, String)> {
    if record.status != RuntimeReportLifecycleStatus::Ready {
        return Ok(record);
    }
    let Some(current) = current_report_for_saved_source(state, user_id, &record).await? else {
        state.evidence_metrics.record_report_source_changed();
        return Ok(source_changed_report(
            record,
            "source_missing",
            "source evidence record is no longer available for report validation",
        ));
    };
    if report_source_metadata_matches(&record, &current) {
        Ok(record)
    } else {
        state.evidence_metrics.record_report_source_changed();
        Ok(source_changed_report(
            record,
            "source_changed",
            "source evidence metadata changed after report generation",
        ))
    }
}

pub(crate) async fn list_runtime_reports(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<RuntimeEvidenceReportRecord>>, (StatusCode, String)> {
    let records = list_runtime_report_records(&state.report_store_dir)
        .await
        .map_err(io_error)?;
    let mut records = {
        let mut materialized = Vec::new();
        for record in records {
            materialized.push(materialize_runtime_report_record(&state, &user_id, record).await?);
        }
        materialized
    };
    records.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.report_id.cmp(&left.report_id))
    });
    Ok(Json(paginate(records, pagination)))
}

pub(crate) async fn get_runtime_report_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(report_id): Path<String>,
) -> Result<Json<RuntimeEvidenceReportRecord>, (StatusCode, String)> {
    let record = load_runtime_report_record(state.report_store_dir.as_ref(), &report_id).await?;
    materialize_runtime_report_record(&state, &user_id, record)
        .await
        .map(Json)
}

pub(crate) async fn export_runtime_report_artifact(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(report_id): Path<String>,
) -> Result<Json<RuntimeEvidenceReportArtifact>, (StatusCode, String)> {
    let record = load_runtime_report_record(state.report_store_dir.as_ref(), &report_id).await?;
    let record = materialize_runtime_report_record(&state, &user_id, record).await?;
    Ok(Json(runtime_report_artifact_from_record(&record)))
}
