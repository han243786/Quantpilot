use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.ops_governance.alerts";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    crate::alert_engine::register_alert_routes(router)
}
