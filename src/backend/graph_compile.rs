use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.graph_compile";

pub(crate) fn register_compile_routes(router: Router<AppState>) -> Router<AppState> {
    crate::compile_api::register_compile_routes(router)
}

pub(crate) fn register_graph_routes(router: Router<AppState>) -> Router<AppState> {
    crate::graph_api::register_graph_routes(router)
}

pub(crate) fn register_graph_quantscript_routes(router: Router<AppState>) -> Router<AppState> {
    crate::graph_quantscript_api::register_graph_quantscript_routes(router)
}
