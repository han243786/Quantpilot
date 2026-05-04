use super::*;

pub(super) fn build_app_router(state: AppState) -> Router {
    let router: Router<AppState> = Router::new()
        .route("/api/health", get(health))
        .route("/api/capabilities", get(get_capabilities));

    let router = register_compile_routes(router);
    let router = register_runtime_routes(router);
    let router = register_graph_routes(router);
    let router = register_graph_quantscript_routes(router);
    let router = register_hotswap_routes(router);
    // Block 5 新路由
    let router = register_sandbox_verification_routes(router);
    let router = register_alert_routes(router);
    let router = register_snapshot_routes(router);
    let router = register_runbook_routes(router);
    let router = register_chaos_routes(router);
    // 测试场景路由
    let router = register_test_scenario_routes(router);

    router
        .fallback(not_found_fallback)
        .with_state(state)
}

async fn not_found_fallback() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "type": "https://quantpilot.dev/problems/not-found",
            "title": "Not Found",
            "status": 404,
            "detail": "The requested endpoint does not exist."
        })),
    )
}

fn register_hotswap_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/hotswap", post(hotswap_api::submit_hotswap))
        .route("/api/hotswap/list", get(hotswap_api::list_hotswaps))
        .route("/api/hotswap/{hotswap_id}", get(hotswap_api::get_hotswap_status))
}

// Block 5 路由注册

fn register_sandbox_verification_routes(router: Router<AppState>) -> Router<AppState> {
    sandbox_verification::register_sandbox_verification_routes(router)
}

fn register_alert_routes(router: Router<AppState>) -> Router<AppState> {
    alert_engine::register_alert_routes(router)
}

fn register_snapshot_routes(router: Router<AppState>) -> Router<AppState> {
    snapshot_service::register_snapshot_routes(router)
}

fn register_runbook_routes(router: Router<AppState>) -> Router<AppState> {
    runbook::register_runbook_routes(router)
}

fn register_chaos_routes(router: Router<AppState>) -> Router<AppState> {
    chaos_experiment::register_chaos_routes(router)
}

fn register_test_scenario_routes(router: Router<AppState>) -> Router<AppState> {
    api_test_scenario::register_test_scenario_routes(router)
}
