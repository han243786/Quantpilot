use axum::routing::{delete, get};
use axum::Router;

use crate::auth::UserId;
use crate::AppState;

mod delete_mutation;
mod key_scope;
mod list_projection;
mod set_mutation;

pub(super) fn register_credential_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/credentials",
            get(list_projection::list_credentials).post(set_mutation::set_credential),
        )
        .route(
            "/api/credentials/:service",
            delete(delete_mutation::delete_credential),
        )
}

/// v2.3.3: 按用户隔离凭证 — vault key 格式为 `{user_id}:{service}`
fn scoped_cv_key(user_id: &UserId, service: &str) -> String {
    key_scope::scoped_cv_key(user_id, service)
}
