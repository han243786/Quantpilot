use super::*;

#[path = "transition_lifecycle/activation_flow.rs"]
mod activation_flow;
#[path = "transition_lifecycle/boundary_safety.rs"]
mod boundary_safety;
#[path = "transition_lifecycle/rollback_flow.rs"]
mod rollback_flow;

pub(crate) use activation_flow::activate_runtime_parameter_mutation;
pub(crate) use rollback_flow::rollback_runtime_parameter_mutation;

use boundary_safety::{
    evaluate_runtime_parameter_mutation_safe_window, resolve_runtime_parameter_mutation_boundary,
};

pub(super) fn validate_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
) -> Result<(), (StatusCode, String)> {
    boundary_safety::validate_runtime_parameter_mutation_boundary(boundary)
}

fn runtime_parameter_mutation_rollback_record_id(
    source_id: &str,
    rollback_of: &str,
    target: &RuntimeParameterMutationTarget,
    created_at_ms: u64,
    source_event_count: usize,
    proposed_parameter_version: &str,
) -> Result<String, (StatusCode, String)> {
    let digest = canonical_json_sha256_digest(&json!({
        "created_at_ms": created_at_ms,
        "rollback_of": rollback_of,
        "source_event_count": source_event_count,
        "source_id": source_id,
        "target": target,
        "proposed_parameter_version": proposed_parameter_version,
    }))
    .map_err(|error| internal_error(anyhow::anyhow!(error)))?;
    Ok(format!(
        "parameter_rollback_{}_{}",
        created_at_ms,
        &digest.value[..12]
    ))
}

fn mutation_lifecycle_entry(
    status: RuntimeParameterMutationStatus,
    event: &FrontendRuntimeEvent,
    sequence_no: u64,
    message: impl Into<String>,
) -> RuntimeParameterMutationLifecycleEntry {
    let (_, reason_code) = mutation_event_contract(status);
    RuntimeParameterMutationLifecycleEntry {
        status,
        event_id: event.event_id.clone(),
        sequence_no,
        occurred_at_ms: event.event_time_ms,
        reason_code: reason_code.to_string(),
        message: message.into(),
    }
}

async fn persist_runtime_parameter_mutation_transition(
    state: &AppState,
    user_id: &auth::UserId,
    record: &RuntimeParameterMutationRecord,
) -> Result<(), (StatusCode, String)> {
    persist_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), record)
        .await
        .map_err(io_error)?;
    state.parameter_mutations.write().await.insert(
        auth::scoped_key(user_id, &record.proposal_id),
        record.clone(),
    );
    Ok(())
}

/// Block 5 P1-6 + P3-2: 激活时自动生成签名快照 + 递增代际
async fn auto_snapshot_on_activation(
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
