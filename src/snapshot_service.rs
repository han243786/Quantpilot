use axum::Router;

use crate::{backend::ops_governance::snapshots, AppState};

#[allow(dead_code)]
pub(super) fn register_snapshot_routes(router: Router<AppState>) -> Router<AppState> {
    snapshots::register_routes(router)
}
