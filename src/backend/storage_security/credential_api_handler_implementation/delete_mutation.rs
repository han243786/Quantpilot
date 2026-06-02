use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use super::scoped_cv_key;
use crate::auth;
use crate::AppState;

/// DELETE /api/credentials/:service → { "deleted": "okx" }
pub(super) async fn delete_credential(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(service): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if service.is_empty()
        || service.len() > 64
        || service.contains('/')
        || service.contains('\\')
        || service.contains("..")
        || service.contains('\0')
    {
        return Err((StatusCode::BAD_REQUEST, "凭证标签无效".to_string()));
    }
    let vault = state.credential_vault.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "凭证保险库未初始化".to_string(),
        )
    })?;

    let scoped_key = scoped_cv_key(&user_id, &service);
    vault.delete_service(&scoped_key).map_err(|e| {
        if e.to_string().contains("不存在") {
            (StatusCode::NOT_FOUND, format!("标签 '{}' 不存在", service))
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("凭证删除失败: {}", e),
            )
        }
    })?;

    safe_eprintln!("[audit] 用户 {} 删除凭证 service={}", user_id.0, service);

    Ok(Json(serde_json::json!({ "deleted": service })))
}
