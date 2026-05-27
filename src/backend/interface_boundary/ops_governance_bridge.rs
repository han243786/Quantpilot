use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.interface_boundary.ops_governance_bridge";

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
