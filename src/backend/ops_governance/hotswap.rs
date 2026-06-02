use axum::{
    routing::{get, post},
    Router,
};

use crate::AppState;

pub const MODULE_ID: &str = "backend.ops_governance.hotswap";

mod handlers;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/hotswap", post(handlers::submit_hotswap))
        .route("/api/hotswap/list", get(handlers::list_hotswaps))
        .route(
            "/api/hotswap/:hotswap_id",
            get(handlers::get_hotswap_status),
        )
}
