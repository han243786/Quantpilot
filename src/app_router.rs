use super::*;

pub(super) fn build_app_router(state: AppState) -> Router {
    let router: Router<AppState> = Router::new()
        .route("/api/health", get(health))
        .route("/api/capabilities", get(get_capabilities));

    let router = register_compile_routes(router);
    let router = register_runtime_routes(router);
    let router = register_graph_routes(router);
    let router = register_graph_quantscript_routes(router);

    router.with_state(state)
}
