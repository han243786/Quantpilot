use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.storage_security";

pub mod credential_api;
mod credential_api_handler_implementation;
pub mod credential_vault;

pub use credential_vault::CredentialVault;

pub(crate) fn register_credential_routes(router: Router<AppState>) -> Router<AppState> {
    credential_api::register_routes(router)
}

fn register_credential_handler_routes(router: Router<AppState>) -> Router<AppState> {
    credential_api_handler_implementation::register_credential_routes(router)
}
