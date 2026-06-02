use crate::*;

pub(super) async fn commit_report(
    state: &AppState,
    request: &RequestSandboxVerificationRequest,
    report: &SandboxVerificationReport,
) -> Result<(), (StatusCode, String)> {
    if let Err(e) = crate::storage_lifecycle::ensure_storage_quota(
        std::path::Path::new("storage"),
        "sandbox-reports",
        crate::storage_lifecycle::StorageLifecycle::Transient,
    ) {
        return Err(io_error(e));
    }

    persist_json(&state.sandbox_report_store_dir, &report.proposal_id, report)
        .await
        .map_err(io_error)?;

    state
        .sandbox_reports
        .write()
        .await
        .insert(request.proposal_id.clone(), report.clone());

    state
        .evidence_metrics
        .report_generation_count
        .fetch_add(1, Ordering::Relaxed);

    Ok(())
}
