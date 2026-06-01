use axum::{http::StatusCode, routing::post, Json, Router};

use crate::{current_time_ms, strategy_config_api, AppState};

pub const MODULE_ID: &str = "backend.strategy_config.artifact";

pub mod schema_model;

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
    Ok(Json(strategy_config_api::build_strategy_config_artifact(
        request,
        current_time_ms(),
    )?))
}
