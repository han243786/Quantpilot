use std::path::Path;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageLifecycle {
    Permanent,
    Temporary,
    Transient,
}

impl StorageLifecycle {
    pub fn ttl(&self) -> Option<Duration> {
        let dev_mode = std::env::var("QUANTPILOT_DEV").unwrap_or_default() == "true";
        match self {
            Self::Permanent => None,
            Self::Temporary => Some(Duration::from_secs(if dev_mode { 24 * 3600 } else { 7 * 24 * 3600 })),
            Self::Transient => Some(Duration::from_secs(if dev_mode { 10 * 60 } else { 3600 })),
        }
    }
}

const GLOBAL_MAX_BYTES: u64 = 500 * 1024 * 1024;       // 500 MB
const WARN_AT_BYTES: u64 = 400 * 1024 * 1024;           // 80%
const FORCE_CLEAN_AT_BYTES: u64 = 450 * 1024 * 1024;    // 90%
const REJECT_AT_BYTES: u64 = 475 * 1024 * 1024;         // 95%
const TEMPORARY_DIR_MAX_BYTES: u64 = 200 * 1024 * 1024; // 200 MB
const TRANSIENT_DIR_MAX_BYTES: u64 = 50 * 1024 * 1024;  // 50 MB

fn directory_lifecycle(dir_name: &str) -> StorageLifecycle {
    match dir_name {
        "graphs" | "audit" => StorageLifecycle::Permanent,
        "backtests" | "runs" | "experiments" | "approvals" | "reports" | "mutations" => StorageLifecycle::Temporary,
        "ai-proposals" | "alerts" | "snapshots" | "sandbox-reports" | "chaos" => StorageLifecycle::Transient,
        _ => StorageLifecycle::Transient,
    }
}

pub(crate) fn dir_size_bytes(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                } else if meta.is_dir() {
                    total += dir_size_bytes(&entry.path());
                }
            }
        }
    }
    total
}

fn file_age(meta: &std::fs::Metadata) -> Option<Duration> {
    SystemTime::now().duration_since(meta.modified().ok()?).ok()
}

fn storage_total_size(storage_root: &Path) -> u64 {
    dir_size_bytes(storage_root)
}

/// 检查全局存储配额。返回 Ok(()) 或中文错误消息。
pub fn check_storage_quota(storage_root: &Path) -> Result<(), String> {
    let total = storage_total_size(storage_root);
    if total > REJECT_AT_BYTES {
        return Err(format!(
            "存储空间已满: 当前 {} MB, 上限 {} MB。请清理过期数据后重试",
            total / (1024 * 1024),
            GLOBAL_MAX_BYTES / (1024 * 1024)
        ));
    }
    if total > FORCE_CLEAN_AT_BYTES {
        safe_eprintln!(
            "[storage] 严重告警: 总大小 {} MB 超过 90% 阈值 ({} MB), 需要立即清理",
            total / (1024 * 1024),
            FORCE_CLEAN_AT_BYTES / (1024 * 1024)
        );
    }
    Ok(())
}

/// 检查单个目录的配额限制。
pub fn check_directory_quota(dir_path: &Path, lifecycle: StorageLifecycle) -> Result<(), String> {
    let max_bytes = match lifecycle {
        StorageLifecycle::Permanent => return Ok(()), // 无上限
        StorageLifecycle::Temporary => TEMPORARY_DIR_MAX_BYTES,
        StorageLifecycle::Transient => TRANSIENT_DIR_MAX_BYTES,
    };
    let size = dir_size_bytes(dir_path);
    if size > max_bytes {
        return Err(format!(
            "目录 {} 已满: 当前 {} MB, 上限 {} MB",
            dir_path.display(),
            size / (1024 * 1024),
            max_bytes / (1024 * 1024)
        ));
    }
    Ok(())
}

pub fn startup_storage_cleanup(storage_root: &Path) {
    let dev_mode = std::env::var("QUANTPILOT_DEV").unwrap_or_default() == "true";
    let entries = match std::fs::read_dir(storage_root) {
        Ok(e) => e,
        Err(e) => {
            safe_eprintln!("[storage] 无法读取存储目录: {}", e);
            return;
        }
    };
    let mut total_size: u64 = 0;
    let mut cleaned_count = 0u64;
    let mut cleaned_bytes = 0u64;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
        let lifecycle = directory_lifecycle(&dir_name);
        let dir_size = dir_size_bytes(&path);
        total_size += dir_size;

        // DEV 模式: 强制清理所有瞬态数据
        if dev_mode && matches!(lifecycle, StorageLifecycle::Transient) {
            for file_entry in std::fs::read_dir(&path).into_iter().flatten().flatten() {
                let file_path = file_entry.path();
                if file_path.is_dir() {
                    cleaned_bytes += dir_size_bytes(&file_path);
                    let _ = std::fs::remove_dir_all(&file_path);
                } else if let Ok(meta) = file_entry.metadata() {
                    cleaned_bytes += meta.len();
                    let _ = std::fs::remove_file(&file_path);
                }
                cleaned_count += 1;
            }
            continue;
        }

        if let Some(ttl) = lifecycle.ttl() {
            if let Ok(dir_entries) = std::fs::read_dir(&path) {
                for file_entry in dir_entries.flatten() {
                    let file_path = file_entry.path();
                    let meta = match file_entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };
                    let aged = file_age(&meta);
                    // 90% 阈值时激进清理: 不添加安全余量
                    let safety_margin = if total_size > FORCE_CLEAN_AT_BYTES {
                        Duration::from_secs(0)
                    } else {
                        Duration::from_secs(10 * 60) // 10分钟
                    };
                    if aged.map_or(false, |age| age > ttl + safety_margin) {
                        if file_path.is_dir() {
                            cleaned_bytes += dir_size_bytes(&file_path);
                            let _ = std::fs::remove_dir_all(&file_path);
                        } else {
                            cleaned_bytes += meta.len();
                            let _ = std::fs::remove_file(&file_path);
                        }
                        cleaned_count += 1;
                    }
                }
            }
        }
    }
    if total_size > WARN_AT_BYTES {
        safe_eprintln!(
            "[storage] 告警: 总大小 {} MB 超过 80% 阈值 ({} MB)",
            total_size / (1024 * 1024),
            WARN_AT_BYTES / (1024 * 1024)
        );
    }
    if total_size > FORCE_CLEAN_AT_BYTES {
        safe_eprintln!(
            "[storage] 严重告警: 总大小 {} MB 超过 90% 阈值, 已强制清理过期暂时/瞬间数据",
            total_size / (1024 * 1024)
        );
    }
    if cleaned_count > 0 {
        safe_eprintln!(
            "[storage] 启动清理: 删除 {} 项过期文件/目录, 释放 {} KB",
            cleaned_count,
            cleaned_bytes / 1024
        );
    }
}

/// 写入前配额检查，返回 io::Error 以兼容异步写入路径
pub fn ensure_storage_quota(storage_root: &Path, dir_name: &str, lifecycle: StorageLifecycle) -> std::io::Result<()> {
    let total = storage_total_size(storage_root);
    if total > REJECT_AT_BYTES {
        return Err(std::io::Error::other(format!(
            "存储空间已满: 当前 {} MB, 上限 {} MB。请清理过期数据后重试",
            total / (1024 * 1024),
            GLOBAL_MAX_BYTES / (1024 * 1024)
        )));
    }
    if total > WARN_AT_BYTES {
        safe_eprintln!(
            "[storage] 告警: 总大小 {} MB 超过 80% 阈值",
            total / (1024 * 1024)
        );
    }
    let dir_path = storage_root.join(dir_name);
    let max_bytes = match lifecycle {
        StorageLifecycle::Permanent => return Ok(()),
        StorageLifecycle::Temporary => TEMPORARY_DIR_MAX_BYTES,
        StorageLifecycle::Transient => TRANSIENT_DIR_MAX_BYTES,
    };
    if dir_path.exists() {
        let dir_size = dir_size_bytes(&dir_path);
        if dir_size > max_bytes {
            return Err(std::io::Error::other(format!(
                "目录 {} 已满: 当前 {} MB, 上限 {} MB",
                dir_path.display(),
                dir_size / (1024 * 1024),
                max_bytes / (1024 * 1024)
            )));
        }
    }
    Ok(())
}

/// 统一持久化写入函数，声明生命周期并检查配额 (§7.4)
pub fn persist_with_ttl(
    storage_root: &Path,
    relative_path: &Path,
    data: &[u8],
    lifecycle: StorageLifecycle,
) -> Result<(), String> {
    // 1. 检查全局配额
    check_storage_quota(storage_root)?;

    // 2. 检查每目录配额
    let dir_name = relative_path
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("unknown");
    let dir_path = storage_root.join(dir_name);
    check_directory_quota(&dir_path, lifecycle)?;

    // 3. 确保目录存在
    let full_path = storage_root.join(relative_path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // 4. 写入数据
    std::fs::write(&full_path, data).map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(())
}

pub fn cleanup_build_artifacts() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target_dir = project_root.join("target");
    if !target_dir.exists() {
        return;
    }

    // Clean stale test artifact directories (older than 24h)
    let test_artifacts = target_dir.join("test-artifacts");
    if test_artifacts.exists() {
        if let Ok(entries) = std::fs::read_dir(&test_artifacts) {
            let ttl = Duration::from_secs(24 * 3600);
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if file_age(&meta).map_or(false, |age| age > ttl) {
                        let _ = std::fs::remove_dir_all(&entry.path());
                    }
                }
            }
        }
    }

    // Remove non-standard target-test-* directories
    if let Ok(entries) = std::fs::read_dir(project_root) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("target-test-") && entry.path().is_dir() {
                let _ = std::fs::remove_dir_all(&entry.path());
                safe_eprintln!("[storage] 已清理非标准构建目录: {}", name);
            }
        }
    }
}
