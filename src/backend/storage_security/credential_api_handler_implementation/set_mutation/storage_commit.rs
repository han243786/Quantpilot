use axum::http::StatusCode;
use axum::Json;

use crate::auth;
use crate::credential_vault::{CredentialFields, CredentialVault};

pub(super) fn commit_set_credential(
    vault: &CredentialVault,
    user_id: &auth::UserId,
    scoped_key: &str,
    service: String,
    fields: CredentialFields,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    vault.set_service(scoped_key, fields).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("凭证存储失败: {}", e),
        )
    })?;

    safe_eprintln!("[audit] 用户 {} 设置凭证 service={}", user_id.0, service);

    Ok(Json(serde_json::json!({ "stored": service })))
}
