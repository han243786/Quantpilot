use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.storage_security.credential_api";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    crate::credential_api::register_credential_routes(router)
}
