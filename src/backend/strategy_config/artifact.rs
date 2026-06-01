use axum::{http::StatusCode, routing::post, Json, Router};

use crate::{current_time_ms, strategy_config_api, AppState};

pub const MODULE_ID: &str = "backend.strategy_config.artifact";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router.route(
        "/api/v1/strategy-config/artifact",
        post(create_strategy_config_artifact),
    )
}

async fn create_strategy_config_artifact(
    Json(request): Json<strategy_config_api::StrategyConfigArtifactRequest>,
) -> Result<Json<strategy_config_api::StrategyConfigArtifact>, (StatusCode, String)> {
    Ok(Json(strategy_config_api::build_strategy_config_artifact(
        request,
        current_time_ms(),
    )?))
}
