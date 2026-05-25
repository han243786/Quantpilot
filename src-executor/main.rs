/// v4.3.0: QuantPilot 实时执行端
/// OKX Paper 模式 — 每交易所独立WS, 策略启动后 RunnerPool 激活
mod api_guard;
mod audit_log;
#[cfg(test)]
mod credential_vault_v2;
mod executor_state;
mod kline_buffer;
mod live_runner;
mod migration_api;
#[cfg(test)]
mod okx_rest;
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

    // v3.7.0: 广播通道 (SSE trigger推送) + OKX Paper WS 事件通道
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
    let _feed_handle = tokio::spawn(run_okx_paper_feed(okx_tx));

    let app = Router::new()
        .route("/api/executor/health", get(health_check))
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
            get(get_params).post(update_params),
        )
        .route("/api/executor/mode", get(get_mode).post(set_mode))
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
    println!("[executor] ✓ 实时执行端已就绪 → {} (OKX Paper)", addr);
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("[executor] 服务运行失败: {}", e);
        std::process::exit(1);
    }
}

// ── OKX Paper 模拟行情 ──

async fn run_okx_paper_feed(tx: mpsc::UnboundedSender<crate::ws_client::WsEvent>) {
    let _ = tx.send(crate::ws_client::WsEvent::Connected {
        exchange: "okx".into(),
    });
    let mut price = 87234.0;
    let mut tick_count: u64 = 0;
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        tick_count += 1;
        let ts = now_ms();
        // 简单确定性价格模拟
        let pseudo = ((ts % 1000) as f64 / 1000.0 - 0.5) * 20.0;
        price = (price + pseudo).max(100.0_f64);
        let _ = tx.send(crate::ws_client::WsEvent::Ticker {
            symbol: "BTCUSDT".into(),
            price,
            ts_ms: ts,
        });
        // v3.2.0 S0修复: 每60秒合成一个Kline事件触发慢周期执行
        if tick_count % 60 == 0 {
            let minute_start = ts / 60_000 * 60_000;
            let _ = tx.send(crate::ws_client::WsEvent::Kline {
                symbol: "BTCUSDT".into(),
                bar: crate::executor_state::KlineBar {
                    open_time_ms: minute_start,
                    close_time_ms: minute_start + 59_999,
                    open: price,
                    high: price * 1.002,
                    low: price * 0.998,
                    close: price,
                    volume: 1.5,
                },
            });
        }
    }
}

// ── 端点实现 ──

async fn health_check() -> &'static str {
    "executor_ok"
}

// v3.5.0: 全局执行模式查询与切换
async fn get_mode(State(state): State<Arc<ExecutorState>>) -> Json<serde_json::Value> {
    let mode = state.current_mode();
    Json(serde_json::json!({
        "mode": format!("{:?}", mode).to_lowercase(),
        "available_modes": ["paper", "live"]
    }))
}

#[derive(serde::Deserialize)]
struct SetModeRequest {
    mode: String,
}

async fn set_mode(
    State(state): State<Arc<ExecutorState>>,
    Json(req): Json<SetModeRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let new_mode = match req.mode.to_lowercase().as_str() {
        "paper" => ExecutionMode::Paper,
        "live" => ExecutionMode::Live,
        other => {
            return Err((
                StatusCode::BAD_REQUEST,
                serde_json::json!({
                    "error": "invalid_mode",
                    "message": format!("不支持的模式: '{}', 仅支持 paper/live", other),
                    "available_modes": ["paper", "live"]
                })
                .to_string(),
            ));
        }
    };
    let old_mode = state.set_mode(new_mode.clone());
    let mode_str = format!("{:?}", new_mode).to_lowercase();
    eprintln!("[executor] 模式切换: {:?} → {:?}", old_mode, new_mode);
    append_audit(
        &state,
        "set_mode",
        None,
        serde_json::json!({
            "previous_mode": format!("{:?}", old_mode).to_lowercase(),
            "current_mode": mode_str.clone(),
        }),
    );
    Ok(Json(serde_json::json!({
        "previous_mode": format!("{:?}", old_mode).to_lowercase(),
        "current_mode": mode_str,
        "message": format!("执行端已切换到 {} 模式", if new_mode == ExecutionMode::Live { "实盘" } else { "模拟盘" })
    })))
}

async fn list_strategies(State(state): State<Arc<ExecutorState>>) -> Json<serde_json::Value> {
    let s = state.strategies.read().unwrap_or_else(|e| e.into_inner());
    let items: Vec<_> = s
        .values()
        .map(|s| {
            serde_json::json!({
                "strategy_id": s.strategy_id, "name": s.name,
                "status": format!("{:?}", s.status), "mode": format!("{:?}", s.execution_mode),
                "runtime_kind": s.runtime_kind.as_str(),
                "runtime_version": s.runtime_kind.as_str(),
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
        execution_mode: parse_execution_mode(request.execution_mode.as_deref())?,
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
        serde_json::json!({ "source": "executor_v4_deploy", "runtime_kind": "v4" }),
    );
    Ok(serde_json::json!({
        "status": "loaded",
        "strategy_id": strategy_id,
        "runtime_kind": "v4",
        "runtime_version": "v4",
        "graph_id": graph_id,
    }))
}

fn parse_execution_mode(
    value: Option<&str>,
) -> Result<ExecutionMode, (axum::http::StatusCode, String)> {
    match value.unwrap_or("paper").to_ascii_lowercase().as_str() {
        "paper" => Ok(ExecutionMode::Paper),
        "live" => Ok(ExecutionMode::Live),
        other => Err((
            axum::http::StatusCode::BAD_REQUEST,
            format!("不支持的 execution_mode: {}", other),
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

fn executor_v4_static_bundle() -> V4StaticContractBundle {
    V4StaticContractBundle {
        venue_matrices: vec![executor_v4_market_matrix("paper-local")],
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
    {
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
                .register(strategy)
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
    state
        .strategies
        .write()
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?
        .get_mut(&strategy_id)
        .map(|s| s.status = StrategyStatus::Running);
    append_audit(
        &state,
        "start_strategy",
        Some(strategy_id.clone()),
        serde_json::json!({ "status": "running" }),
    );
    let runtime_kind = state
        .strategies
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .get(&strategy_id)
        .map(|s| s.runtime_kind.as_str())
        .unwrap_or("v3");
    Ok(Json(serde_json::json!({
        "status": "running",
        "strategy_id": strategy_id,
        "runtime_kind": runtime_kind,
        "runtime_version": runtime_kind,
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
    state
        .strategies
        .write()
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?
        .get_mut(&strategy_id)
        .map(|s| s.status = StrategyStatus::Stopped);
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
        yield Ok(Event::default().data(r#"{"type":"connected"}"#));
        if let Some(snapshot) = initial_v4_snapshot {
            let json = serde_json::json!({
                "type": "v4RuntimeMemorySnapshot",
                "strategy_id": strategy_id,
                "memory_snapshot": snapshot,
                "runtime_events": [],
            });
            yield Ok(Event::default().data(json.to_string()));
        }
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(30)) => {
                    yield Ok(Event::default().data(":keepalive"));
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
                                "type": "trigger", "strategy_id": t.strategy_id,
                                "trigger_type": t.trigger_type,
                                "node_id": t.node_id, "strength": t.strength,
                                "occurred_at_ms": t.occurred_at_ms,
                            });
                            yield Ok(Event::default().data(json.to_string()));
                        }
                        Ok(_) => continue,
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(_) => break,
                    }
                }
                evidence = v4_rx.recv() => {
                    match evidence {
                        Ok(e) if e.strategy_id == strategy_id => {
                            let json = serde_json::json!({
                                "type": e.event_type,
                                "strategy_id": e.strategy_id,
                                "memory_snapshot": e.memory_snapshot,
                                "runtime_events": e.runtime_events,
                            });
                            yield Ok(Event::default().data(json.to_string()));
                        }
                        Ok(_) => continue,
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
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

async fn get_params(
    State(state): State<Arc<ExecutorState>>,
    Path(strategy_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let s = state.strategies.read().map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("锁: {}", e),
        )
    })?;
    let s = s
        .get(&strategy_id)
        .ok_or((axum::http::StatusCode::NOT_FOUND, "策略不存在".into()))?;
    let snapshot_count = state
        .params_snapshots
        .read()
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?
        .get(&strategy_id)
        .map_or(0, |snapshots| snapshots.len());
    Ok(Json(serde_json::json!({
        "strategy_id": strategy_id,
        "params": s.params,
        "snapshot_count": snapshot_count,
    })))
}

async fn update_params(
    State(state): State<Arc<ExecutorState>>,
    Path(strategy_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // 读取策略并提取已有参数键名 (持锁期间完成读取)
    let (existing_params, is_running) = {
        let s = state.strategies.read().map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?;
        let s = s
            .get(&strategy_id)
            .ok_or((axum::http::StatusCode::NOT_FOUND, "策略不存在".into()))?;
        (s.params.clone(), s.status == StrategyStatus::Running)
    };
    if !is_running {
        // v3.0.2 E-2: 资源被锁定(未运行) → 423 Locked
        return Err((axum::http::StatusCode::LOCKED, "策略未在运行中".into()));
    }
    let new_params: BTreeMap<String, serde_json::Value> = body
        .get("params")
        .and_then(|v| v.as_object())
        .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    if new_params.is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "params 不能为空".into(),
        ));
    }

    // P2-12: 校验参数键名 — 拒绝未知键
    for key in new_params.keys() {
        if !existing_params.contains_key(key) {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                serde_json::json!({
                    "error": "invalid_params",
                    "message": format!("未知参数: {}", key)
                })
                .to_string(),
            ));
        }
    }

    // P2-12: 校验参数值类型 — 仅允许基本 JSON 类型 (字符串/数字/布尔), 拒绝 null/数组/对象
    for (key, value) in &new_params {
        if value.is_null() || value.is_array() || value.is_object() {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                serde_json::json!({
                    "error": "invalid_params",
                    "message": format!("参数 '{}' 的值类型无效: 不允许 null/数组/对象", key)
                })
                .to_string(),
            ));
        }
        validate_hot_param_value(key, existing_params.get(key), value)?;
    }

    {
        let mut snapshots = state.params_snapshots.write().map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?;
        let history = snapshots.entry(strategy_id.clone()).or_default();
        history.push(existing_params);
        if history.len() > 20 {
            history.remove(0);
        }
    }

    state
        .pending_params
        .write()
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("锁: {}", e),
            )
        })?
        .insert(strategy_id.clone(), new_params);
    append_audit(
        &state,
        "update_params",
        Some(strategy_id.clone()),
        serde_json::json!({ "status": "pending" }),
    );
    Ok(Json(
        serde_json::json!({"status": "pending", "strategy_id": strategy_id}),
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

fn validate_hot_param_value(
    key: &str,
    existing: Option<&serde_json::Value>,
    value: &serde_json::Value,
) -> Result<(), (axum::http::StatusCode, String)> {
    let Some(existing) = existing else {
        return Ok(());
    };
    let invalid = |message: String| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            serde_json::json!({
                "error": "invalid_params",
                "message": message
            })
            .to_string(),
        )
    };

    match (existing, value) {
        (serde_json::Value::Bool(_), serde_json::Value::Bool(_)) => Ok(()),
        (serde_json::Value::String(_), serde_json::Value::String(next)) => {
            if next.len() > 1024 {
                return Err(invalid(format!("参数 '{}' 字符串长度超过 1024", key)));
            }
            Ok(())
        }
        (serde_json::Value::Number(_), serde_json::Value::Number(next)) => {
            let Some(number) = next.as_f64() else {
                return Err(invalid(format!("参数 '{}' 必须是有限数字", key)));
            };
            if !number.is_finite() || number.abs() > 1_000_000_000_000.0 {
                return Err(invalid(format!("参数 '{}' 超出允许范围", key)));
            }
            Ok(())
        }
        _ => Err(invalid(format!("参数 '{}' 类型必须与部署快照一致", key))),
    }
}
