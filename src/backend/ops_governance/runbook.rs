use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.ops_governance.runbook";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    crate::runbook::register_runbook_routes(router)
}
