use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.interface_boundary.runtime_bridge";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::runtime::register_routes(router)
}
