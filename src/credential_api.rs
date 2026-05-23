use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Json, Router};
use std::collections::BTreeMap;

use super::auth::{self, UserId};
use super::AppState;

pub(super) fn register_credential_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/credentials",
            get(list_credentials).post(set_credential),
        )
        .route("/api/credentials/:service", delete(delete_credential))
}

/// v2.3.3: 按用户隔离凭证 — vault key 格式为 `{user_id}:{service}`
fn scoped_cv_key(user_id: &UserId, service: &str) -> String {
    format!("{}:{}", user_id.0, service)
}

fn unscoped_services_for(
    vault: &super::credential_vault::CredentialVault,
    user_id: &UserId,
) -> Vec<String> {
    let prefix = format!("{}:", user_id.0);
    vault
        .list_services()
        .into_iter()
        .filter(|s| s.starts_with(&prefix))
        .map(|s| s[prefix.len()..].to_string())
        .collect()
}

/// GET /api/credentials → { "services": ["okx", "binance"] }
async fn list_credentials(
    user_id: auth::UserId,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match &state.credential_vault {
        Some(vault) => {
            let services = unscoped_services_for(vault, &user_id);
            Ok(Json(serde_json::json!({ "services": services })))
        }
        None => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "凭证保险库未初始化".to_string(),
        )),
    }
}

/// POST /api/credentials ← { "service": "okx", "fields": {"key":"...","secret":"..."} }
async fn set_credential(
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

    let service = body["service"]
        .as_str()
        .filter(|s| {
            !s.trim().is_empty()
                && s.len() <= 64
                && !s.contains('/')
                && !s.contains('\\')
                && !s.contains("..")
        })
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "缺少 'service' 字段".to_string()))?;

    let fields_obj = body["fields"]
        .as_object()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "缺少 'fields' 对象".to_string()))?;

    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    for (k, v) in fields_obj {
        let val = v.as_str().unwrap_or_default().to_string();
        if val.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("字段 '{}' 的值不能为空", k),
            ));
        }
        fields.insert(k.clone(), val);
    }

    let scoped_key = scoped_cv_key(&user_id, service);
    vault.set_service(&scoped_key, fields).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("凭证存储失败: {}", e),
        )
    })?;

    safe_eprintln!("[audit] 用户 {} 设置凭证 service={}", user_id.0, service);

    Ok(Json(serde_json::json!({ "stored": service })))
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
