use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.storage_security";

pub mod credential_api;
pub mod credential_vault;

pub use credential_vault::CredentialVault;

pub(crate) fn register_credential_routes(router: Router<AppState>) -> Router<AppState> {
    credential_api::register_routes(router)
}
