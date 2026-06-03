use crate::*;
mod create_flow;
mod persistence;
mod read_routes;
mod restore_flow;

mod snapshot_id_validation;

// ── 签名快照服务 ──
// Block 5: deployment_revision 激活时生成不可变签名快照，支持一键恢复

pub(super) fn register_snapshot_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/snapshots", get(read_routes::list_snapshots))
        .route(
            "/api/v1/snapshots/:snapshot_id",
            get(read_routes::get_snapshot),
        )
        .route(
            "/api/v1/snapshots/:snapshot_id/restore",
            post(restore_flow::restore_snapshot),
        )
        .route(
            "/api/v1/snapshots/create",
            post(create_flow::create_snapshot),
        )
}

/// v2.5.0: 共享签名输入构建, 消除创建/验证两侧代码重复
fn build_signature_input(
    capability_hash: &str,
    strategy_version: &str,
    parameter_version: &str,
    core_ir_digest: &str,
    event_slice_bounds: &EventSliceBounds,
    created_at_ms: u64,
) -> serde_json::Value {
    json!({
        "capability_hash": capability_hash,
        "strategy_version": strategy_version,
        "parameter_version": parameter_version,
        "core_ir_digest": core_ir_digest,
        "event_slice_bounds": {
            "from_event_id": &event_slice_bounds.from_event_id,
            "to_event_id": &event_slice_bounds.to_event_id,
            "from_sequence": event_slice_bounds.from_sequence,
            "to_sequence": event_slice_bounds.to_sequence,
            "event_count": event_slice_bounds.event_count,
        },
        "created_at_ms": created_at_ms,
    })
}

// ── 持久化辅助函数 ──

async fn persist_snapshot_restore_audit(
    audit_store_dir: &FsPath,
    snapshot: &DeploymentSignatureSnapshot,
    request: &RestoreSnapshotRequest,
    restored_at_ms: u64,
) -> std::io::Result<()> {
    persistence::persist_snapshot_restore_audit(audit_store_dir, snapshot, request, restored_at_ms)
        .await
}

async fn persist_snapshot(
    store_dir: &FsPath,
    snapshot: &DeploymentSignatureSnapshot,
) -> std::io::Result<()> {
    persistence::persist_snapshot(store_dir, snapshot).await
}

async fn load_snapshot_from_disk(
    store_dir: &FsPath,
    snapshot_id: &str,
) -> Result<DeploymentSignatureSnapshot, (StatusCode, String)> {
    persistence::load_snapshot_from_disk(store_dir, snapshot_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::canonical_json_sha256_digest;

    #[test]
    fn snapshot_signature_is_deterministic() {
        let input = serde_json::json!({
            "capability_hash": "sha256:abc123",
            "strategy_version": "v1",
            "parameter_version": "p1",
            "created_at_ms": 1000u64,
        });
        let sig1 = canonical_json_sha256_digest(&input).unwrap();
        let sig2 = canonical_json_sha256_digest(&input).unwrap();
        assert_eq!(sig1.value, sig2.value);
    }

    #[test]
    fn event_slice_bounds_hold_correct_counts() {
        let bounds = EventSliceBounds {
            from_event_id: "evt_1".to_string(),
            to_event_id: "evt_10".to_string(),
            from_sequence: 1,
            to_sequence: 10,
            event_count: 10,
        };
        assert_eq!(bounds.event_count, 10);
        assert_eq!(bounds.from_sequence, 1);
    }
}
