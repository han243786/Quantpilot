use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.test_support";

pub mod scenario;

pub(crate) fn register_test_scenario_routes(router: Router<AppState>) -> Router<AppState> {
    scenario::register_routes(router)
}
