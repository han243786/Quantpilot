use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use std::collections::BTreeMap;

use super::scoped_cv_key;
use crate::auth;
use crate::AppState;

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
