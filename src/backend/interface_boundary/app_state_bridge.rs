use axum::{response::IntoResponse, Router};

use crate::AppState;

pub const MODULE_ID: &str = "backend.interface_boundary.app_state_bridge";

pub(crate) async fn health(state: axum::extract::State<AppState>) -> impl IntoResponse {
    crate::backend::app_state_wiring::health(state).await
}

pub(crate) fn attach_state(router: Router<AppState>, state: AppState) -> Router {
    crate::backend::app_state_wiring::attach_state(router, state)
}
