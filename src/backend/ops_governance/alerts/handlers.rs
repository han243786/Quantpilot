use crate::*;

mod acknowledge_flow;
mod predicate_checks;
mod rule_catalog;
mod trigger_engine;

// ── 告警规则引擎 ──
// Block 5: 围绕稳态指标的告警触发、抑制、聚合、路由

pub(super) fn register_alert_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/alerts", get(list_alerts))
        .route("/api/v1/alerts/rules", get(list_alert_rules))
        .route(
            "/api/v1/alerts/:firing_id/acknowledge",
            post(acknowledge_flow::acknowledge_alert),
        )
        .route(
            "/api/v1/alerts/check",
            post(trigger_engine::trigger_alert_check),
        )
}

// ── API 处理函数 ──

async fn list_alerts(
    user_id: auth::UserId,
    State(state): State<AppState>,
) -> Result<Json<AlertListResponse>, (StatusCode, String)> {
    let prefix = auth::scoped_key(&user_id, "");
    let firings: Vec<AlertFiring> = state
        .alert_firings
        .read()
        .await
        .iter()
        .filter(|(key, _)| key.starts_with(&prefix))
        .map(|(_, value)| value.clone())
        .collect();
    let rules = state.alert_rules.read().await.clone();
    Ok(Json(AlertListResponse { firings, rules }))
}

async fn list_alert_rules(
    State(state): State<AppState>,
) -> Result<Json<Vec<AlertRule>>, (StatusCode, String)> {
    let rules = state.alert_rules.read().await.clone();
    Ok(Json(rules))
}

async fn should_fire_alert(state: &AppState, user_id: &auth::UserId, rule: &AlertRule) -> bool {
    predicate_checks::should_fire_alert(state, user_id, rule).await
}

pub(super) async fn init_alert_rules(state: &AppState) {
    let mut rules = state.alert_rules.write().await;
    if rules.is_empty() {
        *rules = rule_catalog::default_alert_rules();
    }
}

async fn persist_alert_firing(store_dir: &FsPath, firing: &AlertFiring) -> std::io::Result<()> {
    crate::storage_lifecycle::ensure_storage_quota(
        std::path::Path::new("storage"),
        "alerts",
        crate::storage_lifecycle::StorageLifecycle::Transient,
    )?;
    fs::create_dir_all(&store_dir).await?;
    let file_path = store_dir.join(format!("{}.json", firing.firing_id));
    // v2.3.3: 使用统一原子写入 (含 fsync)
    crate::runtime_persistence::atomic_write_json(&file_path, firing).await
}

/// v3.5.0 §9.3: 检查告警恢复条件
/// 核心原则: 触发条件不再成立时告警自动恢复
/// resolve_condition 字段作为人类可读文档描述"已恢复"的含义。
/// 当前实现采用触发条件取反策略: 触发条件不成立即视为已恢复。
/// 这是正确的语义, 因为若触发条件已不再满足则问题已解决。
async fn is_condition_resolved(state: &AppState, user_id: &auth::UserId, rule: &AlertRule) -> bool {
    // 告警恢复 = 触发条件不再成立
    !should_fire_alert(state, user_id, rule).await
}
