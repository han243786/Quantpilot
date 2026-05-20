/// v3.7.0: 执行端审计日志
/// 追加写入, JSON Lines 格式, 不可删除

use anyhow::Result;
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
    pub fn new(storage_dir: &std::path::Path) -> Result<Self> {
        let path = storage_dir.join(AUDIT_LOG_FILE);
        Ok(Self { path, writer: Mutex::new(()) })
    }

    /// 追加审计条目 (JSON Lines) — v3.0.1 D-2: Mutex守卫生命周期修复
    pub fn append(&self, entry: &AuditEntry) {
        let _guard = self.writer.lock().unwrap(); // 守卫在此作用域内保持
        let line = serde_json::to_string(entry).unwrap_or_default();
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true).append(true).open(&self.path)
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
        let Ok(content) = std::fs::read_to_string(&self.path) else {
            eprintln!("[audit] 读取日志文件失败: {}", self.path.display());
            return Vec::new();
        };
        content
            .lines()
            .filter_map(|line| serde_json::from_str::<AuditEntry>(line).ok())
            .rev()
            .take(count)
            .collect()
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
