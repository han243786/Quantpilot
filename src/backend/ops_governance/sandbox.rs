use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.ops_governance.sandbox";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    crate::sandbox_verification::register_sandbox_verification_routes(router)
}
