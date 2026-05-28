use axum::{
    routing::{get, post},
    Router,
};

use crate::{runtime as runtime_handlers, AppState};

pub const MODULE_ID: &str = "backend.runtime.routes";

pub mod backtest;
pub mod run;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    let router = backtest::register_routes(router);
    let router = run::register_routes(router);

    router
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
        )
        .route(
            "/api/runtime/mutations",
            get(runtime_handlers::list_runtime_parameter_mutations)
                .post(runtime_handlers::create_runtime_parameter_mutation),
        )
        .route(
            "/api/runtime/mutations/:proposal_id",
            get(runtime_handlers::get_runtime_parameter_mutation_detail),
        )
        .route(
            "/api/runtime/mutations/:proposal_id/activate",
            post(runtime_handlers::activate_runtime_parameter_mutation),
        )
        .route(
            "/api/runtime/mutations/:proposal_id/rollback",
            post(runtime_handlers::rollback_runtime_parameter_mutation),
        )
        .route(
            "/api/runtime/ai-proposals",
            get(runtime_handlers::list_runtime_ai_proposals)
                .post(runtime_handlers::create_runtime_ai_proposal),
        )
        .route(
            "/api/runtime/ai-proposals/:ai_proposal_id",
            get(runtime_handlers::get_runtime_ai_proposal_detail),
        )
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
        .route(
            "/api/v1/ai/approvals",
            get(runtime_handlers::list_runtime_approvals),
        )
        .route(
            "/api/v1/ai/approvals/:approval_id",
            get(runtime_handlers::get_runtime_approval_detail),
        )
        .route(
            "/api/v1/ai/proposals/:proposal_id/approve",
            post(runtime_handlers::approve_ai_proposal),
        )
        .route(
            "/api/v1/ai/proposals/:proposal_id/reject",
            post(runtime_handlers::reject_ai_proposal),
        )
        .route(
            "/api/v1/ai/proposals/:proposal_id/claim",
            post(runtime_handlers::claim_ai_proposal_review),
        )
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
