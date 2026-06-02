use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Json, Router};

use crate::auth::{self, UserId};
use crate::AppState;

mod key_scope;
mod list_projection;
mod set_mutation;

pub(super) fn register_credential_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/credentials",
            get(list_projection::list_credentials).post(set_mutation::set_credential),
        )
        .route("/api/credentials/:service", delete(delete_credential))
}

/// v2.3.3: 按用户隔离凭证 — vault key 格式为 `{user_id}:{service}`
fn scoped_cv_key(user_id: &UserId, service: &str) -> String {
    key_scope::scoped_cv_key(user_id, service)
}

/// DELETE /api/credentials/:service → { "deleted": "okx" }
async fn delete_credential(
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
