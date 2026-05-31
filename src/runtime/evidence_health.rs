use crate::{
    cleanup_transient_runtime_report_outputs, current_time_ms, io_error,
    list_runtime_report_records, runtime_evidence_cleanup_policy, AppState,
    RuntimeEvidenceCleanupRequest, RuntimeEvidenceCleanupResponse, RuntimeEvidenceHealthResponse,
    RuntimeEvidenceReportRecord, RuntimeEvidenceReportStatusCounts, RuntimeReportLifecycleStatus,
};
use axum::{extract::State, http::StatusCode, Json};

fn runtime_report_status_counts(
    records: &[RuntimeEvidenceReportRecord],
) -> RuntimeEvidenceReportStatusCounts {
    let mut counts = RuntimeEvidenceReportStatusCounts {
        requested: 0,
        generating: 0,
        ready: 0,
        failed: 0,
        expired: 0,
        source_changed: 0,
    };
    for record in records {
        match record.status {
            RuntimeReportLifecycleStatus::Requested => counts.requested += 1,
            RuntimeReportLifecycleStatus::Generating => counts.generating += 1,
            RuntimeReportLifecycleStatus::Ready => counts.ready += 1,
            RuntimeReportLifecycleStatus::Failed => counts.failed += 1,
            RuntimeReportLifecycleStatus::Expired => counts.expired += 1,
            RuntimeReportLifecycleStatus::SourceChanged => counts.source_changed += 1,
        }
    }
    counts
}

pub(crate) async fn get_runtime_evidence_health(
    State(state): State<AppState>,
) -> Result<Json<RuntimeEvidenceHealthResponse>, (StatusCode, String)> {
    let reports = list_runtime_report_records(&state.report_store_dir)
        .await
        .map_err(io_error)?;
    Ok(Json(RuntimeEvidenceHealthResponse {
        status: "ok".to_string(),
        metrics: state.evidence_metrics.snapshot(),
        persisted_report_count: reports.len(),
        report_status_counts: runtime_report_status_counts(&reports),
        cleanup_policy: runtime_evidence_cleanup_policy(),
    }))
}

pub(crate) async fn cleanup_runtime_evidence(
    State(state): State<AppState>,
    Json(request): Json<RuntimeEvidenceCleanupRequest>,
) -> Result<Json<RuntimeEvidenceCleanupResponse>, (StatusCode, String)> {
    let policy = runtime_evidence_cleanup_policy();
    let max_age_ms = request
        .max_age_ms
        .unwrap_or(policy.transient_generation_ttl_ms);
    let removed = cleanup_transient_runtime_report_outputs(
        state.report_store_dir.as_ref(),
        max_age_ms,
        current_time_ms(),
    )
    .await
    .map_err(io_error)?;
    let retained_report_records = list_runtime_report_records(&state.report_store_dir)
        .await
        .map_err(io_error)?
        .len();
    Ok(Json(RuntimeEvidenceCleanupResponse {
        policy,
        removed_transient_generation_outputs: removed,
        retained_report_records,
    }))
}
