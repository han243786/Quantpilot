use crate::*;

pub(super) async fn persist_alert_firing(
    store_dir: &FsPath,
    firing: &AlertFiring,
) -> std::io::Result<()> {
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
