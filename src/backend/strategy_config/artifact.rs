use axum::{http::StatusCode, routing::post, Json, Router};

use crate::{current_time_ms, AppState};

pub const MODULE_ID: &str = "backend.strategy_config.artifact";

pub mod builder_core;
pub mod domain_projection;
pub mod schema_model;

#[cfg(test)]
pub(crate) use builder_core::STRATEGY_CONFIG_ARTIFACT_SCHEMA;
pub(crate) use builder_core::{
    build_strategy_config_artifact, non_empty, version_artifact_request,
};
pub(crate) use domain_projection::{build_config_domains, finding};
pub(crate) use schema_model::*;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router.route(
        "/api/v1/strategy-config/artifact",
        post(create_strategy_config_artifact),
    )
}

async fn create_strategy_config_artifact(
    Json(request): Json<StrategyConfigArtifactRequest>,
) -> Result<Json<StrategyConfigArtifact>, (StatusCode, String)> {
    Ok(Json(build_strategy_config_artifact(
        request,
        current_time_ms(),
    )?))
}
