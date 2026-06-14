use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures_core::Stream;
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::broadcast;

use super::executor_state::ExecutorState;
use super::now_ms;

pub(super) async fn get_klines(
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

pub(super) async fn strategy_events_sse(
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
