use axum::{routing::get, Router};

use crate::{runtime as runtime_handlers, AppState};

pub const MODULE_ID: &str = "backend.runtime.routes.report_ops";

pub(crate) fn register_runtime_report_routes(router: Router<AppState>) -> Router<AppState> {
    router
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
        )
}

pub(crate) fn register_ops_routes(router: Router<AppState>) -> Router<AppState> {
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
