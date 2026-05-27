use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.interface_boundary.strategy_config_bridge";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::strategy_config::register_routes(router)
}
