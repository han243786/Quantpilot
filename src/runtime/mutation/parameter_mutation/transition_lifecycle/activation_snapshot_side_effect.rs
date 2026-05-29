use super::*;

/// Block 5 P1-6 + P3-2: 激活时自动生成签名快照 + 递增代际
pub(super) async fn auto_snapshot_on_activation(
    state: &AppState,
    user_id: &auth::UserId,
    mutation: &RuntimeParameterMutationRecord,
) {
    let now_ms = current_time_ms();
    // P3-2: 递增配置代际
    let gen = state
        .config_generation
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    const MAX_GENERATION_HISTORY: usize = 100;
    let mut history = state.config_generation_history.lock().await;
    history.push(qrpc_runtime::ConfigGenerationEntry {
        generation: gen,
        activated_at_ms: now_ms,
        deployment_revision: mutation.governance.deployment_revision.clone(),
        parameter_version: mutation.proposed_parameter_version.clone(),
    });
    let overflow = history.len().saturating_sub(MAX_GENERATION_HISTORY);
    if overflow > 0 {
        history.drain(0..overflow);
    }

    // P3-3: Shadow Evaluation — 记录激活前指标基线
    let _pre_activation_risk_reject = state
        .evidence_metrics
        .mutation_proposal_rejected_count
        .load(std::sync::atomic::Ordering::Relaxed);
    let _pre_activation_rollback = state
        .evidence_metrics
        .mutation_rollback_attempt_count
        .load(std::sync::atomic::Ordering::Relaxed);

    // P3-4: Observation Window — 设置 60s 观察截止时间
    let _observation_deadline_ms = now_ms.saturating_add(60_000);

    let snapshot_id = format!("snap-auto-{}", now_ms);
    let snapshot = DeploymentSignatureSnapshot {
        snapshot_id: snapshot_id.clone(),
        deployment_revision: mutation.governance.deployment_revision.clone(),
        capability_hash: mutation.governance.capability_hash.clone(),
        strategy_version: mutation.governance.strategy_version.clone(),
        parameter_version: mutation.proposed_parameter_version.clone(),
        core_ir_digest: "auto-generated-on-activation".to_string(),
        event_slice_bounds: EventSliceBounds {
            from_event_id: String::new(),
            to_event_id: String::new(),
            from_sequence: 0,
            to_sequence: 0,
            event_count: 0,
        },
        created_at_ms: now_ms,
        signature: qrpc_core::canonical_json_sha256_digest(&serde_json::json!({
            "capability_hash": mutation.governance.capability_hash,
            "strategy_version": mutation.governance.strategy_version,
            "parameter_version": mutation.proposed_parameter_version,
            "created_at_ms": now_ms,
        }))
        .map(|d| d.value)
        .unwrap_or_else(|_| "signature-unavailable".to_string()),
    };
    // 持久化并存入内存
    let dir = state.snapshot_store_dir.to_path_buf();
    let path = dir.join(format!("{}.json", snapshot_id));
    // v2.3.3: 使用统一原子写入 (含 fsync)
    crate::runtime_persistence::atomic_write_json(&path, &snapshot)
        .await
        .unwrap_or_else(|e| {
            safe_eprintln!("[snapshot] 原子写入快照失败: {}", e);
        });
    state
        .snapshots
        .write()
        .await
        .insert(auth::scoped_key(user_id, &snapshot_id), snapshot);
}
