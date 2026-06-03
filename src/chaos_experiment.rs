use axum::Router;

use crate::{backend::ops_governance::chaos, AppState};

#[allow(dead_code)]
pub(super) fn register_chaos_routes(router: Router<AppState>) -> Router<AppState> {
    chaos::register_routes(router)
}
