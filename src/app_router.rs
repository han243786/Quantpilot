use super::*;
use crate::backend::interface_boundary;

// 路由版本约定: /api/v1/ 用于 Block 5 新增路由 (alerts/snapshots/runbook/chaos),
// 其余路由使用 /api/ 前缀。后续版本统一迁移至 /api/v1/ 前缀。
pub fn build_app_router(state: AppState) -> Router {
    let router: Router<AppState> = Router::new()
        .route("/api/health", get(interface_boundary::health))
        .route(
            "/api/capabilities",
            get(interface_boundary::get_capabilities),
        );

    let router = interface_boundary::register_compile_routes(router);
    // v3.0.0: 部署策略到执行端
    let router = router.route(
        "/api/executor/strategies",
        axum::routing::post(crate::migration_sender::deploy_strategy),
    );
    let router = interface_boundary::register_runtime_routes(router);
    let router = interface_boundary::register_graph_routes(router);
    let router = interface_boundary::register_graph_quantscript_routes(router);
    let router = interface_boundary::register_hotswap_routes(router);
    // Block 5 新路由
    let router = interface_boundary::register_sandbox_verification_routes(router);
    let router = interface_boundary::register_alert_routes(router);
    let router = interface_boundary::register_snapshot_routes(router);
    let router = interface_boundary::register_runbook_routes(router);
    let router = interface_boundary::register_chaos_routes(router);
    let router = interface_boundary::register_strategy_config_routes(router);
    // 测试场景路由
    let router = interface_boundary::register_test_scenario_routes(router);
    // 凭证管理路由
    let router = interface_boundary::register_credential_routes(router);
    // v2.0.0: 多用户认证路由
    let router = auth::register_auth_routes(router);
    let router = router.route("/api/*path", axum::routing::any(_not_found_fallback));

    // SPA: serve dist/ files, fallback to index.html for client-side routing
    let router = router.fallback_service(
        tower_http::services::ServeDir::new("dist")
            .fallback(tower_http::services::ServeFile::new("dist/index.html")),
    );

    interface_boundary::attach_state(router, state)
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
