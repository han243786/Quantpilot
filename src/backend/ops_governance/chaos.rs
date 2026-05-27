use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.ops_governance.chaos";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    crate::chaos_experiment::register_chaos_routes(router)
}
