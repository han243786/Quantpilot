use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.test_support";

pub(crate) fn register_test_scenario_routes(router: Router<AppState>) -> Router<AppState> {
    crate::api_test_scenario::register_test_scenario_routes(router)
}
