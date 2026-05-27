use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.strategy_config.ai_proposal_binding";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
}
