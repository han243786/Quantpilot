use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.ops_governance.sandbox";

mod handlers;

pub(crate) use handlers::{load_sandbox_report_from_disk, run_sandbox_verification};

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    handlers::register_routes(router)
}
