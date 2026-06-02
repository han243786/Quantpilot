use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.strategy_config.diff";

pub mod artifact_diff;
pub mod evidence_diff;

pub(crate) use artifact_diff::{build_strategy_config_version_diff, StrategyConfigDiffReport};
pub(crate) use evidence_diff::{
    build_strategy_config_evidence_diff_for_backtests, StrategyConfigEvidenceDiffReport,
};
#[cfg(test)]
pub(crate) use evidence_diff::{
    compare_execution_capability_evidence, compare_machine_trajectory_evidence,
    compare_risk_plane_evidence, StrategyConfigEvidenceDiffStatus,
};

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    artifact_diff::register_routes(router)
}
