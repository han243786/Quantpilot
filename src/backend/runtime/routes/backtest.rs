use axum::{
    routing::{get, post},
    Router,
};

use crate::{backtest_compare, runtime as runtime_handlers, AppState};

pub const MODULE_ID: &str = "backend.runtime.routes.backtest";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/runtime/backtest",
            post(runtime_handlers::start_backtest_run),
        )
        .route(
            "/api/runtime/backtests",
            get(runtime_handlers::list_backtests),
        )
        .route(
            "/api/runtime/backtests/compare",
            post(backtest_compare::compare_backtests),
        )
        .route(
            "/api/runtime/backtests/:backtest_id/save",
            post(runtime_handlers::save_backtest_record),
        )
        .route(
            "/api/runtime/backtests/:backtest_id",
            get(runtime_handlers::get_backtest_detail)
                .delete(runtime_handlers::discard_backtest_record),
        )
        .route(
            "/api/runtime/backtests/:backtest_id/replay",
            get(runtime_handlers::get_backtest_replay),
        )
}
