use axum::{
    routing::{get, post},
    Router,
};

use crate::{runtime as runtime_handlers, AppState};

pub const MODULE_ID: &str = "backend.runtime.routes";

pub mod backtest;
pub mod experiment;
pub mod mutation;
pub mod run;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    let router = backtest::register_routes(router);
    let router = run::register_routes(router);

    let router = router
        .route(
            "/api/runtime/runs/:run_id/events",
            get(runtime_handlers::stream_run_events),
        )
        .route(
            "/api/runtime/evidence/health",
            get(runtime_handlers::get_runtime_evidence_health),
        )
        .route(
            "/api/runtime/evidence/cleanup",
            post(runtime_handlers::cleanup_runtime_evidence),
        );

    let router = mutation::register_routes(router);

    let router = router
        .route(
            "/api/runtime/reports",
            get(runtime_handlers::list_runtime_reports)
                .post(runtime_handlers::create_runtime_report),
        )
        .route(
            "/api/runtime/reports/:report_id",
            get(runtime_handlers::get_runtime_report_detail),
        )
        .route(
            "/api/runtime/reports/:report_id/export",
            get(runtime_handlers::export_runtime_report_artifact),
        );

    let router = experiment::register_routes(router);

    router
        .route(
            "/api/v1/merge/records",
            get(runtime_handlers::list_merge_records),
        )
        .route(
            "/api/v1/runtime/generations",
            get(runtime_handlers::list_config_generations),
        )
        .route(
            "/api/v1/storage/health",
            get(runtime_handlers::get_storage_health),
        )
        .route(
            "/api/v1/reports/ops/daily",
            get(runtime_handlers::get_ops_daily_report),
        )
        .route(
            "/api/v1/reports/audit/weekly",
            get(runtime_handlers::get_audit_weekly_report),
        )
        .route(
            "/api/v1/reports/research/monthly",
            get(runtime_handlers::get_research_monthly_report),
        )
}
