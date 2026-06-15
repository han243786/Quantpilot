/// v4.7.0: QuantPilot 实时执行端
/// 双切面执行端 — PaperSimulated 拉取 OKX 公共行情并本地模拟, PaperActual 仅接 OKX demo 回执
mod api_guard;
mod audit_log;
mod credential_vault_v2;
mod executor_state;
mod kline_buffer;
mod live_runner;
mod market_stream_routes;
mod migration_api;
pub mod okx_rest;
mod provider_order_routes;
mod provider_order_support;
mod state_routes;
mod strategy_lifecycle_routes;
mod v4_deploy_support;
mod ws_client;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use executor_state::{ExecutorState, TriggerEvent};
use live_runner::RunnerPool;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

const EXECUTOR_PORT: u16 = 3001;

#[tokio::main]
async fn main() {
    println!(
        "[executor] QuantPilot 实时执行端 v{} 启动中...",
        env!("CARGO_PKG_VERSION")
    );

    if let Err(e) = qrpc_session::init_session_key() {
        eprintln!("[executor] 会话密钥初始化失败: {} (测试端将无法连接)", e);
    } else {
        println!("[executor] 会话密钥已生成");
    }

    let state = ExecutorState::load_default_or_new();

    // v3.7.0/v4.8.0: 广播通道 (SSE trigger 推送) + OKX 公共行情事件通道
    let (trigger_broadcast, _) = broadcast::channel::<TriggerEvent>(256);
    let (okx_tx, mut okx_rx) = mpsc::unbounded_channel::<crate::ws_client::WsEvent>();

    let mut pool = RunnerPool::new(trigger_broadcast.clone());
    pool.register_exchange("okx", okx_tx.clone());
    state
        .ws_tx_map
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .insert("okx".into(), okx_tx.clone());

    let pool = Arc::new(std::sync::Mutex::new(pool));
    *state.runner_pool.lock().unwrap_or_else(|e| e.into_inner()) = Some(pool.clone());

    // v3.7.x: 存储JoinHandle用于shutdown时abort, 防止幽灵任务
    let _event_handle = tokio::spawn(async move {
        while let Some(event) = okx_rx.recv().await {
            if let Ok(mut guard) = pool.lock() {
                guard.broadcast_ws_event(event);
            }
        }
    });
    let _feed_handle = tokio::spawn(crate::ws_client::run_okx_public_market_feed(
        okx_tx,
        crate::ws_client::okx_public_feed_symbols_from_env(),
    ));

    let app = Router::new()
        .route("/api/executor/health", get(state_routes::health_check))
        .route(
            "/api/executor/strategies",
            get(strategy_lifecycle_routes::list_strategies)
                .post(strategy_lifecycle_routes::recv_strategy),
        )
        .route(
            "/api/executor/strategies/:strategy_id",
            get(strategy_lifecycle_routes::get_strategy_detail),
        )
        .route(
            "/api/executor/strategies/:strategy_id/start",
            post(strategy_lifecycle_routes::start_strategy),
        )
        .route(
            "/api/executor/strategies/:strategy_id/stop",
            post(strategy_lifecycle_routes::stop_strategy),
        )
        .route(
            "/api/executor/strategies/:strategy_id/klines",
            get(market_stream_routes::get_klines),
        )
        .route(
            "/api/executor/strategies/:strategy_id/events",
            get(market_stream_routes::strategy_events_sse),
        )
        .route(
            "/api/executor/params/:strategy_id",
            get(state_routes::get_params).post(state_routes::update_params),
        )
        .route(
            "/api/executor/mode",
            get(state_routes::get_mode).post(state_routes::set_mode),
        )
        .route(
            "/api/executor/provider/okx-demo/orders",
            post(provider_order_routes::submit_okx_demo_order),
        )
        .route(
            "/api/executor/provider/okx-demo/orders/query",
            post(provider_order_routes::query_okx_demo_order),
        )
        .route(
            "/api/executor/provider/okx-demo/orders/cancel",
            post(provider_order_routes::cancel_okx_demo_order),
        )
        // v3.3.0: API守卫中间件 (Phase 4透传, Phase 5启用HMAC验证)
        .layer(axum::middleware::from_fn(
            crate::api_guard::api_guard_middleware,
        ))
        .with_state(state);

    let addr = format!("127.0.0.1:{}", EXECUTOR_PORT);
    let listener = TcpListener::bind(&addr).await.unwrap_or_else(|e| {
        eprintln!("[executor] 绑定 {} 失败: {}", addr, e);
        std::process::exit(1);
    });
    println!("[executor] ✓ 实时执行端已就绪 → {} (双执行切面)", addr);
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("[executor] 服务运行失败: {}", e);
        std::process::exit(1);
    }
}

// ── 端点实现 ──

fn internal_error(message: String) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        serde_json::json!({
            "error": "executor_internal_error",
            "message": message,
        })
        .to_string(),
    )
}

fn append_audit(
    state: &ExecutorState,
    operation: &str,
    strategy_id: Option<String>,
    details: serde_json::Value,
) {
    state.audit_log.append(&audit_log::AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: operation.to_string(),
        actor: "executor_api".to_string(),
        strategy_id,
        details,
    });
}

#[cfg(test)]
mod tests {
    use super::executor_state::{ActiveStrategy, ExecutionMode, RuntimeKind, StrategyStatus};
    use super::v4_deploy_support::{empty_core_ir, ensure_strategy_config_preflight_allows_start};
    use axum::http::StatusCode;
    use std::collections::BTreeMap;

    fn sample_v4_strategy(preflight: Option<serde_json::Value>) -> ActiveStrategy {
        ActiveStrategy {
            strategy_id: "strategy_preflight_test".to_string(),
            name: "Strategy preflight test".to_string(),
            runtime_kind: RuntimeKind::V4,
            core_ir: empty_core_ir("strategy_preflight_test"),
            v4_graph: None,
            graph_json: serde_json::Value::Null,
            params: BTreeMap::new(),
            status: StrategyStatus::Loaded,
            subscribed_symbols: vec![],
            execution_mode: ExecutionMode::PaperSimulated,
            strategy_config_preflight: preflight,
        }
    }

    fn preflight_report(
        mode_label: &str,
        decision: &str,
        can_paper_simulated: bool,
        can_paper_actual_demo: bool,
        live_execution_allowed: bool,
        allowed_actions: Vec<&str>,
    ) -> serde_json::Value {
        serde_json::json!({
            "schema_version": "quantpilot/v4-strategy-config-preflight/v1",
            "decision": decision,
            "can_paper_simulated": can_paper_simulated,
            "can_paper_actual_demo": can_paper_actual_demo,
            "can_live_execution": live_execution_allowed,
            "allowed_actions": allowed_actions,
            "artifact": {
                "artifact_digest": "sha256:test",
                "runtime_boundary": {
                    "mode_label": mode_label,
                    "live_execution_allowed": live_execution_allowed
                }
            }
        })
    }

    #[test]
    fn v4_start_requires_strategy_config_preflight() {
        let strategy = sample_v4_strategy(None);
        let error = ensure_strategy_config_preflight_allows_start(&strategy).unwrap_err();
        assert_eq!(error.0, StatusCode::LOCKED);
        assert!(error.1.contains("strategy_config_preflight_missing"));
    }

    #[test]
    fn v4_start_rejects_blocked_or_live_execution_preflight() {
        let blocked = sample_v4_strategy(Some(preflight_report(
            "PaperSimulated",
            "blocked",
            false,
            false,
            false,
            vec![],
        )));
        let error = ensure_strategy_config_preflight_allows_start(&blocked).unwrap_err();
        assert_eq!(error.0, StatusCode::LOCKED);
        assert!(error.1.contains("strategy_config_preflight_blocked"));

        let live_allowed = sample_v4_strategy(Some(preflight_report(
            "PaperSimulated",
            "ready",
            true,
            false,
            true,
            vec!["start_paper_simulated"],
        )));
        let error = ensure_strategy_config_preflight_allows_start(&live_allowed).unwrap_err();
        assert!(error.1.contains("strategy_config_live_execution_forbidden"));
    }

    #[test]
    fn v4_start_accepts_matching_paper_simulated_preflight() {
        let strategy = sample_v4_strategy(Some(preflight_report(
            "PaperSimulated",
            "restricted",
            true,
            false,
            false,
            vec!["compile", "start_paper_simulated"],
        )));
        ensure_strategy_config_preflight_allows_start(&strategy).unwrap();
    }

    #[test]
    fn v4_paper_actual_start_requires_matching_demo_preflight() {
        let mut strategy = sample_v4_strategy(Some(preflight_report(
            "PaperSimulated",
            "ready",
            true,
            false,
            false,
            vec!["start_paper_simulated"],
        )));
        strategy.execution_mode = ExecutionMode::PaperActual;
        let error = ensure_strategy_config_preflight_allows_start(&strategy).unwrap_err();
        assert!(error
            .1
            .contains("strategy_config_runtime_boundary_mismatch"));

        strategy.strategy_config_preflight = Some(preflight_report(
            "PaperActual",
            "ready",
            true,
            true,
            false,
            vec!["start_paper_actual_demo"],
        ));
        ensure_strategy_config_preflight_allows_start(&strategy).unwrap();
    }
}
