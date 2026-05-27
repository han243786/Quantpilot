use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.interface_boundary.test_support_bridge";

pub(crate) fn register_test_scenario_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::test_support::register_test_scenario_routes(router)
}
