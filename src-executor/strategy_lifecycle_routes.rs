use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use super::executor_state::{ExecutionMode, ExecutorState, StrategyStatus};
use super::provider_order_support::load_okx_demo_credentials;
use super::v4_deploy_support::{
    deploy_v4_strategy, ensure_strategy_config_preflight_allows_start, is_v4_deploy_request,
    json_path_str,
};
use super::{append_audit, migration_api};

pub(super) async fn list_strategies(
    State(state): State<Arc<ExecutorState>>,
) -> Json<serde_json::Value> {
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

pub(super) async fn get_strategy_detail(
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

pub(super) async fn recv_strategy(
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

pub(super) async fn start_strategy(
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

pub(super) async fn stop_strategy(
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
