use crate::*;

mod acknowledge_flow;
mod persistence;
mod predicate_checks;
mod read_routes;
mod rule_catalog;
mod startup_initialization;
mod trigger_engine;

// ── 告警规则引擎 ──
// Block 5: 围绕稳态指标的告警触发、抑制、聚合、路由

pub(super) fn register_alert_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/alerts", get(read_routes::list_alerts))
        .route("/api/v1/alerts/rules", get(read_routes::list_alert_rules))
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

async fn should_fire_alert(state: &AppState, user_id: &auth::UserId, rule: &AlertRule) -> bool {
    predicate_checks::should_fire_alert(state, user_id, rule).await
}

pub(super) async fn init_alert_rules(state: &AppState) {
    startup_initialization::init_alert_rules(state, rule_catalog::default_alert_rules).await
}

async fn persist_alert_firing(store_dir: &FsPath, firing: &AlertFiring) -> std::io::Result<()> {
    persistence::persist_alert_firing(store_dir, firing).await
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
