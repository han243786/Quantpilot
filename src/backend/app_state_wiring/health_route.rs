use axum::response::IntoResponse;

use crate::AppState;

pub const MODULE_ID: &str = "backend.app_state_wiring.health_route";

pub(crate) async fn health(state: axum::extract::State<AppState>) -> impl IntoResponse {
    crate::app_runtime_helpers::health(state).await
}
