/// v3.7.0: 执行端审计日志
/// 追加写入, JSON Lines 格式, 不可删除
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Mutex;

const AUDIT_LOG_FILE: &str = ".executor-audit.log";

/// 审计日志记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub operation: String,
    pub actor: String,
    pub strategy_id: Option<String>,
    pub details: Value,
}

/// 审计日志写入器
pub struct AuditLog {
    path: PathBuf,
    writer: Mutex<()>, // 串行化写入
}

impl AuditLog {
    pub fn new(storage_dir: &std::path::Path) -> Self {
        let path = storage_dir.join(AUDIT_LOG_FILE);
        Self {
            path,
            writer: Mutex::new(()),
        }
    }

    /// 追加审计条目 (JSON Lines) — v3.0.1 D-2: Mutex守卫生命周期修复
    pub fn append(&self, entry: &AuditEntry) {
        let _guard = self.writer.lock().unwrap(); // 守卫在此作用域内保持
        let line = serde_json::to_string(entry).unwrap_or_default();
        if let Some(parent) = self.path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("[audit] 创建目录失败: {}", e);
                return;
            }
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            use std::io::Write;
            if let Err(e) = writeln!(file, "{}", line) {
                eprintln!("[audit] 写入失败: {}", e);
            }
            // v3.0.1 D-3: fsync确保日志落盘
            if let Err(e) = file.sync_all() {
                eprintln!("[audit] 同步失败: {}", e);
            }
        } else {
            eprintln!("[audit] 打开日志文件失败: {}", self.path.display());
        }
    }

    /// 读取最近 N 条审计记录
    pub fn recent(&self, count: usize) -> Vec<AuditEntry> {
        let content = match std::fs::read_to_string(&self.path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
            Err(error) => {
                eprintln!(
                    "[audit] 读取日志文件失败: {} ({})",
                    self.path.display(),
                    error
                );
                return Vec::new();
            }
        };
        content
            .lines()
            .filter_map(|line| serde_json::from_str::<AuditEntry>(line).ok())
            .rev()
            .take(count)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_recent_roundtrip_json_lines() {
        let dir =
            std::env::temp_dir().join(format!("quantpilot-executor-audit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let log = AuditLog::new(&dir);

        log.append(&AuditEntry {
            timestamp: "2026-05-24T00:00:00Z".to_string(),
            operation: "start_strategy".to_string(),
            actor: "test".to_string(),
            strategy_id: Some("s1".to_string()),
            details: serde_json::json!({"status": "running"}),
        });

        let entries = log.recent(1);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].operation, "start_strategy");
        assert_eq!(entries[0].strategy_id.as_deref(), Some("s1"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_audit_log_reads_as_empty() {
        let dir = std::env::temp_dir().join(format!(
            "quantpilot-executor-audit-missing-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let log = AuditLog::new(&dir);

        assert!(log.recent(10).is_empty());
    }
}

/// 便捷宏: 记录审计日志
#[macro_export]
macro_rules! audit {
    ($log:expr, $op:literal, $actor:expr $(, $key:ident: $val:expr)*) => {
        $log.append(&$crate::audit_log::AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: $op.to_string(),
            actor: $actor.to_string(),
            strategy_id: None,
            details: serde_json::json!({ $(stringify!($key): $val),* }),
        });
    };
}
