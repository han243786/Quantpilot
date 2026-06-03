use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.ops_governance.sandbox";

mod comparison_metrics;
mod handlers;
mod metrics_evaluation;
mod proposal_loader;
mod report_api;
mod verification_run;

pub(crate) use handlers::load_sandbox_report_from_disk;
pub(crate) use verification_run::run_sandbox_verification;

use comparison_metrics::compute_comparison_metrics;
use metrics_evaluation::{
    compute_metrics_diff, compute_sandbox_warnings, determine_sandbox_verdict,
};
use proposal_loader::load_or_fetch_ai_proposal;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    report_api::register_routes(router)
}
