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
            trigger_condition: "data_freshness_p95_ms > 3 * poll_interval_ms AND duration >= 300s"
                .to_string(),
            severity: AlertSeverity::P1,
            action: "暂停 Execution 产出，通知值班".to_string(),
            enabled: true,
        },
        AlertRule {
            rule_name: "event_orphan_detected".to_string(),
            description: "任意 event_orphan_total 增长".to_string(),
            trigger_condition: "event_orphan_total > 0".to_string(),
            severity: AlertSeverity::P1,
            action: "标记 run 为审计不可信，通知值班".to_string(),
            enabled: true,
        },
        AlertRule {
            rule_name: "risk_reject_rate_spike".to_string(),
            description: "5min 拒绝率 > 90% 且样本数 > 50".to_string(),
            trigger_condition: "risk_reject_rate_5m > 0.90 AND sample_count > 50".to_string(),
            severity: AlertSeverity::P2,
            action: "通知策略负责人，检查参数是否异常".to_string(),
            enabled: true,
        },
        AlertRule {
            rule_name: "replay_divergence_detected".to_string(),
            description: "replay_divergence_total 增长".to_string(),
            trigger_condition: "replay_divergence_total > 0".to_string(),
            severity: AlertSeverity::P1,
            action: "归档差异证据，通知值班 + QA".to_string(),
            enabled: true,
        },
        AlertRule {
            rule_name: "ai_proposal_reject_rate_high".to_string(),
            description: "24h 拒绝率 > 80% 且提案数 > 5".to_string(),
            trigger_condition: "ai_proposal_reject_rate_24h > 0.80 AND proposal_count > 5"
                .to_string(),
            severity: AlertSeverity::P2,
            action: "检查 AI 模型输出质量，考虑冻结提案".to_string(),
            enabled: true,
        },
        AlertRule {
            rule_name: "sandbox_verification_timeout".to_string(),
            description: "沙箱验证超 5min 未完成".to_string(),
            trigger_condition: "sandbox_verification_duration_ms > 300000".to_string(),
            severity: AlertSeverity::P2,
            action: "取消本次验证，通知提交者重试".to_string(),
            enabled: true,
        },
        AlertRule {
            rule_name: "storage_watermark_critical".to_string(),
            description: "磁盘水位 > 90%".to_string(),
            trigger_condition: "disk_watermark_ratio > 0.90".to_string(),
            severity: AlertSeverity::P1,
            action: "强制降级：关 debug -> 采样 DataUpdated -> 暂停新 run".to_string(),
            enabled: true,
        },
        AlertRule {
            rule_name: "approval_expiry_warning".to_string(),
            description: "审批单 4h 内到期未处理".to_string(),
            trigger_condition: "approval_expires_in_ms < 14400000".to_string(),
            severity: AlertSeverity::P3,
            action: "提醒审批人".to_string(),
            enabled: true,
        },
        AlertRule {
            rule_name: "hotswap_rollback_occurred".to_string(),
            description: "热插拔回滚发生".to_string(),
            trigger_condition: "hotswap_rollback_count > 0".to_string(),
            severity: AlertSeverity::P1,
            action: "通知值班 + 策略负责人，冻结 AI 提案 24h".to_string(),
            enabled: true,
        },
        AlertRule {
            rule_name: "capability_hash_mismatch".to_string(),
            description: "compile/runtime hash 不一致".to_string(),
            trigger_condition: "capability_hash_compile != capability_hash_runtime".to_string(),
            severity: AlertSeverity::P1,
            action: "阻断启动，通知值班".to_string(),
            enabled: true,
        },
    ]
}

// ── API 处理函数 ──

async fn list_alerts(
    State(state): State<AppState>,
) -> Result<Json<AlertListResponse>, (StatusCode, String)> {
    let firings: Vec<AlertFiring> = state
        .alert_firings
        .read()
        .await
        .values()
        .cloned()
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
struct AcknowledgeAlertRequest {
    actor_id: String,
}

async fn acknowledge_alert(
    State(state): State<AppState>,
    Path(firing_id): Path<String>,
    Json(request): Json<AcknowledgeAlertRequest>,
) -> Result<Json<AlertFiring>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let mut firings = state.alert_firings.write().await;
    if let Some(firing) = firings.get_mut(&firing_id) {
        firing.state = AlertFiringState::Acknowledged;
        firing.acknowledged_at_ms = Some(now_ms);
        firing.acknowledged_by = Some(request.actor_id.clone());
        let updated = firing.clone();
        // 持久化告警状态变更
        let _ = persist_alert_firing(state.alert_store_dir.as_ref(), &updated).await;
        return Ok(Json(updated));
    }
    Err(json_bad_request(
        "not_found",
        format!("告警触发记录 '{}' 不存在", firing_id),
    ))
}

async fn trigger_alert_check(
    State(state): State<AppState>,
) -> Result<Json<Vec<AlertFiring>>, (StatusCode, String)> {
    let rules = state.alert_rules.read().await.clone();
    let now_ms = current_time_ms();
    let mut new_firings = Vec::new();

    for rule in &rules {
        if !rule.enabled {
            continue;
        }
        if should_fire_alert(&state, rule).await {
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
            new_firings.push(firing.clone());
            state
                .alert_firings
                .write()
                .await
                .insert(firing_id.clone(), firing.clone());
            // 持久化告警状态
            let _ = persist_alert_firing(state.alert_store_dir.as_ref(), &firing).await;
        }
    }

    Ok(Json(new_firings))
}

async fn should_fire_alert(state: &AppState, rule: &AlertRule) -> bool {
    match rule.rule_name.as_str() {
        "data_freshness_critical" => check_data_freshness(state).await,
        "event_orphan_detected" => check_event_orphan(state).await,
        "risk_reject_rate_spike" => check_risk_reject_rate(state).await,
        "replay_divergence_detected" => check_replay_divergence(state).await,
        "ai_proposal_reject_rate_high" => check_ai_reject_rate(state).await,
        "sandbox_verification_timeout" => check_sandbox_timeout(state).await,
        "storage_watermark_critical" => check_storage_watermark(state).await,
        "approval_expiry_warning" => check_approval_expiry(state).await,
        "hotswap_rollback_occurred" => check_hotswap_rollback(state).await,
        "capability_hash_mismatch" => check_capability_hash_mismatch(state).await,
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

async fn check_sandbox_timeout(state: &AppState) -> bool {
    let timeout_secs: u64 = std::env::var("QUANTPILOT_SANDBOX_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);
    let reports = state.sandbox_reports.read().await;
    let now_ms = current_time_ms();
    reports.values().any(|r| now_ms.saturating_sub(r.generated_at_ms) > timeout_secs * 1000)
}

async fn check_hotswap_rollback(state: &AppState) -> bool {
    let hotswaps = state.hotswap_records.read().await;
    hotswaps.values().any(|h| {
        h.rollback_reason.is_some()
    })
}

async fn check_capability_hash_mismatch(state: &AppState) -> bool {
    // 比较compile_hash与运行时hash
    let backtests = state.backtests.read().await;
    if backtests.is_empty() {
        return false;
    }
    // 检查最近的backtest governance一致性
    let hashes: Vec<&str> = backtests
        .values()
        .filter_map(|b| {
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

async fn check_storage_watermark(state: &AppState) -> bool {
    let watermark_mb: u64 = std::env::var("QUANTPILOT_STORAGE_WATERMARK_MB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(90);
    let watermark_bytes = watermark_mb * 1024 * 1024;
    let dir = state.report_store_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        compute_dir_size_sync(&dir).unwrap_or(0) > watermark_bytes
    })
    .await
    .unwrap_or(false)
}

fn compute_dir_size_sync(dir: &std::path::Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len();
                } else if metadata.is_dir() {
                    total += compute_dir_size_sync(&entry.path()).unwrap_or(0);
                }
            }
        }
    }
    Ok(total)
}

async fn check_approval_expiry(state: &AppState) -> bool {
    let now_ms = current_time_ms();
    let four_hours_ms = 4 * 3600 * 1000;
    let approvals = state.approval_records.read().await;
    approvals.values().any(|a| {
        a.review_state == RuntimeApprovalReviewState::Pending
            && a.expires_at_ms > now_ms
            && a.expires_at_ms.saturating_sub(now_ms) < four_hours_ms
    })
}

async fn check_ai_reject_rate(state: &AppState) -> bool {
    let proposals = state.ai_proposals.read().await;
    let now_ms = current_time_ms();
    let day_ms = 24 * 3600 * 1000;
    let recent: Vec<_> = proposals
        .values()
        .filter(|p| now_ms.saturating_sub(p.created_at_ms) < day_ms)
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
    let json = serde_json::to_vec_pretty(firing)?;
    fs::create_dir_all(store_dir).await?;
    let file_path = store_dir.join(format!("{}.json", firing.firing_id));
    fs::write(&file_path, &json).await?;
    Ok(())
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
            assert!(!rule.action.is_empty(), "rule {} has no action", rule.rule_name);
        }
    }
}
