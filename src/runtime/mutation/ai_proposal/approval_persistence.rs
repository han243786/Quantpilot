use super::*;

pub(super) async fn persist_approval(
    store_dir: &FsPath,
    approval: &RuntimeApprovalRecord,
) -> std::io::Result<()> {
    fs::create_dir_all(store_dir).await?;
    let file_path = store_dir.join(format!("{}.json", approval.approval_id));
    // v2.3.3: 使用统一原子写入 (含 fsync)
    crate::runtime_persistence::atomic_write_json(&file_path, approval).await
}

pub(super) async fn load_approval_from_disk(
    store_dir: &FsPath,
    approval_id: &str,
) -> Result<RuntimeApprovalRecord, (StatusCode, String)> {
    let file_path = store_dir.join(format!("{}.json", approval_id));
    let json = fs::read(&file_path)
        .await
        .map_err(|_| json_bad_request("not_found", format!("审批单 '{}' 不存在", approval_id)))?;
    serde_json::from_slice(&json).map_err(|error| internal_error(anyhow::anyhow!("{}", error)))
}
