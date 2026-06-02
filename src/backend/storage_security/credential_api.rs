use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.storage_security.credential_api";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    super::register_credential_handler_routes(router)
}
