/// v3.7.0: QuantPilot 实时执行端
/// OKX Paper 模式 — 每交易所独立WS, 策略启动后 RunnerPool 激活

mod api_guard;
mod audit_log;
mod credential_vault_v2;
mod executor_state;
mod kline_buffer;
mod live_runner;
mod migration_api;
mod okx_rest;
use qrpc_session as crypto;
mod ws_client;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use executor_state::{ExecutorState, ExecutionMode, StrategyStatus, TriggerEvent};
use futures_core::Stream;
use live_runner::RunnerPool;
// crypto already imported above as qrpc_session
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

const EXECUTOR_PORT: u16 = 3001;

#[tokio::main]
async fn main() {
    println!("[executor] QuantPilot 实时执行端 v3.7.0 启动中...");

    if let Err(e) = qrpc_session::init_session_key() {
        eprintln!("[executor] 会话密钥初始化失败: {} (测试端将无法连接)", e);
    } else {
        println!("[executor] 会话密钥已生成");
    }

    let state = ExecutorState::new();

    // v3.7.0: 广播通道 (SSE trigger推送) + OKX Paper WS 事件通道
    let (trigger_broadcast, _) = broadcast::channel::<TriggerEvent>(256);
    let (okx_tx, mut okx_rx) = mpsc::unbounded_channel::<crate::ws_client::WsEvent>();

    let mut pool = RunnerPool::new(trigger_broadcast.clone());
    pool.register_exchange("okx", okx_tx.clone());
    state.ws_tx_map.write().unwrap_or_else(|e| e.into_inner()).insert("okx".into(), okx_tx.clone());

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
        .route("/api/executor/strategies", get(list_strategies).post(recv_strategy))
        .route("/api/executor/strategies/:strategy_id", get(get_strategy_detail))
        .route("/api/executor/strategies/:strategy_id/start", post(start_strategy))
        .route("/api/executor/strategies/:strategy_id/stop", post(stop_strategy))
        .route("/api/executor/strategies/:strategy_id/klines", get(get_klines))
        .route("/api/executor/strategies/:strategy_id/events", get(strategy_events_sse))
        .route("/api/executor/params/:strategy_id", get(get_params).post(update_params))
        .route("/api/executor/mode", get(get_mode).post(set_mode))
        // v3.3.0: API守卫中间件 (Phase 4透传, Phase 5启用HMAC验证)
        .layer(axum::middleware::from_fn(crate::api_guard::api_guard_middleware))
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
    let _ = tx.send(crate::ws_client::WsEvent::Connected { exchange: "okx".into() });
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
            symbol: "BTCUSDT".into(), price, ts_ms: ts,
        });
        // v3.2.0 S0修复: 每60秒合成一个Kline事件触发慢周期执行
        if tick_count % 60 == 0 {
            let minute_start = ts / 60_000 * 60_000;
            let _ = tx.send(crate::ws_client::WsEvent::Kline {
                symbol: "BTCUSDT".into(),
                bar: crate::executor_state::KlineBar {
                    open_time_ms: minute_start,
                    close_time_ms: minute_start + 59_999,
                    open: price, high: price * 1.002, low: price * 0.998, close: price,
                    volume: 1.5,
                },
            });
        }
    }
}

// ── 端点实现 ──

async fn health_check() -> &'static str { "executor_ok" }

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
            return Err((StatusCode::BAD_REQUEST, serde_json::json!({
                "error": "invalid_mode",
                "message": format!("不支持的模式: '{}', 仅支持 paper/live", other),
                "available_modes": ["paper", "live"]
            }).to_string()));
        }
    };
    let old_mode = state.set_mode(new_mode.clone());
    let mode_str = format!("{:?}", new_mode).to_lowercase();
    eprintln!("[executor] 模式切换: {:?} → {:?}", old_mode, new_mode);
    Ok(Json(serde_json::json!({
        "previous_mode": format!("{:?}", old_mode).to_lowercase(),
        "current_mode": mode_str,
        "message": format!("执行端已切换到 {} 模式", if new_mode == ExecutionMode::Live { "实盘" } else { "模拟盘" })
    })))
}

async fn list_strategies(
    State(state): State<Arc<ExecutorState>>,
) -> Json<serde_json::Value> {
    let s = state.strategies.read().unwrap_or_else(|e| e.into_inner());
    let items: Vec<_> = s.values().map(|s| serde_json::json!({
        "strategy_id": s.strategy_id, "name": s.name,
        "status": format!("{:?}", s.status), "mode": format!("{:?}", s.execution_mode),
    })).collect();
    Json(serde_json::json!({"strategies": items}))
}

async fn get_strategy_detail(
    State(state): State<Arc<ExecutorState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let s = state.strategies.read().unwrap_or_else(|e| e.into_inner());
    let s = s.get(&id).ok_or((axum::http::StatusCode::NOT_FOUND, "策略不存在".into()))?;
    Ok(Json(serde_json::json!({
        "strategy_id": s.strategy_id, "name": s.name,
        "open_orders": [], "portfolio": {"cash_balance": 100000.0, "available_cash_balance": 100000.0, "frozen_cash_balance": 0.0, "total_net_notional": 0.0},
    })))
}

async fn recv_strategy(
    State(state): State<Arc<ExecutorState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let body_bytes = serde_json::to_vec(&body).map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("序列化失败: {}", e)))?;
    let pkg = migration_api::decrypt_package(&body_bytes).map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("策略包解析失败: {}", e)))?;
    let strategy_id = pkg.strategy_id.clone();
    // v3.0.2 E-1: 签名/溯源错误→401
    migration_api::load_strategy(&state, pkg).map_err(|e| {
        let msg = format!("{:#}", e);
        let status = if msg.contains("签名") || msg.contains("溯源") { axum::http::StatusCode::UNAUTHORIZED } else { axum::http::StatusCode::BAD_REQUEST };
        (status, msg)
    })?;
    Ok(Json(serde_json::json!({"status": "loaded", "strategy_id": strategy_id})))
}

async fn start_strategy(
    State(state): State<Arc<ExecutorState>>,
    Path(strategy_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // v3.2.2: 幂等保护 — Running状态不允许重复启动
    {
        let s = state.strategies.read().map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", e)))?;
        let strategy = s.get(&strategy_id).ok_or((axum::http::StatusCode::NOT_FOUND, "策略不存在".into()))?;
        if strategy.status == StrategyStatus::Running {
            return Ok(Json(serde_json::json!({"status": "already_running", "strategy_id": strategy_id})));
        }
        let pool_opt = state.runner_pool.lock().map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", e)))?;
        if let Some(ref pool_arc) = *pool_opt {
            pool_arc.lock().map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", e)))?
                .register(strategy);
        }
    }
    state.strategies.write().map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", e)))?
        .get_mut(&strategy_id).map(|s| s.status = StrategyStatus::Running);
    Ok(Json(serde_json::json!({"status": "running", "strategy_id": strategy_id})))
}

async fn stop_strategy(
    State(state): State<Arc<ExecutorState>>,
    Path(strategy_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // v3.3.0 P2修复: 先检查策略是否存在
    {
        let s = state.strategies.read().map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", e)))?;
        if !s.contains_key(&strategy_id) {
            return Err((axum::http::StatusCode::NOT_FOUND, "策略不存在".into()));
        }
    }
    // 从RunnerPool移除停止的策略
    {
        let pool_opt = state.runner_pool.lock().map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", e)))?;
        if let Some(ref pool_arc) = *pool_opt {
            pool_arc.lock().map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", e)))?
                .remove(&strategy_id);
        }
    }
    state.strategies.write().map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", e)))?
        .get_mut(&strategy_id).map(|s| s.status = StrategyStatus::Stopped);
    Ok(Json(serde_json::json!({"status": "stopped", "strategy_id": strategy_id})))
}

async fn get_klines(
    State(state): State<Arc<ExecutorState>>,
    Path(strategy_id): Path<String>,
) -> Json<serde_json::Value> {
    let buffers = state.kline_buffers.read().unwrap_or_else(|e| e.into_inner());
    let bars: Vec<_> = buffers.values().flat_map(|b| b.bars.iter()).cloned().collect();
    Json(serde_json::json!({"strategy_id": strategy_id, "bars": bars}))
}

async fn strategy_events_sse(
    State(state): State<Arc<ExecutorState>>,
    Path(strategy_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = {
        let pool_opt = state.runner_pool.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref pool_arc) = *pool_opt {
            pool_arc.lock().unwrap_or_else(|e| e.into_inner()).trigger_broadcast.subscribe()
        } else {
            let (bc, _) = broadcast::channel(1);
            bc.subscribe()
        }
    };
    let stream = async_stream::stream! {
        yield Ok(Event::default().data(r#"{"type":"connected"}"#));
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(30)) => {
                    yield Ok(Event::default().data(":keepalive"));
                }
                trigger = rx.recv() => {
                    match trigger {
                        Ok(t) if t.strategy_id == strategy_id => {
                            let json = serde_json::json!({
                                "type": "trigger", "strategy_id": t.strategy_id,
                                "node_id": t.node_id, "strength": t.strength,
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
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(5)).text("keepalive"))
}

async fn get_params(
    State(state): State<Arc<ExecutorState>>,
    Path(strategy_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let s = state.strategies.read().map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", e)))?;
    let s = s.get(&strategy_id).ok_or((axum::http::StatusCode::NOT_FOUND, "策略不存在".into()))?;
    Ok(Json(serde_json::json!({"strategy_id": strategy_id, "params": s.params})))
}

async fn update_params(
    State(state): State<Arc<ExecutorState>>,
    Path(strategy_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // 读取策略并提取已有参数键名 (持锁期间完成读取)
    let (existing_param_keys, is_running) = {
        let s = state.strategies.read().map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", e)))?;
        let s = s.get(&strategy_id).ok_or((axum::http::StatusCode::NOT_FOUND, "策略不存在".into()))?;
        let keys: Vec<String> = s.params.keys().cloned().collect();
        (keys, s.status == StrategyStatus::Running)
    };
    if !is_running {
        // v3.0.2 E-2: 资源被锁定(未运行) → 423 Locked
        return Err((axum::http::StatusCode::LOCKED, "策略未在运行中".into()));
    }
    let new_params: BTreeMap<String, serde_json::Value> = body.get("params")
        .and_then(|v| v.as_object()).map(|o| o.iter().map(|(k,v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    if new_params.is_empty() { return Err((axum::http::StatusCode::BAD_REQUEST, "params 不能为空".into())); }

    // P2-12: 校验参数键名 — 拒绝未知键
    for key in new_params.keys() {
        if !existing_param_keys.contains(key) {
            return Err((axum::http::StatusCode::BAD_REQUEST, serde_json::json!({
                "error": "invalid_params",
                "message": format!("未知参数: {}", key)
            }).to_string()));
        }
    }

    // P2-12: 校验参数值类型 — 仅允许基本 JSON 类型 (字符串/数字/布尔), 拒绝 null/数组/对象
    for (key, value) in &new_params {
        if value.is_null() || value.is_array() || value.is_object() {
            return Err((axum::http::StatusCode::BAD_REQUEST, serde_json::json!({
                "error": "invalid_params",
                "message": format!("参数 '{}' 的值类型无效: 不允许 null/数组/对象", key)
            }).to_string()));
        }
    }

    state.pending_params.write().map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", e)))?
        .insert(strategy_id.clone(), new_params);
    Ok(Json(serde_json::json!({"status": "pending", "strategy_id": strategy_id})))
}
