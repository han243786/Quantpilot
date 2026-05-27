use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.storage_security";

pub use crate::credential_vault::CredentialVault;

pub(crate) fn register_credential_routes(router: Router<AppState>) -> Router<AppState> {
    crate::credential_api::register_credential_routes(router)
}
