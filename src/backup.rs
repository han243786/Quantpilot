// ── 自动备份模块 (v2.1.0) ──
// 每日一次将 Permanent 级数据打包到 storage/backups/

use std::path::Path;
use std::time::Duration;

const BACKUP_ROOT: &str = "storage/backups";
const MAX_BACKUP_AGE_DAYS: u64 = 7;
const MAX_BACKUP_TOTAL_MB: u64 = 200;

/// Permanent 级数据源（需要备份的目录和文件）
const PERMANENT_SOURCES: &[(&str, bool)] = &[
    ("storage/graphs", true),       // true = 目录
    ("storage/audit", true),
    ("storage/.credentials", false), // false = 单个文件
    ("storage/.machine_key", false),
    ("storage/auth.db", false),
];

/// 执行一次备份 (由后台任务每天调用)
pub async fn backup_permanent_storage() {
    let now = chrono::Utc::now();
    let dir_name = now.format("%Y-%m-%d_%H%M").to_string();
    let backup_dir = Path::new(BACKUP_ROOT).join(&dir_name);

    if let Err(e) = tokio::fs::create_dir_all(&backup_dir).await {
        safe_eprintln!("[backup] 创建备份目录失败: {}", e);
        return;
    }

    let mut manifest_files = Vec::new();
    let mut total_bytes: u64 = 0;

    for (source_path, is_dir) in PERMANENT_SOURCES {
        let source = Path::new(source_path);
        if !source.try_exists().unwrap_or(false) {
            continue;
        }

        if *is_dir {
            match copy_dir_recursive(source, &backup_dir).await {
                Ok((files, bytes)) => {
                    manifest_files.extend(files);
                    total_bytes += bytes;
                }
                Err(e) => {
                    safe_eprintln!("[backup] 复制目录 {} 失败: {}", source_path, e);
                }
            }
        } else {
            let file_name = source.file_name().unwrap_or_default();
            let dest = backup_dir.join(file_name);
            match tokio::fs::copy(source, &dest).await {
                Ok(bytes) => {
                    manifest_files.push(BackupFileEntry {
                        path: format!("{}/{}", dir_name, file_name.to_string_lossy()),
                        size_bytes: bytes,
                    });
                    total_bytes += bytes;
                }
                Err(e) => {
                    safe_eprintln!("[backup] 复制文件 {} 失败: {}", source_path, e);
                }
            }
        }
    }

    // 写入备份清单
    let manifest = BackupManifest {
        backup_id: dir_name.clone(),
        created_at: now.to_rfc3339(),
        files: manifest_files,
        total_size_bytes: total_bytes,
    };

    match serde_json::to_string_pretty(&manifest) {
        Ok(json) => {
            let manifest_path = backup_dir.join("backup_manifest.json");
            let tmp = manifest_path.with_extension("tmp");
            if let Err(e) = tokio::fs::write(&tmp, &json).await {
                safe_eprintln!("[backup] 写入清单失败: {}", e);
            } else {
                let _ = tokio::fs::rename(&tmp, &manifest_path).await;
            }
        }
        Err(e) => {
            safe_eprintln!("[backup] 序列化清单失败: {}", e);
        }
    }

    safe_eprintln!(
        "[backup] 备份完成: {} ({} 文件, {} MB)",
        dir_name,
        manifest.files.len(),
        total_bytes / (1024 * 1024)
    );

    // 清理过期备份
    cleanup_old_backups().await;
}

async fn copy_dir_recursive(
    source: &Path,
    backup_parent: &Path,
) -> std::io::Result<(Vec<BackupFileEntry>, u64)> {
    let dir_name = source.file_name().unwrap_or_default();
    let dest_dir = backup_parent.join(dir_name);
    tokio::fs::create_dir_all(&dest_dir).await?;

    let mut files = Vec::new();
    let mut total = 0u64;
    let mut entries = tokio::fs::read_dir(source).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            let (sub_files, sub_bytes) = Box::pin(copy_dir_recursive(&path, &dest_dir)).await?;
            files.extend(sub_files);
            total += sub_bytes;
        } else if let Some(name) = path.file_name() {
            let dest = dest_dir.join(name);
            let bytes = tokio::fs::copy(&path, &dest).await?;
            files.push(BackupFileEntry {
                path: format!(
                    "{}/{}",
                    dir_name.to_string_lossy(),
                    name.to_string_lossy()
                ),
                size_bytes: bytes,
            });
            total += bytes;
        }
    }

    Ok((files, total))
}

async fn cleanup_old_backups() {
    let backup_root = Path::new(BACKUP_ROOT);
    let Ok(mut entries) = tokio::fs::read_dir(backup_root).await else {
        return;
    };

    let cutoff = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::days(MAX_BACKUP_AGE_DAYS as i64));

    let Some(cutoff) = cutoff else {
        return;
    };

    let mut total_backup_bytes: u64 = 0;
    let mut to_remove = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // 检查备份年龄
        if let Ok(metadata) = tokio::fs::metadata(&path).await {
            if let Ok(modified) = metadata.modified() {
                let age_secs = modified.elapsed().unwrap_or(Duration::ZERO);
                if age_secs.as_secs() > MAX_BACKUP_AGE_DAYS * 24 * 3600 {
                    to_remove.push(path.clone());
                    continue;
                }
            }
            total_backup_bytes += metadata.len();
        }

        // 从目录名推断时间
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if let Ok(date_part) = chrono::NaiveDate::parse_from_str(
                &name[..name.len().min(10)],
                "%Y-%m-%d",
            ) {
                let age_days = (chrono::Utc::now().naive_utc().date() - date_part)
                    .num_days();
                if age_days > MAX_BACKUP_AGE_DAYS as i64 {
                    to_remove.push(path.clone());
                }
            }
        }
    }

    // 检查总大小
    let max_bytes = MAX_BACKUP_TOTAL_MB * 1024 * 1024;
    if total_backup_bytes > max_bytes {
        safe_eprintln!(
            "[backup] 备份总大小 {} MB 超出 {} MB 限制，将删除最旧备份",
            total_backup_bytes / (1024 * 1024),
            MAX_BACKUP_TOTAL_MB
        );
    }

    for path in to_remove {
        if let Err(e) = tokio::fs::remove_dir_all(&path).await {
            safe_eprintln!("[backup] 删除旧备份 {} 失败: {}", path.display(), e);
        } else {
            safe_eprintln!("[backup] 已删除过期备份: {}", path.display());
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BackupManifest {
    backup_id: String,
    created_at: String,
    files: Vec<BackupFileEntry>,
    total_size_bytes: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BackupFileEntry {
    path: String,
    size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tokio::fs;

    #[tokio::test]
    async fn backup_creates_directory_and_manifest() {
        let backup_dir = Path::new(BACKUP_ROOT);
        // 清理之前测试残留
        let _ = fs::remove_dir_all(backup_dir).await;

        backup_permanent_storage().await;

        // 验证备份目录被创建且包含文件
        let mut entries = fs::read_dir(backup_dir).await.unwrap();
        let mut found_backup = false;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                let manifest = path.join("backup_manifest.json");
                if fs::try_exists(&manifest).await.unwrap_or(false) {
                    found_backup = true;
                    let content = fs::read_to_string(&manifest).await.unwrap();
                    assert!(content.contains("backup_id"));
                    assert!(content.contains("files"));
                }
            }
        }
        // 清理测试产物
        let _ = fs::remove_dir_all(backup_dir).await;
        assert!(found_backup, "备份应包含有效的 manifest.json");
    }

    #[test]
    fn manifest_serialization_roundtrip() {
        let entry = BackupFileEntry {
            path: "graphs/test.json".to_string(),
            size_bytes: 1024,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: BackupFileEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.path, "graphs/test.json");
        assert_eq!(parsed.size_bytes, 1024);
    }

    #[test]
    fn manifest_rejects_unknown_fields() {
        let json = r#"{"path":"test.json","size_bytes":100,"unknown_field":42}"#;
        let result = serde_json::from_str::<BackupFileEntry>(json);
        assert!(result.is_err());
    }
}
