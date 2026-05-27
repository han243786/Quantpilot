use axum::{
    routing::{get, post},
    Router,
};

use crate::AppState;

pub const MODULE_ID: &str = "backend.ops_governance.hotswap";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/hotswap", post(crate::hotswap_api::submit_hotswap))
        .route("/api/hotswap/list", get(crate::hotswap_api::list_hotswaps))
        .route(
            "/api/hotswap/:hotswap_id",
            get(crate::hotswap_api::get_hotswap_status),
        )
}
