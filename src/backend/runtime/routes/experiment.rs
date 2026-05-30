use axum::{
    routing::{get, post},
    Router,
};

use crate::{runtime as runtime_handlers, AppState};

pub const MODULE_ID: &str = "backend.runtime.routes.experiment";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/runtime/experiments/backtest-sweep",
            post(runtime_handlers::start_backtest_experiment),
        )
        .route(
            "/api/runtime/experiments",
            get(runtime_handlers::list_experiments),
        )
        .route(
            "/api/runtime/experiments/:experiment_id/save",
            post(runtime_handlers::save_experiment_record),
        )
        .route(
            "/api/runtime/experiments/:experiment_id",
            get(runtime_handlers::get_experiment_detail)
                .delete(runtime_handlers::discard_experiment_record),
        )
}
