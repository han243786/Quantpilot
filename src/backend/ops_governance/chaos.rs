use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.ops_governance.chaos";

mod handlers;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    handlers::register_chaos_routes(router)
}
