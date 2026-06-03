use crate::*;

pub(super) async fn should_fire_alert(
    state: &AppState,
    user_id: &auth::UserId,
    rule: &AlertRule,
) -> bool {
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
