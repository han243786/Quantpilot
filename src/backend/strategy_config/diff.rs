use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.strategy_config.diff";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    crate::strategy_config_api::register_strategy_config_diff_route(router)
}
