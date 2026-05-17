use super::*;

// 路由版本约定: /api/v1/ 用于 Block 5 新增路由 (alerts/snapshots/runbook/chaos),
// 其余路由使用 /api/ 前缀。后续版本统一迁移至 /api/v1/ 前缀。
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
    // 凭证管理路由
    let router = credential_api::register_credential_routes(router);
    // v2.0.0: 多用户认证路由
    let router = auth::register_auth_routes(router);

    // SPA: serve dist/ files, fallback to index.html for client-side routing
    router
        .fallback_service(
            tower_http::services::ServeDir::new("dist")
                .fallback(tower_http::services::ServeFile::new("dist/index.html"))
        )
        .with_state(state)
}

async fn _not_found_fallback() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "not_found",
            "message": "请求的资源不存在"
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
