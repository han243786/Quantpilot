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

const GLOBAL_MAX_BYTES: u64 = 500 * 1024 * 1024;
const WARN_AT_BYTES: u64 = 400 * 1024 * 1024;

fn directory_lifecycle(dir_name: &str) -> StorageLifecycle {
    match dir_name {
        "graphs" | "audit" => StorageLifecycle::Permanent,
        "backtests" | "runs" | "experiments" | "approvals" | "reports" | "mutations" => StorageLifecycle::Temporary,
        _ => StorageLifecycle::Transient,
    }
}

fn dir_size_bytes(path: &Path) -> u64 {
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

pub fn startup_storage_cleanup(storage_root: &Path) {
    let entries = match std::fs::read_dir(storage_root) {
        Ok(e) => e,
        Err(e) => {
            crate::safe_eprintln!("[storage] 无法读取存储目录: {}", e);
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
        if let Some(ttl) = lifecycle.ttl() {
            if let Ok(dir_entries) = std::fs::read_dir(&path) {
                for file_entry in dir_entries.flatten() {
                    let file_path = file_entry.path();
                    let meta = match file_entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };
                    let aged = file_age(&meta);
                    let safety_margin = Duration::from_secs(10 * 60); // 10分钟
                    if aged.map_or(false, |age| age > ttl + safety_margin) {
                        // 仅在超过 TTL + 10分钟时才删除（确保不在活跃使用中）
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
        eprintln!(
            "[storage] 告警: 总大小 {} MB 超过 80% 阈值 ({} MB)",
            total_size / (1024 * 1024),
            WARN_AT_BYTES / (1024 * 1024)
        );
    }
    if cleaned_count > 0 {
        eprintln!(
            "[storage] 启动清理: 删除 {} 项过期文件/目录, 释放 {} KB",
            cleaned_count,
            cleaned_bytes / 1024
        );
    }
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
                eprintln!("[storage] 已清理非标准构建目录: {}", name);
            }
        }
    }
}
