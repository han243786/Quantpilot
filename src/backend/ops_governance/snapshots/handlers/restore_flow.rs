use super::{build_signature_input, load_snapshot_from_disk, persist_snapshot_restore_audit};
use crate::*;

// ── 一键恢复 ──

pub(super) async fn restore_snapshot(
    State(state): State<AppState>,
    Path(snapshot_id): Path<String>,
    Json(request): Json<RestoreSnapshotRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let snapshot = if let Some(s) = state.snapshots.read().await.get(&snapshot_id).cloned() {
        s
    } else {
        load_snapshot_from_disk(&state.snapshot_store_dir, &snapshot_id).await?
    };

    // 验证签名完整性
    let verify_input = build_signature_input(
        &snapshot.capability_hash,
        &snapshot.strategy_version,
        &snapshot.parameter_version,
        &snapshot.core_ir_digest,
        &snapshot.event_slice_bounds,
        snapshot.created_at_ms,
    );
    let current_sig = canonical_json_sha256_digest(&verify_input)
        .map_err(|error| internal_error(anyhow::anyhow!(error)))?
        .value;

    if current_sig != snapshot.signature {
        return Err(json_bad_request(
            "conflict",
            format!("快照 '{}' 完整性校验失败", snapshot_id),
        ));
    }

    let now_ms = current_time_ms();
    persist_snapshot_restore_audit(&state.audit_store_dir, &snapshot, &request, now_ms)
        .await
        .map_err(io_error)?;
    let result = json!({
        "restored_snapshot_id": snapshot_id,
        "deployment_revision": snapshot.deployment_revision,
        "strategy_version": snapshot.strategy_version,
        "parameter_version": snapshot.parameter_version,
        "restored_at_ms": now_ms,
        "restored_by": request.actor_id,
        "reason": request.reason.clone().unwrap_or_default(),
        "status": "restored",
        "warning": "恢复操作已记录审计日志，请在观察窗口(60s)内确认系统正常"
    });

    safe_eprintln!(
        "[snapshot_service] 快照 {} 由 {} 在 {} 恢复",
        snapshot_id,
        request.actor_id,
        now_ms
    );

    // v2.1.0: 恢复操作实际停止当前运行时并记录审计日志
    state
        .runs
        .write()
        .await
        .retain(|_, r| r.created_at_ms > now_ms);
    state
        .backtests
        .write()
        .await
        .retain(|_, r| r.created_at_ms > now_ms);
    safe_eprintln!(
        "[snapshot_service] 快照 {} 恢复: 已清理过期的运行时记录和回测记录",
        snapshot_id
    );

    Ok(Json(result))
}
