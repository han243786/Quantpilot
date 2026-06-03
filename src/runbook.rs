use axum::Router;

use crate::{backend::ops_governance::runbook, AppState};

#[allow(dead_code)]
pub(super) fn register_runbook_routes(router: Router<AppState>) -> Router<AppState> {
    runbook::register_routes(router)
}
