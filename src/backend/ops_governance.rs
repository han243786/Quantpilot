use axum::{
    routing::{get, post},
    Router,
};

use crate::AppState;

pub const MODULE_ID: &str = "backend.ops_governance";

pub(crate) fn register_hotswap_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/hotswap", post(crate::hotswap_api::submit_hotswap))
        .route("/api/hotswap/list", get(crate::hotswap_api::list_hotswaps))
        .route(
            "/api/hotswap/:hotswap_id",
            get(crate::hotswap_api::get_hotswap_status),
        )
}

pub(crate) fn register_sandbox_verification_routes(router: Router<AppState>) -> Router<AppState> {
    crate::sandbox_verification::register_sandbox_verification_routes(router)
}

pub(crate) fn register_alert_routes(router: Router<AppState>) -> Router<AppState> {
    crate::alert_engine::register_alert_routes(router)
}

pub(crate) fn register_snapshot_routes(router: Router<AppState>) -> Router<AppState> {
    crate::snapshot_service::register_snapshot_routes(router)
}

pub(crate) fn register_runbook_routes(router: Router<AppState>) -> Router<AppState> {
    crate::runbook::register_runbook_routes(router)
}

pub(crate) fn register_chaos_routes(router: Router<AppState>) -> Router<AppState> {
    crate::chaos_experiment::register_chaos_routes(router)
}
