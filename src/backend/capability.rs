use axum::response::IntoResponse;

pub const MODULE_ID: &str = "backend.capability";

pub(crate) async fn get_capabilities() -> impl IntoResponse {
    crate::capability_api::get_capabilities().await
}
