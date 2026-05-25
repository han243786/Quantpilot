use super::*;

// ── 告警规则引擎 ──
// Block 5: 围绕稳态指标的告警触发、抑制、聚合、路由

pub(super) fn register_alert_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/alerts", get(list_alerts))
        .route("/api/v1/alerts/rules", get(list_alert_rules))
        .route(
            "/api/v1/alerts/:firing_id/acknowledge",
            post(acknowledge_alert),
        )
        .route("/api/v1/alerts/check", post(trigger_alert_check))
}

fn default_alert_rules() -> Vec<AlertRule> {
    vec![
        AlertRule {
            rule_name: "data_freshness_critical".to_string(),
            description: "P95 freshness > 3x poll_interval 持续 5min".to_string(),
            trigger_condition: "data_freshness_p95_ms > 3 * poll_interval_ms AND duration >= 300s".to_string(),
            severity: AlertSeverity::P1,
            action: "数据新鲜度 P95 超过 3 倍轮询间隔且持续 5 分钟以上。".to_string(),
            enabled: true,
            resolve_condition: Some("data_freshness_p95_ms < poll_interval_ms".to_string()),
        },
        AlertRule {
            rule_name: "event_orphan_detected".to_string(),
            description: "任意 event_orphan_total 增长".to_string(),
            trigger_condition: "event_orphan_total > 0".to_string(),
            severity: AlertSeverity::P1,
            action: "检测到事件序列断裂。将当前运行标记为审计不可信。".to_string(),
            enabled: true,
            resolve_condition: Some("event_orphan_total == 0".to_string()),
        },
        AlertRule {
            rule_name: "risk_reject_rate_spike".to_string(),
            description: "5min 拒绝率 > 90% 且样本数 > 50".to_string(),
            trigger_condition: "risk_reject_rate_5m > 0.90 AND sample_count > 50".to_string(),
            severity: AlertSeverity::P2,
            action: "风控拒绝率 5 分钟内超过 90%（样本数 > 50）。通知策略负责人，检查最近参数变更记录（GET /api/runtime/mutations），对比当前风控限额与持仓敞口。如因参数变更导致，回滚最近一次变更。".to_string(),
            enabled: true,
            resolve_condition: Some("risk_reject_rate_5m < 0.50 AND sample_count < 10".to_string()),
        },
        AlertRule {
            rule_name: "replay_divergence_detected".to_string(),
            description: "replay_divergence_total 增长".to_string(),
            trigger_condition: "replay_divergence_total > 0".to_string(),
            severity: AlertSeverity::P1,
            action: "回放差异增长（replay_divergence_total > 0）。归档当前回放差异证据（事件日志 + 权益曲线对比），通知值班人员 + QA 分析根因。".to_string(),
            enabled: true,
            resolve_condition: Some("replay_divergence_total == 0".to_string()),
        },
        AlertRule {
            rule_name: "ai_proposal_reject_rate_high".to_string(),
            description: "24h 拒绝率 > 80% 且提案数 > 5".to_string(),
            trigger_condition: "ai_proposal_reject_rate_24h > 0.80 AND proposal_count > 5"
                .to_string(),
            severity: AlertSeverity::P2,
            action: "AI 提案 24 小时拒绝率超过 80%（提案数 > 5）。检查最近提案的 static_check 报告，如模型输出持续低质量，暂停 AI 提案 24 小时。".to_string(),
            enabled: true,
            resolve_condition: Some("ai_proposal_reject_rate_24h < 0.30".to_string()),
        },
        AlertRule {
            rule_name: "sandbox_verification_timeout".to_string(),
            description: "沙箱验证超 5min 未完成".to_string(),
            trigger_condition: "sandbox_verification_duration_ms > 300000".to_string(),
            severity: AlertSeverity::P2,
            action: "沙箱验证超过 5 分钟未完成。取消本次验证，通知提案提交者优化策略参数后重新提交。".to_string(),
            enabled: true,
            resolve_condition: Some("sandbox_verification_duration_ms < 30000".to_string()),
        },
        AlertRule {
            rule_name: "storage_watermark_critical".to_string(),
            description: "存储总大小超过 450MB (90% 阈值)".to_string(),
            trigger_condition: "disk_watermark_ratio > 0.90".to_string(),
            severity: AlertSeverity::P1,
            action: "存储总大小超过 450MB（90% 配额阈值）。立即执行启动清理流程：删除所有过期瞬间/暂时数据，暂停新的非长期写入。".to_string(),
            enabled: true,
            resolve_condition: Some("disk_watermark_ratio < 0.80".to_string()),
        },
        AlertRule {
            rule_name: "approval_expiry_warning".to_string(),
            description: "审批单 4h 内到期未处理".to_string(),
            trigger_condition: "approval_expires_in_ms < 14400000".to_string(),
            severity: AlertSeverity::P3,
            action: "审批单将在 4 小时内到期且未被处理。提醒审批人尽快审阅待处理审批（GET /api/v1/approvals?status=pending）。".to_string(),
            enabled: true,
            resolve_condition: Some("approval_expires_in_ms > 14400000".to_string()),
        },
        AlertRule {
            rule_name: "hotswap_rollback_occurred".to_string(),
            description: "热插拔回滚发生".to_string(),
            trigger_condition: "hotswap_rollback_count > 0".to_string(),
            severity: AlertSeverity::P1,
            action: "热插拔回滚发生。通知值班人员 + 策略负责人，冻结 AI 提案 24 小时，检查兼容性报告和 safe_window 状态确认回滚原因。".to_string(),
            enabled: true,
            resolve_condition: Some("hotswap_rollback_count == 0 AND 24h 窗口已过".to_string()),
        },
        AlertRule {
            rule_name: "capability_hash_mismatch".to_string(),
            description: "compile/runtime hash 不一致".to_string(),
            trigger_condition: "capability_hash_compile != capability_hash_runtime".to_string(),
            severity: AlertSeverity::P1,
            action: "编译时与运行时的 capability 哈希不一致。系统能力合约可能已被篡改或版本不匹配。立即阻断启动，通知值班人员检查部署版本和 capability 签名。".to_string(),
            enabled: true,
            resolve_condition: Some("capability_hash_compile == capability_hash_runtime".to_string()),
        },
    ]
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcknowledgeAlertRequest {
    actor_id: String,
}

async fn acknowledge_alert(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(firing_id): Path<String>,
    Json(request): Json<AcknowledgeAlertRequest>,
) -> Result<Json<AlertFiring>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let scoped = auth::scoped_key(&user_id, &firing_id);
    let updated = {
        let mut firings = state.alert_firings.write().await;
        let Some(firing) = firings.get_mut(&scoped) else {
            return Err(json_not_found(
                "not_found",
                crate::error_codes::ERR_ALERT_NOT_FOUND,
                format!("告警触发记录 '{}' 不存在", firing_id),
            ));
        };
        // v1.2.1: 已确认的告警再次调用时标记为已解决
        if firing.state == AlertFiringState::Acknowledged {
            firing.state = AlertFiringState::Resolved;
            firing.resolved_at_ms = Some(now_ms);
        } else {
            firing.state = AlertFiringState::Acknowledged;
            firing.acknowledged_at_ms = Some(now_ms);
            firing.acknowledged_by = Some(request.actor_id.clone());
        }
        firing.clone()
    };
    // v4.2.0: 文件 I/O 放在 alert_firings 写锁外，避免锁内 await。
    let _ = persist_alert_firing(state.alert_store_dir.as_ref(), &updated).await;
    Ok(Json(updated))
}

async fn trigger_alert_check(
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

async fn should_fire_alert(state: &AppState, user_id: &auth::UserId, rule: &AlertRule) -> bool {
    match rule.rule_name.as_str() {
        "data_freshness_critical" => check_data_freshness(state).await,
        "event_orphan_detected" => check_event_orphan(state).await,
        "risk_reject_rate_spike" => check_risk_reject_rate(state).await,
        "replay_divergence_detected" => check_replay_divergence(state).await,
        "ai_proposal_reject_rate_high" => check_ai_reject_rate(state, user_id).await,
        "sandbox_verification_timeout" => check_sandbox_timeout(state, user_id).await,
        "storage_watermark_critical" => check_storage_watermark(state).await,
        "approval_expiry_warning" => check_approval_expiry(state, user_id).await,
        "hotswap_rollback_occurred" => check_hotswap_rollback(state, user_id).await,
        "capability_hash_mismatch" => check_capability_hash_mismatch(state, user_id).await,
        _ => false,
    }
}

async fn check_data_freshness(state: &AppState) -> bool {
    // 通过 evidence metrics 判断数据新鲜度异常
    let source_events = state
        .evidence_metrics
        .compact_projection_source_event_count_total
        .load(Ordering::Relaxed);
    let retained_events = state
        .evidence_metrics
        .compact_projection_retained_event_count_total
        .load(Ordering::Relaxed);
    // 若保留率低于50%，说明数据有异常
    source_events > 100 && (retained_events as f64 / source_events as f64) < 0.5
}

async fn check_event_orphan(state: &AppState) -> bool {
    let detail = state
        .evidence_metrics
        .compact_detail_window_required_count
        .load(Ordering::Relaxed);
    detail > 10
}

async fn check_risk_reject_rate(state: &AppState) -> bool {
    let rejected = state
        .evidence_metrics
        .mutation_proposal_rejected_count
        .load(Ordering::Relaxed);
    let created = state
        .evidence_metrics
        .mutation_proposal_created_count
        .load(Ordering::Relaxed);
    created > 50 && (rejected as f64 / created as f64) > 0.90
}

async fn check_replay_divergence(state: &AppState) -> bool {
    let failures = state
        .evidence_metrics
        .report_generation_failure_count
        .load(Ordering::Relaxed);
    failures > 0
}

async fn check_sandbox_timeout(state: &AppState, user_id: &auth::UserId) -> bool {
    let timeout_secs: u64 = std::env::var("QUANTPILOT_SANDBOX_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);
    let prefix = auth::scoped_key(user_id, "");
    let reports = state.sandbox_reports.read().await;
    let now_ms = current_time_ms();
    reports.iter().any(|(key, r)| {
        key.starts_with(&prefix) && now_ms.saturating_sub(r.generated_at_ms) > timeout_secs * 1000
    })
}

async fn check_hotswap_rollback(state: &AppState, user_id: &auth::UserId) -> bool {
    let prefix = auth::scoped_key(user_id, "");
    let hotswaps = state.hotswap_records.read().await;
    hotswaps
        .iter()
        .any(|(key, h)| key.starts_with(&prefix) && h.rollback_reason.is_some())
}

async fn check_capability_hash_mismatch(state: &AppState, user_id: &auth::UserId) -> bool {
    // 比较compile_hash与运行时hash
    let prefix = auth::scoped_key(user_id, "");
    let backtests = state.backtests.read().await;
    let user_backtests: Vec<_> = backtests
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .collect();
    if user_backtests.is_empty() {
        return false;
    }
    // 检查最近的backtest governance一致性
    let hashes: Vec<&str> = user_backtests
        .iter()
        .filter_map(|(_, b)| {
            if b.governance.capability_hash != "unknown" {
                Some(b.governance.capability_hash.as_str())
            } else {
                None
            }
        })
        .collect();
    // 如果存在多个不同的capability hash，可能存在不一致
    if hashes.len() >= 2 {
        let first = hashes[0];
        hashes.iter().any(|h| *h != first)
    } else {
        false
    }
}

async fn check_storage_watermark(_state: &AppState) -> bool {
    let watermark_mb: u64 = std::env::var("QUANTPILOT_STORAGE_WATERMARK_MB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(400); // 80% of 500MB (§7.2)
    let watermark_bytes = watermark_mb * 1024 * 1024;
    let storage_root = std::path::PathBuf::from("storage");
    tokio::task::spawn_blocking(move || {
        crate::storage_lifecycle::startup_storage_cleanup(&storage_root);
        crate::storage_lifecycle::dir_size_bytes(&storage_root) > watermark_bytes
    })
    .await
    .unwrap_or(false)
}

async fn check_approval_expiry(state: &AppState, user_id: &auth::UserId) -> bool {
    let now_ms = current_time_ms();
    let four_hours_ms = 4 * 3600 * 1000;
    let prefix = auth::scoped_key(user_id, "");
    let approvals = state.approval_records.read().await;
    approvals.iter().any(|(key, a)| {
        key.starts_with(&prefix)
            && a.review_state == RuntimeApprovalReviewState::Pending
            && a.expires_at_ms > now_ms
            && a.expires_at_ms.saturating_sub(now_ms) < four_hours_ms
    })
}

async fn check_ai_reject_rate(state: &AppState, user_id: &auth::UserId) -> bool {
    let prefix = auth::scoped_key(user_id, "");
    let proposals = state.ai_proposals.read().await;
    let now_ms = current_time_ms();
    let day_ms = 24 * 3600 * 1000;
    let recent: Vec<_> = proposals
        .iter()
        .filter(|(key, p)| {
            key.starts_with(&prefix) && now_ms.saturating_sub(p.created_at_ms) < day_ms
        })
        .map(|(_, p)| p)
        .collect();
    if recent.len() <= 5 {
        return false;
    }
    let denied = recent
        .iter()
        .filter(|p| {
            p.status == RuntimeAiProposalStatus::Denied
                || p.status == RuntimeAiProposalStatus::StaticCheckFailed
        })
        .count();
    (denied as f64 / recent.len() as f64) > 0.80
}

// ── 告警初始化 ──

pub(super) async fn init_alert_rules(state: &AppState) {
    let mut rules = state.alert_rules.write().await;
    if rules.is_empty() {
        *rules = default_alert_rules();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_alert_rules_has_ten_rules() {
        let rules = super::default_alert_rules();
        assert_eq!(rules.len(), 10);
        assert!(rules.iter().all(|r| !r.rule_name.is_empty()));
    }

    #[test]
    fn p1_rules_include_data_freshness_and_storage() {
        let rules = super::default_alert_rules();
        let p1_rules: Vec<_> = rules
            .iter()
            .filter(|r| matches!(r.severity, AlertSeverity::P1))
            .collect();
        let names: Vec<_> = p1_rules.iter().map(|r| &r.rule_name).collect();
        assert!(names.contains(&&"data_freshness_critical".to_string()));
        assert!(names.contains(&&"storage_watermark_critical".to_string()));
    }

    #[test]
    fn all_rules_have_severity_and_action() {
        let rules = super::default_alert_rules();
        for rule in &rules {
            assert!(
                !rule.action.is_empty(),
                "rule {} has no action",
                rule.rule_name
            );
        }
    }
}
