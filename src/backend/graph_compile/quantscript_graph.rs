use axum::Router;

use crate::AppState;

pub const MODULE_ID: &str = "backend.graph_compile.quantscript_graph";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    crate::graph_quantscript_api::register_graph_quantscript_routes(router)
}
