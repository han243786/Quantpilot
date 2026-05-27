use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.interface_boundary.graph_compile_bridge";

pub(crate) fn register_compile_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::graph_compile::register_compile_routes(router)
}

pub(crate) fn register_graph_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::graph_compile::register_graph_routes(router)
}

pub(crate) fn register_graph_quantscript_routes(router: Router<AppState>) -> Router<AppState> {
    crate::backend::graph_compile::register_graph_quantscript_routes(router)
}
