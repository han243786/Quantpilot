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
mod state_routes;
mod strategy_lifecycle_routes;
mod v4_deploy_support;
mod ws_client;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use executor_state::{ExecutionMode, ExecutorState, TriggerEvent};
use live_runner::RunnerPool;
use okx_rest::{OkxOrderRequest, OKX_DEMO_AUDIT_ENVIRONMENT, OKX_DEMO_PROVIDER_KEY};
use std::path::PathBuf;
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

#[derive(Debug, serde::Deserialize)]
struct OkxDemoOrderSubmitRequest {
    #[serde(default)]
    strategy_id: Option<String>,
    inst_id: String,
    side: String,
    sz: String,
    #[serde(default = "default_okx_td_mode")]
    td_mode: String,
    #[serde(default = "default_okx_order_type")]
    ord_type: String,
    #[serde(default)]
    px: Option<String>,
    #[serde(default)]
    cl_ord_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct OkxDemoOrderLookupRequest {
    #[serde(default)]
    strategy_id: Option<String>,
    inst_id: String,
    #[serde(default)]
    ord_id: Option<String>,
    #[serde(default)]
    cl_ord_id: Option<String>,
}

#[derive(Debug, Clone)]
struct OkxDemoCredentialSet {
    api_key: String,
    secret: String,
    passphrase: String,
    source: &'static str,
}

fn default_okx_td_mode() -> String {
    "cash".to_string()
}

fn default_okx_order_type() -> String {
    "limit".to_string()
}

fn ensure_okx_demo_provider_mode(
    state: &ExecutorState,
    strategy_id: Option<&str>,
) -> Result<(), (StatusCode, String)> {
    if state.current_mode() != ExecutionMode::PaperActual {
        return Err((
            StatusCode::LOCKED,
            serde_json::json!({
                "error": "paper_actual_required",
                "message": "OKX 模拟盘 provider 回执路径只允许在 paper_actual 模式调用",
                "required_mode": "paper_actual",
                "current_mode": state.current_mode().as_str(),
                "environment": OKX_DEMO_AUDIT_ENVIRONMENT,
            })
            .to_string(),
        ));
    }
    if let Some(strategy_id) = strategy_id {
        let strategies = state
            .strategies
            .read()
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", error)))?;
        let strategy = strategies.get(strategy_id).ok_or((
            StatusCode::NOT_FOUND,
            format!("策略不存在: {}", strategy_id),
        ))?;
        if strategy.execution_mode != ExecutionMode::PaperActual {
            return Err((
                StatusCode::LOCKED,
                serde_json::json!({
                    "error": "strategy_paper_actual_required",
                    "message": "该策略不是 OKX 模拟盘 / 非真实资金 execution_mode",
                    "strategy_id": strategy_id,
                    "strategy_mode": strategy.execution_mode.as_str(),
                    "required_mode": "paper_actual",
                })
                .to_string(),
            ));
        }
    }
    Ok(())
}

fn build_okx_demo_order_request(
    req: &OkxDemoOrderSubmitRequest,
) -> Result<OkxOrderRequest, (StatusCode, String)> {
    let inst_id = clean_required_ascii(&req.inst_id, "inst_id", valid_okx_inst_char)?;
    let side = clean_enum(&req.side, "side", &["buy", "sell"])?;
    let ord_type = clean_enum(&req.ord_type, "ord_type", &["market", "limit"])?;
    let td_mode = clean_enum(&req.td_mode, "td_mode", &["cash"])?;
    let sz = clean_positive_decimal(&req.sz, "sz")?;
    let px = match req.px.as_deref() {
        Some(value) => Some(clean_positive_decimal(value, "px")?),
        None if ord_type == "limit" => {
            return Err((
                StatusCode::BAD_REQUEST,
                "OKX 模拟盘限价单必须提供 px".to_string(),
            ))
        }
        None => None,
    };
    let cl_ord_id = match req.cl_ord_id.as_deref() {
        Some(value) => Some(clean_required_ascii(value, "cl_ord_id", valid_okx_id_char)?),
        None => Some(default_okx_client_order_id()),
    };

    Ok(OkxOrderRequest {
        inst_id,
        td_mode,
        side,
        ord_type,
        sz,
        cl_ord_id,
        px,
    })
}

fn clean_required_ascii(
    value: &str,
    field: &str,
    valid: fn(char) -> bool,
) -> Result<String, (StatusCode, String)> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.chars().all(valid) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("{} 不能为空，且包含不允许的字符", field),
        ));
    }
    Ok(trimmed.to_string())
}

fn clean_enum(value: &str, field: &str, allowed: &[&str]) -> Result<String, (StatusCode, String)> {
    let normalized = value.trim().to_ascii_lowercase();
    if allowed.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            format!("{} 只允许: {}", field, allowed.join(", ")),
        ))
    }
}

fn clean_positive_decimal(value: &str, field: &str) -> Result<String, (StatusCode, String)> {
    let trimmed = value.trim();
    let parsed = trimmed
        .parse::<f64>()
        .map_err(|_| (StatusCode::BAD_REQUEST, format!("{} 必须是正数", field)))?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err((StatusCode::BAD_REQUEST, format!("{} 必须是有限正数", field)));
    }
    Ok(trimmed.to_string())
}

fn valid_okx_inst_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-'
}

fn valid_okx_id_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
}

fn default_okx_client_order_id() -> String {
    format!("qpw02{}", now_ms())
        .chars()
        .take(32)
        .collect::<String>()
}

fn load_okx_demo_credentials() -> Result<OkxDemoCredentialSet, (StatusCode, String)> {
    if let Some(credentials) = credentials_from_env() {
        return Ok(credentials);
    }
    credentials_from_executor_vault().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            serde_json::json!({
                "error": "missing_okx_demo_credentials",
                "message": "OKX 模拟盘需要配置 QUANTPILOT_OKX_DEMO_API_KEY/SECRET/PASSPHRASE，或执行端凭证保险库服务 okx_demo，或旧主凭证库标签 0:okx_test / 0:regtest_okx",
                "environment": OKX_DEMO_AUDIT_ENVIRONMENT,
            })
            .to_string(),
        )
    })
}

fn credentials_from_env() -> Option<OkxDemoCredentialSet> {
    let api_key = std::env::var("QUANTPILOT_OKX_DEMO_API_KEY").ok()?;
    let secret = std::env::var("QUANTPILOT_OKX_DEMO_SECRET").ok()?;
    let passphrase = std::env::var("QUANTPILOT_OKX_DEMO_PASSPHRASE").ok()?;
    if api_key.trim().is_empty() || secret.trim().is_empty() || passphrase.trim().is_empty() {
        return None;
    }
    Some(OkxDemoCredentialSet {
        api_key,
        secret,
        passphrase,
        source: "env",
    })
}

fn credentials_from_executor_vault() -> Option<OkxDemoCredentialSet> {
    if let Ok(vault) = credential_vault_v2::ExecutorCredentialVault::load(&executor_storage_dir()) {
        for service in ["okx_demo", "okx"] {
            let Ok(fields) = vault.get_service(service) else {
                continue;
            };
            let Some(api_key) = fields.get("api_key").or_else(|| fields.get("key")).cloned() else {
                continue;
            };
            let Some(secret) = fields.get("secret").cloned() else {
                continue;
            };
            let Some(passphrase) = fields.get("passphrase").cloned() else {
                continue;
            };
            if !api_key.trim().is_empty()
                && !secret.trim().is_empty()
                && !passphrase.trim().is_empty()
            {
                return Some(OkxDemoCredentialSet {
                    api_key,
                    secret,
                    passphrase,
                    source: "executor_vault",
                });
            }
        }
    }
    credentials_from_legacy_vault()
}

fn credentials_from_legacy_vault() -> Option<OkxDemoCredentialSet> {
    let vault = quantpilot::credential_vault::CredentialVault::load().ok()?;
    quantpilot::safe_log::register_credential_patterns(vault.extract_secret_patterns());
    for service in ["okx_demo", "okx", "0:okx_test", "0:regtest_okx"] {
        let Some(fields) = vault.get_service(service) else {
            continue;
        };
        let Some(api_key) = fields
            .get("api_key")
            .or_else(|| fields.get("key"))
            .map(|value| value.as_str().to_string())
        else {
            continue;
        };
        let Some(secret) = fields.get("secret").map(|value| value.as_str().to_string()) else {
            continue;
        };
        let Some(passphrase) = fields
            .get("passphrase")
            .map(|value| value.as_str().to_string())
        else {
            continue;
        };
        if !api_key.trim().is_empty() && !secret.trim().is_empty() && !passphrase.trim().is_empty()
        {
            return Some(OkxDemoCredentialSet {
                api_key,
                secret,
                passphrase,
                source: "legacy_vault",
            });
        }
    }
    None
}

fn executor_storage_dir() -> PathBuf {
    std::env::var_os("QUANTPILOT_STORAGE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("storage"))
}

fn okx_provider_error(action: &str, error: anyhow::Error) -> (StatusCode, String) {
    let message = quantpilot::safe_log::sanitize_secrets(&format!("{:#}", error));
    (
        StatusCode::BAD_GATEWAY,
        serde_json::json!({
            "error": "okx_demo_provider_error",
            "action": action,
            "message": message,
            "provider": OKX_DEMO_PROVIDER_KEY,
            "environment": OKX_DEMO_AUDIT_ENVIRONMENT,
            "simulated_trading": true,
        })
        .to_string(),
    )
}

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

fn okx_demo_order_audit_payload(
    strategy_id: Option<&str>,
    order: &OkxOrderRequest,
) -> serde_json::Value {
    serde_json::json!({
        "strategy_id": strategy_id,
        "inst_id": &order.inst_id,
        "td_mode": &order.td_mode,
        "side": &order.side,
        "ord_type": &order.ord_type,
        "sz": &order.sz,
        "px": &order.px,
        "cl_ord_id": &order.cl_ord_id,
    })
}

fn okx_demo_lookup_audit_payload(
    strategy_id: Option<&str>,
    req: &OkxDemoOrderLookupRequest,
) -> serde_json::Value {
    serde_json::json!({
        "strategy_id": strategy_id,
        "inst_id": &req.inst_id,
        "ord_id": &req.ord_id,
        "cl_ord_id": &req.cl_ord_id,
    })
}

fn okx_demo_provider_audit_details(
    action: &str,
    credential_source: &str,
    request: serde_json::Value,
    provider_response: &serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "provider": OKX_DEMO_PROVIDER_KEY,
        "environment": OKX_DEMO_AUDIT_ENVIRONMENT,
        "simulated_trading": true,
        "demo_flag": "1",
        "simulated_trading_header": "x-simulated-trading=1",
        "action": action,
        "credential_source": credential_source,
        "request": request,
        "provider_result": {
            "code": provider_response.get("code").and_then(|value| value.as_str()).unwrap_or("unknown"),
            "first_order_id": provider_response
                .get("data")
                .and_then(|value| value.as_array())
                .and_then(|items| items.first())
                .and_then(|item| item.get("ordId"))
                .and_then(|value| value.as_str()),
            "first_state": provider_response
                .get("data")
                .and_then(|value| value.as_array())
                .and_then(|items| items.first())
                .and_then(|item| item.get("state"))
                .and_then(|value| value.as_str()),
        }
    })
}

fn okx_demo_provider_response(
    status: &str,
    strategy_id: Option<&str>,
    provider_response: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "status": status,
        "strategy_id": strategy_id,
        "provider": OKX_DEMO_PROVIDER_KEY,
        "environment": OKX_DEMO_AUDIT_ENVIRONMENT,
        "simulated_trading": true,
        "demo_flag": "1",
        "provider_response": provider_response,
    })
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
    use super::executor_state::{
        ActiveStrategy, ExecutionMode, ExecutorState, RuntimeKind, StrategyStatus,
    };
    use super::v4_deploy_support::{empty_core_ir, ensure_strategy_config_preflight_allows_start};
    use super::*;
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

    #[test]
    fn okx_demo_provider_route_requires_paper_actual_mode() {
        let state = ExecutorState::new();
        let error = ensure_okx_demo_provider_mode(&state, None).unwrap_err();
        assert_eq!(error.0, StatusCode::LOCKED);
        assert!(error.1.contains("paper_actual"));

        state.set_mode(ExecutionMode::PaperActual);
        ensure_okx_demo_provider_mode(&state, None).unwrap();
    }

    #[test]
    fn okx_demo_order_request_validation_keeps_provider_specific_shape() {
        let request = OkxDemoOrderSubmitRequest {
            strategy_id: Some("s1".to_string()),
            inst_id: "BTC-USDT".to_string(),
            side: "BUY".to_string(),
            sz: "0.001".to_string(),
            td_mode: "cash".to_string(),
            ord_type: "limit".to_string(),
            px: Some("70000".to_string()),
            cl_ord_id: Some("qp_w0_2".to_string()),
        };

        let order = build_okx_demo_order_request(&request).unwrap();
        let value = serde_json::to_value(order).unwrap();
        assert_eq!(value["instId"], "BTC-USDT");
        assert_eq!(value["tdMode"], "cash");
        assert_eq!(value["side"], "buy");
        assert_eq!(value["ordType"], "limit");
        assert_eq!(value["clOrdId"], "qp_w0_2");
        assert_eq!(value["px"], "70000");
    }

    #[test]
    fn okx_demo_audit_details_never_include_credentials_or_signatures() {
        let request = serde_json::json!({
            "inst_id": "BTC-USDT",
            "side": "buy",
            "ord_type": "limit",
            "sz": "0.001",
            "px": "70000",
        });
        let provider_response = serde_json::json!({
            "code": "0",
            "data": [{"ordId": "123", "state": "live"}],
        });

        let details = okx_demo_provider_audit_details("submit", "env", request, &provider_response);
        let text = details.to_string();
        assert!(text.contains("x-simulated-trading=1"));
        assert!(text.contains(OKX_DEMO_AUDIT_ENVIRONMENT));
        assert!(!text.contains("OK-ACCESS"));
        assert!(!text.contains("secret"));
        assert!(!text.contains("signature"));
        assert!(!text.contains("passphrase"));
    }
}
