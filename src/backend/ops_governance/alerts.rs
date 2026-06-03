use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.ops_governance.alerts";

mod handlers;

pub(crate) async fn init_alert_rules(state: &AppState) {
    handlers::init_alert_rules(state).await;
}

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    handlers::register_alert_routes(router)
}
