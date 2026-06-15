use axum::{
    routing::{get, post},
    Router,
};

use crate::{runtime as runtime_handlers, AppState};

pub const MODULE_ID: &str = "backend.runtime.routes.run";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/runtime/v4/run",
            post(runtime_handlers::start_v4_runtime_run),
        )
        .route("/api/runtime/runs", get(runtime_handlers::list_runs))
        .route(
            "/api/runtime/runs/:run_id/save",
            post(runtime_handlers::save_run_record),
        )
        .route(
            "/api/runtime/runs/:run_id",
            get(runtime_handlers::get_run_detail).delete(runtime_handlers::discard_run_record),
        )
        .route(
            "/api/runtime/runs/:run_id/replay",
            get(runtime_handlers::get_run_replay),
        )
        .route(
            "/api/runtime/runs/:run_id/status",
            get(runtime_handlers::get_run_status),
        )
}
