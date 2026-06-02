use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use super::scoped_cv_key;
use crate::auth;
use crate::AppState;

mod delete_commit;
mod service_path_validation;

/// DELETE /api/credentials/:service → { "deleted": "okx" }
pub(super) async fn delete_credential(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(service): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = service_path_validation::validate_service_path(service)?;
    let vault = state.credential_vault.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "凭证保险库未初始化".to_string(),
        )
    })?;

    let scoped_key = scoped_cv_key(&user_id, &service);
    delete_commit::commit_delete_credential(vault, &user_id, &scoped_key, service)
}
