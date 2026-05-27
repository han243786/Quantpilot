use axum::response::IntoResponse;

pub const MODULE_ID: &str = "backend.capability";

pub mod snapshot;

pub(crate) async fn get_capabilities() -> impl IntoResponse {
    snapshot::get_capabilities().await
}
