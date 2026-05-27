use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.strategy_config";

pub mod ai_proposal_binding;
pub mod artifact;
pub mod diff;
pub mod preflight;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    let router = artifact::register_routes(router);
    let router = preflight::register_routes(router);
    let router = diff::register_routes(router);
    ai_proposal_binding::register_routes(router)
}
