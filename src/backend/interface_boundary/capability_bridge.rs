use axum::response::IntoResponse;

pub const MODULE_ID: &str = "backend.interface_boundary.capability_bridge";

pub(crate) async fn get_capabilities() -> impl IntoResponse {
    crate::backend::capability::get_capabilities().await
}
