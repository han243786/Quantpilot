use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.runtime";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    crate::runtime::register_runtime_routes(router)
}
