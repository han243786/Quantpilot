use axum::{response::IntoResponse, Router};

use crate::AppState;

pub const MODULE_ID: &str = "backend.app_state_wiring";

pub use crate::app_runtime_helpers::new_app_state;

pub(crate) async fn health(state: axum::extract::State<AppState>) -> impl IntoResponse {
    crate::app_runtime_helpers::health(state).await
}

pub(crate) fn attach_state(router: Router<AppState>, state: AppState) -> Router {
    router.with_state(state)
}
