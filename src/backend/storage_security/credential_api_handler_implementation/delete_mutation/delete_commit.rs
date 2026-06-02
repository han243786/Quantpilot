use axum::http::StatusCode;
use axum::Json;

use crate::auth;
use crate::credential_vault::CredentialVault;

pub(super) fn commit_delete_credential(
    vault: &CredentialVault,
    user_id: &auth::UserId,
    scoped_key: &str,
    service: String,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    vault.delete_service(scoped_key).map_err(|e| {
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
