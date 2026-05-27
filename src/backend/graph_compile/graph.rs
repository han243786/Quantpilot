use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.graph_compile.graph";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    crate::graph_api::register_graph_routes(router)
}
