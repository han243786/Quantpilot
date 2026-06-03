use super::snapshot_id_validation;
use crate::*;

pub(super) async fn persist_snapshot_restore_audit(
    audit_store_dir: &FsPath,
    snapshot: &DeploymentSignatureSnapshot,
    request: &RestoreSnapshotRequest,
    restored_at_ms: u64,
) -> std::io::Result<()> {
    fs::create_dir_all(audit_store_dir).await?;
    let path = audit_store_dir.join(format!(
        "snapshot-restore-{}-{}.json",
        snapshot.snapshot_id, restored_at_ms
    ));
    let entry = json!({
        "event_type": "snapshot_restore",
        "snapshot_id": snapshot.snapshot_id,
        "deployment_revision": snapshot.deployment_revision,
        "strategy_version": snapshot.strategy_version,
        "parameter_version": snapshot.parameter_version,
        "actor_id": request.actor_id,
        "reason": request.reason.clone().unwrap_or_default(),
        "restored_at_ms": restored_at_ms,
    });
    crate::runtime_persistence::atomic_write_json(&path, &entry).await
}

pub(super) async fn persist_snapshot(
    store_dir: &FsPath,
    snapshot: &DeploymentSignatureSnapshot,
) -> std::io::Result<()> {
    crate::storage_lifecycle::ensure_storage_quota(
        std::path::Path::new("storage"),
        "snapshots",
        crate::storage_lifecycle::StorageLifecycle::Transient,
    )?;
    fs::create_dir_all(&store_dir).await?;
    let file_path = store_dir.join(format!("{}.json", snapshot.snapshot_id));
    // v2.3.3: 使用统一原子写入 (含 fsync)
    crate::runtime_persistence::atomic_write_json(&file_path, snapshot).await
}

pub(super) async fn load_snapshot_from_disk(
    store_dir: &FsPath,
    snapshot_id: &str,
) -> Result<DeploymentSignatureSnapshot, (StatusCode, String)> {
    if let Err(msg) = snapshot_id_validation::validate_snapshot_id(snapshot_id) {
        return Err(json_bad_request("invalid_snapshot_id", msg));
    }
    let file_path = store_dir.join(format!("{}.json", snapshot_id));
    let json = fs::read(&file_path)
        .await
        .map_err(|_| json_bad_request("not_found", format!("快照 '{}' 不存在", snapshot_id)))?;
    serde_json::from_slice(&json).map_err(|error| internal_error(anyhow::anyhow!("{}", error)))
}
