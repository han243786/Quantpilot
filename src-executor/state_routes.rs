use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::collections::BTreeMap;
use std::sync::Arc;

use super::audit_log;
use super::executor_state::{ExecutionMode, ExecutorState, StrategyStatus};

pub(super) async fn health_check(
    State(state): State<Arc<ExecutorState>>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "executor_ok",
        "mode": state.current_mode().as_str(),
        "sse_lagged_event_count": state.sse_lagged_count(),
    }))
}

// v3.5.0: 全局执行模式查询与切换
pub(super) async fn get_mode(State(state): State<Arc<ExecutorState>>) -> Json<serde_json::Value> {
    let mode = state.current_mode();
    Json(serde_json::json!({
        "mode": mode.as_str(),
        "mode_label": mode.display_label(),
        "available_modes": ExecutionMode::available_mode_keys(),
        "deferred_modes": ["live_simulated", "live_actual"]
    }))
}

#[derive(serde::Deserialize)]
pub(super) struct SetModeRequest {
    mode: String,
}

pub(super) async fn set_mode(
    State(state): State<Arc<ExecutorState>>,
    Json(req): Json<SetModeRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let new_mode = match ExecutionMode::from_api_label(&req.mode) {
        Some(mode) => mode,
        None if req.mode.eq_ignore_ascii_case("live")
            || req.mode.eq_ignore_ascii_case("live_actual")
            || req.mode.eq_ignore_ascii_case("live_simulated") =>
        {
            return Err((
                StatusCode::BAD_REQUEST,
                serde_json::json!({
                    "error": "deferred_mode",
                    "message": "真实资金或真实账户上下文模式已延后；v4.8.0 仅允许 paper_simulated / paper_actual",
                    "available_modes": ExecutionMode::available_mode_keys(),
                    "deferred_modes": ["live_simulated", "live_actual"]
                })
                .to_string(),
            ));
        }
        None => {
            let other = req.mode.as_str();
            return Err((
                StatusCode::BAD_REQUEST,
                serde_json::json!({
                    "error": "invalid_mode",
                    "message": format!("不支持的模式: '{}', 仅支持 paper_simulated / paper_actual", other),
                    "available_modes": ExecutionMode::available_mode_keys(),
                    "deferred_modes": ["live_simulated", "live_actual"]
                })
                .to_string(),
            ));
        }
    };
    let old_mode = state.set_mode(new_mode);
    let mode_str = new_mode.as_str();
    eprintln!("[executor] 模式切换: {:?} → {:?}", old_mode, new_mode);
    append_audit(
        &state,
        "set_mode",
        None,
        serde_json::json!({
            "previous_mode": old_mode.as_str(),
            "current_mode": mode_str,
            "current_mode_label": new_mode.display_label(),
            "provider_order_submission_attached": new_mode.provider_order_submission_attached(),
        }),
    );
    Ok(Json(serde_json::json!({
        "previous_mode": old_mode.as_str(),
        "current_mode": mode_str,
        "current_mode_label": new_mode.display_label(),
        "message": format!("执行端已切换到 {} 模式", new_mode.display_label())
    })))
}

pub(super) async fn get_params(
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

pub(super) async fn update_params(
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
