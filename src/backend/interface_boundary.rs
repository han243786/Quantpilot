use axum::{response::IntoResponse, Router};

use crate::AppState;

pub const MODULE_ID: &str = "backend.interface_boundary";

pub use crate::app_router::build_app_router;

pub(crate) async fn get_capabilities() -> impl IntoResponse {
    crate::backend::capability::get_capabilities().await
}

pub(crate) async fn health(state: axum::extract::State<AppState>) -> impl IntoResponse {
    crate::backend::app_state_wiring::health(state).await
}

pub(crate) fn register_compile_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::graph_compile::register_compile_routes(router)
}

pub(crate) fn register_runtime_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::runtime::register_routes(router)
}

pub(crate) fn register_graph_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::graph_compile::register_graph_routes(router)
}

pub(crate) fn register_graph_quantscript_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::graph_compile::register_graph_quantscript_routes(router)
}

pub(crate) fn register_hotswap_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::ops_governance::register_hotswap_routes(router)
}

pub(crate) fn register_sandbox_verification_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::ops_governance::register_sandbox_verification_routes(router)
}

pub(crate) fn register_alert_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::ops_governance::register_alert_routes(router)
}

pub(crate) fn register_snapshot_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::ops_governance::register_snapshot_routes(router)
}

pub(crate) fn register_runbook_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::ops_governance::register_runbook_routes(router)
}

pub(crate) fn register_chaos_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::ops_governance::register_chaos_routes(router)
}

pub(crate) fn register_strategy_config_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::strategy_config::register_routes(router)
}

pub(crate) fn register_test_scenario_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::test_support::register_test_scenario_routes(router)
}

pub(crate) fn register_credential_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::storage_security::register_credential_routes(router)
}

pub(crate) fn attach_state(router: Router<AppState>, state: AppState) -> Router {
    crate::backend::app_state_wiring::attach_state(router, state)
}
