use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.interface_boundary.storage_security_bridge";

pub(crate) fn register_credential_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::storage_security::register_credential_routes(router)
}
