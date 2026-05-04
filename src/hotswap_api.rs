use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use super::frontend_api_types::{
    HotSwapRecord, HotSwapResponse, HotSwapStatusResponse, SubmitHotSwapRequest,
};
use super::AppState;

pub(super) async fn submit_hotswap(
    State(state): State<AppState>,
    Json(body): Json<SubmitHotSwapRequest>,
) -> impl IntoResponse {
    let now_ms = std::time::UNIX_EPOCH
        .elapsed()
        .unwrap_or_default()
        .as_millis() as u64;
    let hotswap_id = format!("hotswap-{now_ms}");

    let record = HotSwapRecord {
        hotswap_id: hotswap_id.clone(),
        status: "proposed".to_string(),
        step: "idle".to_string(),
        request: body.clone(),
        started_at_ms: now_ms,
        completed_at_ms: None,
        success: None,
        rollback_reason: None,
        events: Vec::new(),
    };

    // Compatibility validation
    if body.module_targets.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "type": "https://quantpilot.dev/problems/hotswap-validation-failed",
                "title": "Validation failed",
                "status": 400,
                "detail": "No module targets specified",
                "error_code": "HOTSWAP_NO_TARGETS",
            })),
        );
    }

    for target in &body.module_targets {
        if target.module_key.trim().is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "type": "https://quantpilot.dev/problems/hotswap-validation-failed",
                    "title": "Validation failed",
                    "status": 400,
                    "detail": "Module key must not be empty",
                    "error_code": "HOTSWAP_EMPTY_MODULE_KEY",
                })),
            );
        }
    }

    state
        .hotswap_records
        .write()
        .await
        .insert(hotswap_id.clone(), record);

    let response = HotSwapResponse {
        hotswap_id: hotswap_id.clone(),
        success: true,
        new_deployment_revision: Some(body.deployment_revision),
        rollback_reason: None,
        final_step: "proposed".to_string(),
        elapsed_ms: 0,
        event_count: 0,
    };

    (StatusCode::OK, Json(serde_json::to_value(response).unwrap_or_default()))
}

pub(super) async fn get_hotswap_status(
    State(state): State<AppState>,
    Path(hotswap_id): Path<String>,
) -> impl IntoResponse {
    let records = state.hotswap_records.read().await;

    match records.get(&hotswap_id) {
        Some(record) => {
            let response = HotSwapStatusResponse {
                hotswap_id: record.hotswap_id.clone(),
                status: record.status.clone(),
                step: record.step.clone(),
                started_at_ms: record.started_at_ms,
                events: record.events.clone(),
            };
            (StatusCode::OK, Json(serde_json::to_value(response).unwrap_or_default()))
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "type": "https://quantpilot.dev/problems/hotswap-not-found",
                "title": "Hot-swap record not found",
                "status": 404,
                "detail": format!("No hot-swap record found with id '{}'", hotswap_id),
            })),
        ),
    }
}

pub(super) async fn list_hotswaps(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let records = state.hotswap_records.read().await;
    let items: Vec<&HotSwapRecord> = records.values().collect();

    (StatusCode::OK, Json(json!({
        "hotswaps": items.iter().map(|r| json!({
            "hotswap_id": r.hotswap_id,
            "status": r.status,
            "step": r.step,
            "started_at_ms": r.started_at_ms,
            "success": r.success,
        })).collect::<Vec<_>>(),
    })))
}
