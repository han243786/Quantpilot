use crate::*;

use super::{load_sandbox_report_from_disk, run_sandbox_verification};

pub(super) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/v1/ai/proposals/:proposal_id/sandbox-report",
            get(get_sandbox_report),
        )
        .route(
            "/api/v1/ai/proposals/:proposal_id/request-sandbox",
            post(request_sandbox_verification),
        )
}

async fn get_sandbox_report(
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
) -> Result<Json<SandboxVerificationReport>, (StatusCode, String)> {
    let reports = state.sandbox_reports.read().await;
    if let Some(report) = reports.values().find(|r| r.proposal_id == proposal_id) {
        return Ok(Json(report.clone()));
    }
    // 尝试从磁盘加载
    match load_sandbox_report_from_disk(&state.sandbox_report_store_dir, &proposal_id).await {
        Ok(report) => Ok(Json(report)),
        Err(_) => Err(json_bad_request(
            "not_found",
            format!("提案 '{}' 的沙箱报告不存在", proposal_id),
        )),
    }
}

async fn request_sandbox_verification(
    State(state): State<AppState>,
    Path(_proposal_id): Path<String>,
    Json(request): Json<RequestSandboxVerificationRequest>,
) -> Result<Json<SandboxVerificationReport>, (StatusCode, String)> {
    let report = run_sandbox_verification(&state, &request).await?;
    Ok(Json(report))
}
