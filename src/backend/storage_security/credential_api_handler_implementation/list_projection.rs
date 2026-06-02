use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use super::super::CredentialVault;
use crate::auth::{self, UserId};
use crate::AppState;

fn unscoped_services_for(vault: &CredentialVault, user_id: &UserId) -> Vec<String> {
    let prefix = format!("{}:", user_id.0);
    vault
        .list_services()
        .into_iter()
        .filter(|s| s.starts_with(&prefix))
        .map(|s| s[prefix.len()..].to_string())
        .collect()
}

/// GET /api/credentials → { "services": ["okx", "binance"] }
pub(super) async fn list_credentials(
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
