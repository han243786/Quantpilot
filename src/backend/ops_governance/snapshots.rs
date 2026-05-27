use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.ops_governance.snapshots";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    crate::snapshot_service::register_snapshot_routes(router)
}
