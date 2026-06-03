use super::{is_condition_resolved, persist_alert_firing, should_fire_alert};
use crate::*;

pub(super) async fn trigger_alert_check(
    user_id: auth::UserId,
    State(state): State<AppState>,
) -> Result<Json<Vec<AlertFiring>>, (StatusCode, String)> {
    let rules = state.alert_rules.read().await.clone();
    let now_ms = current_time_ms();
    let mut new_firings = Vec::new();

    for rule in &rules {
        if !rule.enabled {
            continue;
        }
        // v2.3.5: 告警去重 — 已存在同规则名的未处理告警则跳过 (持写锁防 TOCTOU)
        let firing = {
            let mut firings = state.alert_firings.write().await;
            let already_firing = firings.values().any(|f| {
                f.rule_name == rule.rule_name && matches!(f.state, AlertFiringState::Firing)
            });
            if already_firing {
                continue;
            }
            if should_fire_alert(&state, &user_id, rule).await {
                let firing_id = format!("alert-{}-{}", rule.rule_name, now_ms);
                let firing = AlertFiring {
                    firing_id: firing_id.clone(),
                    rule_name: rule.rule_name.clone(),
                    severity: rule.severity,
                    state: AlertFiringState::Firing,
                    fired_at_ms: now_ms,
                    acknowledged_at_ms: None,
                    resolved_at_ms: None,
                    acknowledged_by: None,
                    detail: format!("{}: {}", rule.description, rule.action),
                };
                firings.insert(auth::scoped_key(&user_id, &firing_id), firing.clone());
                firing
            } else {
                continue;
            }
        }; // write lock dropped
        new_firings.push(firing.clone());
        // 持久化告警状态 (no lock)
        let _ = persist_alert_firing(state.alert_store_dir.as_ref(), &firing).await;
    }

    // v3.5.0 §9.3: 自动恢复 — 触发条件不再成立时自动 Resolved (两阶段: 先检查再 I/O)
    for rule in &rules {
        if rule.resolve_condition.is_none() && rule.rule_name != "event_orphan_detected" {
            continue; // 跳过未配置恢复条件的规则 (一次性事件类告警需手动确认)
        }
        // Phase 1: 检查恢复条件 (no lock)
        if !is_condition_resolved(&state, &user_id, rule).await {
            continue;
        }
        // Phase 1b: 收集待恢复 key (read lock, short)
        let to_resolve: Vec<String> = {
            let firings = state.alert_firings.read().await;
            firings
                .iter()
                .filter(|(_, f)| {
                    f.rule_name == rule.rule_name && f.state == AlertFiringState::Firing
                })
                .map(|(k, _)| k.clone())
                .collect()
        };
        if to_resolve.is_empty() {
            continue;
        }
        // Phase 2: 更新状态 (write lock, short hold)
        let resolved_firings: Vec<AlertFiring> = {
            let mut firings = state.alert_firings.write().await;
            to_resolve
                .iter()
                .filter_map(|key| {
                    let f = firings.get_mut(key)?;
                    f.state = AlertFiringState::Resolved;
                    f.resolved_at_ms = Some(current_time_ms());
                    Some(f.clone())
                })
                .collect()
        };
        // Phase 3: 持久化 (no lock)
        for f in &resolved_firings {
            let _ = persist_alert_firing(state.alert_store_dir.as_ref(), f).await;
        }
    }

    // v2.1.0: 清理已解决的告警记录，防止无限增长
    // P2-6: 先收集已解决告警的 key, 删除内存记录后同时清理磁盘文件
    let resolved_keys: Vec<String> = {
        let firings = state.alert_firings.read().await;
        firings
            .iter()
            .filter(|(_, f)| f.state == AlertFiringState::Resolved)
            .map(|(k, _)| k.clone())
            .collect()
    };

    state
        .alert_firings
        .write()
        .await
        .retain(|_, firing| firing.state != AlertFiringState::Resolved);

    // P2-6: 删除已解决告警对应的磁盘文件
    for key in &resolved_keys {
        let file_path = state.alert_store_dir.join(format!("{}.json", key));
        let _ = tokio::fs::remove_file(&file_path).await;
    }

    Ok(Json(new_firings))
}
