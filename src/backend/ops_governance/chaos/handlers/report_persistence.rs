use crate::*;

pub(super) async fn persist_chaos_report(
    store_dir: &FsPath,
    report: &ChaosExperimentReport,
) -> std::io::Result<()> {
    crate::storage_lifecycle::ensure_storage_quota(
        std::path::Path::new("storage"),
        "chaos",
        crate::storage_lifecycle::StorageLifecycle::Transient,
    )?;
    fs::create_dir_all(&store_dir).await?;
    let file_path = store_dir.join(format!("{}.json", report.experiment_id));
    // v2.3.3: 使用统一原子写入 (含 fsync)
    crate::runtime_persistence::atomic_write_json(&file_path, report).await
}

pub(super) async fn load_chaos_report_from_disk(
    store_dir: &FsPath,
    experiment_id: &str,
) -> Result<ChaosExperimentReport, (StatusCode, String)> {
    if let Err(msg) = validate_experiment_id(experiment_id) {
        return Err(json_bad_request("invalid_experiment_id", msg));
    }
    let file_path = store_dir.join(format!("{}.json", experiment_id));
    let json = fs::read(&file_path).await.map_err(|_| {
        json_bad_request("not_found", format!("混沌实验 '{}' 不存在", experiment_id))
    })?;
    serde_json::from_slice(&json).map_err(|error| internal_error(anyhow::anyhow!("{}", error)))
}

fn validate_experiment_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("experiment_id 不能为空".to_string());
    }
    if id.len() > 128 {
        return Err("experiment_id 长度不能超过 128 字符".to_string());
    }
    if id.contains("..") || id.contains('/') || id.contains('\\') || id.contains('\0') {
        return Err("experiment_id 不能包含路径分隔符".to_string());
    }
    if !id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err("experiment_id 只能使用 ASCII 字母、数字、'_' 或 '-'".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_experiment_id_accepts_safe_ids() {
        assert!(validate_experiment_id("chaos-123_ok").is_ok());
    }

    #[test]
    fn validate_experiment_id_rejects_path_like_ids() {
        assert!(validate_experiment_id("../chaos").is_err());
        assert!(validate_experiment_id("chaos\\bad").is_err());
        assert!(validate_experiment_id("chaos/bad").is_err());
    }

    #[test]
    fn validate_experiment_id_rejects_empty_and_non_ascii_ids() {
        assert!(validate_experiment_id("").is_err());
        assert!(validate_experiment_id("混沌").is_err());
    }
}
