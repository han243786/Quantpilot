/// v4.7.0: QuantPilot 实时执行端
/// 双切面执行端 — PaperSimulated 拉取 OKX 公共行情并本地模拟, PaperActual 仅接 OKX demo 回执
mod api_guard;
mod audit_log;
mod credential_vault_v2;
mod executor_state;
mod kline_buffer;
mod live_runner;
mod migration_api;
pub mod okx_rest;
mod provider_order_routes;
mod state_routes;
mod ws_client;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use executor_state::{
    ActiveStrategy, ExecutionMode, ExecutorState, RuntimeKind, StrategyStatus, TriggerEvent,
};
use futures_core::Stream;
use live_runner::RunnerPool;
use okx_rest::{OkxOrderRequest, OKX_DEMO_AUDIT_ENVIRONMENT, OKX_DEMO_PROVIDER_KEY};
use qrpc_core::CoreStrategyIr;
use qrpc_core_ir::{
    v4::{
        CapabilitySupportSource, ExecutionCapabilityKind, PluginKind, PluginManifestSpec,
        PluginNetworkPermission, PluginRuntimePermission, PluginSideEffect, QsScalarTypeKind,
        QsTypeRef, RuntimeTradingMode, V4MachineGraphContract, V4StaticContractBundle,
        VenueCapabilityMatrix,
    },
    CoreMetadata, CoreSourceKind, CoreTimeInForce, ExecutionRule, ExecutionSizingKind,
};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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
            get(list_strategies).post(recv_strategy),
        )
        .route(
            "/api/executor/strategies/:strategy_id",
            get(get_strategy_detail),
        )
        .route(
            "/api/executor/strategies/:strategy_id/start",
            post(start_strategy),
        )
        .route(
            "/api/executor/strategies/:strategy_id/stop",
            post(stop_strategy),
        )
        .route(
            "/api/executor/strategies/:strategy_id/klines",
            get(get_klines),
        )
        .route(
            "/api/executor/strategies/:strategy_id/events",
            get(strategy_events_sse),
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

async fn list_strategies(State(state): State<Arc<ExecutorState>>) -> Json<serde_json::Value> {
    let s = state.strategies.read().unwrap_or_else(|e| e.into_inner());
    let items: Vec<_> = s
        .values()
        .map(|s| {
            serde_json::json!({
                "strategy_id": s.strategy_id, "name": s.name,
                "status": format!("{:?}", s.status), "mode": s.execution_mode.as_str(),
                "mode_label": s.execution_mode.display_label(),
                "runtime_kind": s.runtime_kind.as_str(),
                "runtime_version": s.runtime_kind.as_str(),
                "strategy_config_preflight": s.strategy_config_preflight,
            })
        })
        .collect();
    Json(serde_json::json!({"strategies": items}))
}

async fn get_strategy_detail(
    State(state): State<Arc<ExecutorState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let s = state.strategies.read().unwrap_or_else(|e| e.into_inner());
    let s = s
        .get(&id)
        .ok_or((axum::http::StatusCode::NOT_FOUND, "策略不存在".into()))?;
    let graph_node_count = s
        .graph_json
        .get("nodes")
        .and_then(|nodes| nodes.as_array())
        .map_or(0, |nodes| nodes.len());
    let recent_trigger_count = state
        .trigger_events
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .filter(|event| event.strategy_id == id)
        .count();
    let recent_audit_count = state
        .audit_log
        .recent(50)
        .into_iter()
        .filter(|entry| entry.strategy_id.as_deref() == Some(id.as_str()))
        .count();
    Ok(Json(serde_json::json!({
        "strategy_id": s.strategy_id, "name": s.name,
        "runtime_kind": s.runtime_kind.as_str(),
        "runtime_version": s.runtime_kind.as_str(),
        "graph_node_count": graph_node_count,
        "recent_trigger_count": recent_trigger_count,
        "recent_audit_count": recent_audit_count,
        "strategy_config_preflight": s.strategy_config_preflight,
        "open_orders": [], "portfolio": {"cash_balance": 100000.0, "available_cash_balance": 100000.0, "frozen_cash_balance": 0.0, "total_net_notional": 0.0},
    })))
}

#[derive(Debug, serde::Deserialize)]
struct V4StrategyDeployRequest {
    #[serde(default)]
    strategy_id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    runtime_kind: Option<String>,
    #[serde(default)]
    runtime_version: Option<String>,
    #[serde(default)]
    graph: Option<V4MachineGraphContract>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    params: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(default)]
    params_snapshot: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(default)]
    execution_mode: Option<String>,
    #[serde(default)]
    strategy_config_preflight: Option<serde_json::Value>,
}

fn is_v4_deploy_request(body: &serde_json::Value) -> bool {
    let runtime = body
        .get("runtime_kind")
        .or_else(|| body.get("runtime_version"))
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    runtime.eq_ignore_ascii_case("v4")
        || body.get("graph").is_some()
        || body.get("source").is_some()
}

fn deploy_v4_strategy(
    state: &Arc<ExecutorState>,
    body: serde_json::Value,
) -> Result<serde_json::Value, (axum::http::StatusCode, String)> {
    let request: V4StrategyDeployRequest = serde_json::from_value(body).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("v4 策略部署请求解析失败: {}", e),
        )
    })?;
    let runtime_label = request
        .runtime_kind
        .as_deref()
        .or(request.runtime_version.as_deref())
        .unwrap_or("v4");
    if !runtime_label.eq_ignore_ascii_case("v4") {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            format!("不支持的 runtime_kind: {}", runtime_label),
        ));
    }
    let graph = resolve_v4_deploy_graph(&request)?;
    let graph_id = graph.graph_id.clone();
    let strategy_id = request
        .strategy_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| graph_id.clone());
    let name = request
        .name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            graph
                .metadata
                .get("name")
                .and_then(|value| value.as_str())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| graph_id.clone());
    let params = request
        .params
        .or(request.params_snapshot)
        .unwrap_or_default();
    let subscribed_symbols = extract_v4_subscribed_symbols(&graph);
    let graph_json = serde_json::to_value(&graph).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("v4 graph 序列化失败: {}", e),
        )
    })?;
    let execution_mode = parse_execution_mode(request.execution_mode.as_deref())?;
    let strategy = ActiveStrategy {
        strategy_id: strategy_id.clone(),
        name,
        runtime_kind: RuntimeKind::V4,
        core_ir: empty_core_ir(&strategy_id),
        v4_graph: Some(graph),
        graph_json,
        params,
        status: StrategyStatus::Loaded,
        subscribed_symbols,
        execution_mode,
        strategy_config_preflight: request.strategy_config_preflight,
    };
    state.register(strategy).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("v4 策略注册失败: {:#}", e),
        )
    })?;
    append_audit(
        state,
        "load_strategy",
        Some(strategy_id.clone()),
        serde_json::json!({
            "source": "executor_v4_deploy",
            "runtime_kind": "v4",
            "execution_mode": execution_mode.as_str(),
            "execution_mode_label": execution_mode.display_label(),
        }),
    );
    Ok(serde_json::json!({
        "status": "loaded",
        "strategy_id": strategy_id,
        "runtime_kind": "v4",
        "runtime_version": "v4",
        "execution_mode": execution_mode.as_str(),
        "graph_id": graph_id,
    }))
}

fn parse_execution_mode(
    value: Option<&str>,
) -> Result<ExecutionMode, (axum::http::StatusCode, String)> {
    let raw = value.unwrap_or("paper_simulated");
    match ExecutionMode::from_api_label(raw) {
        Some(mode) => Ok(mode),
        None if raw.eq_ignore_ascii_case("live")
            || raw.eq_ignore_ascii_case("live_actual")
            || raw.eq_ignore_ascii_case("live_simulated") =>
        {
            Err((
                axum::http::StatusCode::BAD_REQUEST,
                "真实资金或真实账户上下文 execution_mode 已延后；请使用 paper_simulated / paper_actual"
                    .to_string(),
            ))
        }
        None => Err((
            axum::http::StatusCode::BAD_REQUEST,
            format!("不支持的 execution_mode: {}", raw),
        )),
    }
}

fn resolve_v4_deploy_graph(
    request: &V4StrategyDeployRequest,
) -> Result<V4MachineGraphContract, (axum::http::StatusCode, String)> {
    if let Some(graph) = request.graph.clone() {
        graph.validate_static_contract().map_err(|errors| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                format!("v4 graph 静态契约失败: {}", errors.join("; ")),
            )
        })?;
        return Ok(graph);
    }

    let source = request
        .source
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or((
            axum::http::StatusCode::BAD_REQUEST,
            "v4 策略部署需要 graph 或 source".to_string(),
        ))?;
    let report = quantscript::audit_v4_quant_script_static(source, &executor_v4_static_bundle());
    let handoff = quantscript::build_v4_qs_runtime_handoff(&report);
    if !handoff.accepted_for_runtime_handoff {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            format!(
                "v4 QS runtime handoff rejected: {}",
                handoff.diagnostics.join("; ")
            ),
        ));
    }
    report.parsed_graph.ok_or((
        axum::http::StatusCode::BAD_REQUEST,
        "v4 QS static audit did not produce a machine graph".to_string(),
    ))
}

fn extract_v4_subscribed_symbols(graph: &V4MachineGraphContract) -> Vec<qrpc_core::Symbol> {
    let mut symbols = Vec::new();
    for key in ["symbol", "default_symbol"] {
        if let Some(symbol) = graph.metadata.get(key).and_then(|value| value.as_str()) {
            symbols.push(qrpc_core::Symbol::Other(symbol.to_string()));
        }
    }
    if let Some(values) = graph
        .metadata
        .get("symbols")
        .and_then(|value| value.as_array())
    {
        for value in values {
            if let Some(symbol) = value.as_str() {
                symbols.push(qrpc_core::Symbol::Other(symbol.to_string()));
            }
        }
    }
    symbols
}

fn empty_core_ir(strategy_id: &str) -> CoreStrategyIr {
    CoreStrategyIr::new(
        CoreMetadata {
            strategy_id: strategy_id.to_string(),
            name: strategy_id.to_string(),
            source_kind: CoreSourceKind::RuntimeProtocol,
        },
        ExecutionRule {
            execution_id: format!("exec_{}", strategy_id),
            venue_kind: "paper".into(),
            sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
            slippage_bps: 0.0,
            taker_fee_bps: 0.0,
            total_cost_buffer_bps: 0.0,
            time_in_force: CoreTimeInForce::Gtc,
            params: BTreeMap::new(),
        },
    )
}

fn ensure_strategy_config_preflight_allows_start(
    strategy: &ActiveStrategy,
) -> Result<(), (StatusCode, String)> {
    if strategy.runtime_kind != RuntimeKind::V4 {
        return Ok(());
    }

    let Some(preflight) = strategy.strategy_config_preflight.as_ref() else {
        return Err(strategy_config_preflight_error(
            strategy,
            "strategy_config_preflight_missing",
            "v4 策略启动前缺少 strategy config preflight，已拒绝启动。",
            None,
        ));
    };

    let decision = json_path_str(preflight, &["decision"]).unwrap_or("blocked");
    if decision == "blocked" {
        return Err(strategy_config_preflight_error(
            strategy,
            "strategy_config_preflight_blocked",
            "strategy config preflight 已阻断启动。",
            Some(preflight),
        ));
    }

    if json_path_bool(preflight, &["can_live_execution"]).unwrap_or(false)
        || json_path_bool(
            preflight,
            &["artifact", "runtime_boundary", "live_execution_allowed"],
        )
        .unwrap_or(false)
    {
        return Err(strategy_config_preflight_error(
            strategy,
            "strategy_config_live_execution_forbidden",
            "live_execution_allowed=false 是执行端硬边界，已拒绝启动。",
            Some(preflight),
        ));
    }

    let mode_label = json_path_str(preflight, &["artifact", "runtime_boundary", "mode_label"])
        .unwrap_or("PaperSimulated");
    let (expected_mode_label, can_field, required_action) = match strategy.execution_mode {
        ExecutionMode::PaperSimulated => (
            "PaperSimulated",
            "can_paper_simulated",
            "start_paper_simulated",
        ),
        ExecutionMode::PaperActual => (
            "PaperActual",
            "can_paper_actual_demo",
            "start_paper_actual_demo",
        ),
    };

    if mode_label != expected_mode_label {
        return Err(strategy_config_preflight_error(
            strategy,
            "strategy_config_runtime_boundary_mismatch",
            "strategy config preflight 的运行边界与执行端策略模式不一致，已拒绝启动。",
            Some(preflight),
        ));
    }

    if !json_path_bool(preflight, &[can_field]).unwrap_or(false)
        || !json_array_contains(preflight, &["allowed_actions"], required_action)
    {
        return Err(strategy_config_preflight_error(
            strategy,
            "strategy_config_start_action_not_allowed",
            "strategy config preflight 未允许当前执行端启动动作，已拒绝启动。",
            Some(preflight),
        ));
    }

    Ok(())
}

fn strategy_config_preflight_error(
    strategy: &ActiveStrategy,
    code: &str,
    message: &str,
    preflight: Option<&serde_json::Value>,
) -> (StatusCode, String) {
    (
        StatusCode::LOCKED,
        serde_json::json!({
            "error": code,
            "message": message,
            "strategy_id": strategy.strategy_id,
            "runtime_kind": strategy.runtime_kind.as_str(),
            "execution_mode": strategy.execution_mode.as_str(),
            "execution_mode_label": strategy.execution_mode.display_label(),
            "preflight_decision": preflight.and_then(|value| json_path_str(value, &["decision"])),
            "artifact_digest": preflight
                .and_then(|value| json_path_str(value, &["artifact", "artifact_digest"])),
        })
        .to_string(),
    )
}

fn json_path_str<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn json_path_bool(value: &serde_json::Value, path: &[&str]) -> Option<bool> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_bool()
}

fn json_array_contains(value: &serde_json::Value, path: &[&str], expected: &str) -> bool {
    let mut current = value;
    for key in path {
        let Some(next) = current.get(*key) else {
            return false;
        };
        current = next;
    }
    current
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(expected)))
}

fn executor_v4_static_bundle() -> V4StaticContractBundle {
    V4StaticContractBundle {
        venue_matrices: vec![
            executor_v4_market_matrix("paper-local"),
            executor_v4_market_matrix("paper-simulated"),
        ],
        plugin_manifests: vec![executor_v4_sample_plugin_manifest()],
        ..V4StaticContractBundle::default()
    }
}

fn executor_v4_market_matrix(venue_id: impl Into<String>) -> VenueCapabilityMatrix {
    let mut matrix = qrpc_core_ir::v4::unsupported_v4_first_wave_matrix(venue_id);
    for entry in &mut matrix.capabilities {
        if matches!(
            entry.capability,
            ExecutionCapabilityKind::Market
                | ExecutionCapabilityKind::Limit
                | ExecutionCapabilityKind::StopMarket
                | ExecutionCapabilityKind::StopLimit
                | ExecutionCapabilityKind::TakeProfitMarket
                | ExecutionCapabilityKind::TakeProfitLimit
                | ExecutionCapabilityKind::Gtc
                | ExecutionCapabilityKind::Ioc
                | ExecutionCapabilityKind::Fok
                | ExecutionCapabilityKind::Day
                | ExecutionCapabilityKind::Gtd
                | ExecutionCapabilityKind::PostOnly
                | ExecutionCapabilityKind::ReduceOnly
                | ExecutionCapabilityKind::CloseOnly
                | ExecutionCapabilityKind::ClientOrderId
                | ExecutionCapabilityKind::OpenLong
                | ExecutionCapabilityKind::CloseLong
                | ExecutionCapabilityKind::OpenShort
                | ExecutionCapabilityKind::CloseShort
        ) {
            entry.source = CapabilitySupportSource::RuntimeSimulated;
            entry.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
        }
    }
    matrix
}

fn executor_v4_sample_plugin_manifest() -> PluginManifestSpec {
    PluginManifestSpec {
        plugin_id: "pure.indicator.zscore".to_string(),
        name: "ZScore".to_string(),
        version: "0.1.0".to_string(),
        kind: PluginKind::Pure,
        input_schema: Some(QsTypeRef::List {
            item: Box::new(QsTypeRef::Scalar {
                scalar: QsScalarTypeKind::Price,
            }),
            max_items: 256,
        }),
        output_schema: Some(QsTypeRef::Scalar {
            scalar: QsScalarTypeKind::Decimal,
        }),
        deterministic: true,
        side_effect: PluginSideEffect::None,
        runtime_permission: PluginRuntimePermission::None,
        network_permission: PluginNetworkPermission::None,
        capability_matrix: None,
        test_fixture_id: "fixture.zscore.basic".to_string(),
    }
}

async fn recv_strategy(
    State(state): State<Arc<ExecutorState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, String)> {
    if is_v4_deploy_request(&body) {
        let response = deploy_v4_strategy(&state, body)?;
        return Ok((StatusCode::CREATED, Json(response)));
    }

    let body_bytes = serde_json::to_vec(&body).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("序列化失败: {}", e),
        )
    })?;
    let pkg = migration_api::decrypt_package(&body_bytes).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("策略包解析失败: {}", e),
        )
    })?;
    let strategy_id = pkg.strategy_id.clone();
    // v3.0.2 E-1: 签名/溯源错误→401
    migration_api::load_strategy(&state, pkg).map_err(|e| {
        let msg = format!("{:#}", e);
        let status = if msg.contains("签名") || msg.contains("溯源") {
            axum::http::StatusCode::UNAUTHORIZED
        } else {
            axum::http::StatusCode::BAD_REQUEST
        };
        (status, msg)
    })?;
    append_audit(
        &state,
        "load_strategy",
        Some(strategy_id.clone()),
        serde_json::json!({ "source": "migration_api" }),
    );
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({"status": "loaded", "strategy_id": strategy_id})),
    ))
}

async fn start_strategy(
    State(state): State<Arc<ExecutorState>>,
    Path(strategy_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // v3.2.2: 幂等保护 — Running状态不允许重复启动
    let strategy = {
        let s = state.strategies.read().map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?;
        let strategy = s
            .get(&strategy_id)
            .ok_or((axum::http::StatusCode::NOT_FOUND, "策略不存在".into()))?;
        if strategy.status == StrategyStatus::Running {
            return Ok(Json(
                serde_json::json!({"status": "already_running", "strategy_id": strategy_id}),
            ));
        }
        strategy.clone()
    };
    if let Err(error) = ensure_strategy_config_preflight_allows_start(&strategy) {
        append_audit(
            &state,
            "strategy_config_preflight_blocked",
            Some(strategy_id.clone()),
            serde_json::json!({
                "status": "blocked",
                "runtime_kind": strategy.runtime_kind.as_str(),
                "execution_mode": strategy.execution_mode.as_str(),
                "reason": &error.1,
            }),
        );
        return Err(error);
    }
    if strategy.execution_mode.provider_order_submission_attached() {
        // PaperActual is non-real-funds only. Verify demo credentials before allowing the
        // automatic runner to leave Loaded state; provider submit/query/cancel routes stay
        // explicit audited boundaries for order-router integration.
        load_okx_demo_credentials()?;
    }
    {
        let pool_opt = state.runner_pool.lock().map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?;
        if let Some(ref pool_arc) = *pool_opt {
            pool_arc
                .lock()
                .map_err(|e| {
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("锁: {}", e),
                    )
                })?
                .register(&strategy)
                .map_err(|e| {
                    (
                        axum::http::StatusCode::BAD_REQUEST,
                        format!("runner 注册失败: {:#}", e),
                    )
                })?;
        } else {
            drop(pool_opt);
            if let Ok(mut strategies) = state.strategies.write() {
                if let Some(strategy) = strategies.get_mut(&strategy_id) {
                    strategy.status = StrategyStatus::Error("runner_pool_unavailable".to_string());
                }
            }
            return Err((
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "执行端运行池不可用".into(),
            ));
        }
    }
    if let Some(strategy) = state
        .strategies
        .write()
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?
        .get_mut(&strategy_id)
    {
        strategy.status = StrategyStatus::Running;
    }
    let (runtime_kind, execution_mode) = state
        .strategies
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .get(&strategy_id)
        .map(|s| (s.runtime_kind.as_str(), s.execution_mode))
        .unwrap_or(("v3", ExecutionMode::PaperSimulated));
    append_audit(
        &state,
        "start_strategy",
        Some(strategy_id.clone()),
        serde_json::json!({
            "status": "running",
            "execution_mode": execution_mode.as_str(),
            "execution_mode_label": execution_mode.display_label(),
            "provider_order_submission_attached": execution_mode.provider_order_submission_attached(),
            "provider_order_submission_policy": if execution_mode.provider_order_submission_attached() {
                "okx_demo_credentials_verified_manual_provider_routes_available"
            } else {
                "runtime_simulated"
            },
            "strategy_config_preflight_decision": strategy.strategy_config_preflight.as_ref().and_then(|value| json_path_str(value, &["decision"])),
            "strategy_config_artifact_digest": strategy.strategy_config_preflight.as_ref().and_then(|value| json_path_str(value, &["artifact", "artifact_digest"])),
        }),
    );
    Ok(Json(serde_json::json!({
        "status": "running",
        "strategy_id": strategy_id,
        "runtime_kind": runtime_kind,
        "runtime_version": runtime_kind,
        "execution_mode": execution_mode.as_str(),
        "execution_mode_label": execution_mode.display_label(),
        "provider_order_submission_attached": execution_mode.provider_order_submission_attached(),
        "provider_order_submission_policy": if execution_mode.provider_order_submission_attached() {
            "okx_demo_credentials_verified_manual_provider_routes_available"
        } else {
            "runtime_simulated"
        },
        "strategy_config_preflight_decision": strategy.strategy_config_preflight.as_ref().and_then(|value| json_path_str(value, &["decision"])),
        "strategy_config_artifact_digest": strategy.strategy_config_preflight.as_ref().and_then(|value| json_path_str(value, &["artifact", "artifact_digest"])),
    })))
}

async fn stop_strategy(
    State(state): State<Arc<ExecutorState>>,
    Path(strategy_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // v3.3.0 P2修复: 先检查策略是否存在
    {
        let s = state.strategies.read().map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?;
        if !s.contains_key(&strategy_id) {
            return Err((axum::http::StatusCode::NOT_FOUND, "策略不存在".into()));
        }
    }
    // 从RunnerPool移除停止的策略
    {
        let pool_opt = state.runner_pool.lock().map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?;
        if let Some(ref pool_arc) = *pool_opt {
            pool_arc
                .lock()
                .map_err(|e| {
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("锁: {}", e),
                    )
                })?
                .remove(&strategy_id);
        }
    }
    if let Some(strategy) = state
        .strategies
        .write()
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?
        .get_mut(&strategy_id)
    {
        strategy.status = StrategyStatus::Stopped;
    }
    append_audit(
        &state,
        "stop_strategy",
        Some(strategy_id.clone()),
        serde_json::json!({ "status": "stopped" }),
    );
    Ok(Json(
        serde_json::json!({"status": "stopped", "strategy_id": strategy_id}),
    ))
}

async fn get_klines(
    State(state): State<Arc<ExecutorState>>,
    Path(strategy_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let subscribed_symbols = {
        let strategies = state.strategies.read().map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?;
        let strategy = strategies
            .get(&strategy_id)
            .ok_or((axum::http::StatusCode::NOT_FOUND, "策略不存在".into()))?;
        strategy
            .subscribed_symbols
            .iter()
            .map(|symbol| symbol.as_str().to_string())
            .collect::<Vec<_>>()
    };
    let mut bars = Vec::new();
    let mut latest_prices = serde_json::Map::new();
    if let Some(pool_arc) = state
        .runner_pool
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .cloned()
    {
        let pool = pool_arc.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(runner) = pool.runners.get(&strategy_id) {
            if let Some(kline_pool) = runner.kline_pool() {
                let symbols = if subscribed_symbols.is_empty() {
                    kline_pool.buffers.keys().cloned().collect::<Vec<_>>()
                } else {
                    subscribed_symbols.clone()
                };
                for symbol in symbols {
                    bars.extend(kline_pool.recent_bars(&symbol, 1_000).into_iter().cloned());
                    if let Some(price) = kline_pool.latest_price(&symbol) {
                        latest_prices.insert(symbol, serde_json::json!(price));
                    }
                }
            }
        }
    }
    if !bars.is_empty() {
        return Ok(Json(serde_json::json!({
            "strategy_id": strategy_id,
            "bars": bars,
            "latest_prices": latest_prices,
        })));
    }
    let buffers = state
        .kline_buffers
        .read()
        .unwrap_or_else(|e| e.into_inner());
    let bars: Vec<_> = buffers
        .values()
        .flat_map(|b| b.bars.iter())
        .cloned()
        .collect();
    Ok(Json(serde_json::json!({
        "strategy_id": strategy_id,
        "bars": bars,
        "latest_prices": latest_prices,
    })))
}

async fn strategy_events_sse(
    State(state): State<Arc<ExecutorState>>,
    Path(strategy_id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (axum::http::StatusCode, String)> {
    {
        let strategies = state.strategies.read().map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?;
        if !strategies.contains_key(&strategy_id) {
            return Err((axum::http::StatusCode::NOT_FOUND, "策略不存在".into()));
        }
    }
    let initial_v4_snapshot = {
        let pool_opt = state.runner_pool.lock().unwrap_or_else(|e| e.into_inner());
        pool_opt.as_ref().and_then(|pool_arc| {
            let pool = pool_arc.lock().unwrap_or_else(|e| e.into_inner());
            pool.runners
                .get(&strategy_id)
                .and_then(|runner| runner.v4_memory_snapshot(now_ms()))
        })
    };
    let (mut rx, mut v4_rx) = {
        let pool_opt = state.runner_pool.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref pool_arc) = *pool_opt {
            let pool = pool_arc.lock().unwrap_or_else(|e| e.into_inner());
            (
                pool.trigger_broadcast.subscribe(),
                pool.v4_evidence_broadcast.subscribe(),
            )
        } else {
            let (bc, _) = broadcast::channel(1);
            let (v4_bc, _) = broadcast::channel(1);
            (bc.subscribe(), v4_bc.subscribe())
        }
    };
    let stream = async_stream::stream! {
        yield Ok(Event::default().event("connected").data("{}"));
        if let Some(snapshot) = initial_v4_snapshot {
            let json = serde_json::json!({
                "strategy_id": strategy_id,
                "memory_snapshot": snapshot,
                "runtime_events": [],
            });
            yield Ok(Event::default().event("v4RuntimeMemorySnapshot").data(json.to_string()));
        }
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(30)) => {
                    yield Ok(Event::default().event("keepalive").data("{}"));
                }
                trigger = rx.recv() => {
                    match trigger {
                        Ok(t) if t.strategy_id == strategy_id => {
                            if let Ok(mut events) = state.trigger_events.write() {
                                events.push(t.clone());
                                if events.len() > 1_000 {
                                    events.remove(0);
                                }
                            }
                            let json = serde_json::json!({
                                "strategy_id": t.strategy_id,
                                "trigger_type": t.trigger_type,
                                "node_id": t.node_id, "strength": t.strength,
                                "occurred_at_ms": t.occurred_at_ms,
                            });
                            yield Ok(Event::default().event("trigger").data(json.to_string()));
                        }
                        Ok(_) => continue,
                        Err(broadcast::error::RecvError::Lagged(dropped)) => {
                            state.record_sse_lagged("trigger", dropped);
                            continue;
                        }
                        Err(_) => break,
                    }
                }
                evidence = v4_rx.recv() => {
                    match evidence {
                        Ok(e) if e.strategy_id == strategy_id => {
                            let json = serde_json::json!({
                                "strategy_id": e.strategy_id,
                                "memory_snapshot": e.memory_snapshot,
                                "runtime_events": e.runtime_events,
                            });
                            yield Ok(Event::default().event(e.event_type).data(json.to_string()));
                        }
                        Ok(_) => continue,
                        Err(broadcast::error::RecvError::Lagged(dropped)) => {
                            state.record_sse_lagged("v4_evidence", dropped);
                            continue;
                        }
                        Err(_) => break,
                    }
                }
            }
        }
    };
    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(5))
            .text("keepalive"),
    ))
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
    use super::*;

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
