use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use super::scoped_cv_key;
use crate::auth;
use crate::AppState;

mod service_and_fields_validation;

/// POST /api/credentials ← { "service": "okx", "fields": {"key":"...","secret":"..."} }
pub(super) async fn set_credential(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let vault = state.credential_vault.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "凭证保险库未初始化".to_string(),
        )
    })?;

    let (service, fields) = service_and_fields_validation::validate_set_request(&body)?;

    let scoped_key = scoped_cv_key(&user_id, &service);
    vault.set_service(&scoped_key, fields).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("凭证存储失败: {}", e),
        )
    })?;

    safe_eprintln!("[audit] 用户 {} 设置凭证 service={}", user_id.0, service);

    Ok(Json(serde_json::json!({ "stored": service })))
}
