use axum::{
    routing::{get, post},
    Router,
};

use crate::{runtime as runtime_handlers, AppState};

pub const MODULE_ID: &str = "backend.runtime.routes.evidence";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/runtime/evidence/health",
            get(runtime_handlers::get_runtime_evidence_health),
        )
        .route(
            "/api/runtime/evidence/cleanup",
            post(runtime_handlers::cleanup_runtime_evidence),
        )
}
