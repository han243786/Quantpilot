use axum::{response::IntoResponse, Router};

use crate::AppState;

pub const MODULE_ID: &str = "backend.interface_boundary";

pub mod app_state_bridge;
pub mod capability_bridge;
pub mod graph_compile_bridge;
pub mod ops_governance_bridge;
pub mod runtime_bridge;
pub mod storage_security_bridge;
pub mod strategy_config_bridge;
pub mod test_support_bridge;

pub use crate::app_router::build_app_router;

pub(crate) async fn get_capabilities() -> impl IntoResponse {
    capability_bridge::get_capabilities().await
}

pub(crate) async fn health(state: axum::extract::State<AppState>) -> impl IntoResponse {
    app_state_bridge::health(state).await
}

pub(crate) fn register_compile_routes(router: Router<AppState>) -> Router<AppState> {
    graph_compile_bridge::register_compile_routes(router)
}

pub(crate) fn register_runtime_routes(router: Router<AppState>) -> Router<AppState> {
    runtime_bridge::register_routes(router)
}

pub(crate) fn register_graph_routes(router: Router<AppState>) -> Router<AppState> {
    graph_compile_bridge::register_graph_routes(router)
}

pub(crate) fn register_graph_quantscript_routes(router: Router<AppState>) -> Router<AppState> {
    graph_compile_bridge::register_graph_quantscript_routes(router)
}

pub(crate) fn register_hotswap_routes(router: Router<AppState>) -> Router<AppState> {
    ops_governance_bridge::register_hotswap_routes(router)
}

pub(crate) fn register_sandbox_verification_routes(router: Router<AppState>) -> Router<AppState> {
    ops_governance_bridge::register_sandbox_verification_routes(router)
}

pub(crate) fn register_alert_routes(router: Router<AppState>) -> Router<AppState> {
    ops_governance_bridge::register_alert_routes(router)
}

pub(crate) fn register_snapshot_routes(router: Router<AppState>) -> Router<AppState> {
    ops_governance_bridge::register_snapshot_routes(router)
}

pub(crate) fn register_runbook_routes(router: Router<AppState>) -> Router<AppState> {
    ops_governance_bridge::register_runbook_routes(router)
}

pub(crate) fn register_chaos_routes(router: Router<AppState>) -> Router<AppState> {
    ops_governance_bridge::register_chaos_routes(router)
}

pub(crate) fn register_strategy_config_routes(router: Router<AppState>) -> Router<AppState> {
    strategy_config_bridge::register_routes(router)
}

pub(crate) fn register_test_scenario_routes(router: Router<AppState>) -> Router<AppState> {
    test_support_bridge::register_test_scenario_routes(router)
}

pub(crate) fn register_credential_routes(router: Router<AppState>) -> Router<AppState> {
    storage_security_bridge::register_credential_routes(router)
}

pub(crate) fn attach_state(router: Router<AppState>, state: AppState) -> Router {
    app_state_bridge::attach_state(router, state)
}
