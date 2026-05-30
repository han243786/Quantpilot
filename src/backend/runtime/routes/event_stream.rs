use axum::{routing::get, Router};

use crate::{runtime as runtime_handlers, AppState};

pub const MODULE_ID: &str = "backend.runtime.routes.event_stream";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router.route(
        "/api/runtime/runs/:run_id/events",
        get(runtime_handlers::stream_run_events),
    )
}
