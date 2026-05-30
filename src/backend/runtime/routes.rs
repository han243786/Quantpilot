use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.runtime.routes";

pub mod backtest;
pub mod event_stream;
pub mod evidence;
pub mod experiment;
pub mod mutation;
pub mod report_ops;
pub mod run;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    let router = backtest::register_routes(router);
    let router = run::register_routes(router);

    let router = event_stream::register_routes(router);
    let router = evidence::register_routes(router);
    let router = mutation::register_routes(router);

    let router = report_ops::register_runtime_report_routes(router);
    let router = experiment::register_routes(router);

    report_ops::register_ops_routes(router)
}
