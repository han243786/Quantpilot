// ── 配置追踪器 (v2.2.0) ──
// 从 RuntimeCoordinator 提取: 部署修订 + 配置代际 + 历史

use crate::ConfigGenerationEntry;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct ConfigTracker {
    pub applied_deployment_revisions: Vec<String>,
    pub config_generation: Arc<AtomicU64>,
    // 使用 std::sync::Mutex: 锁持有时间极短 (仅 Vec push), 调用链为同步上下文
    pub config_generation_history: Arc<std::sync::Mutex<Vec<ConfigGenerationEntry>>>,
}

impl Default for ConfigTracker {
    fn default() -> Self {
        Self {
            applied_deployment_revisions: Vec::new(),
            config_generation: Arc::new(AtomicU64::new(1)),
            config_generation_history: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}

impl ConfigTracker {
    pub fn bump_generation(&self) -> u64 {
        self.config_generation.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn current_generation(&self) -> u64 {
        self.config_generation.load(Ordering::Relaxed)
    }
}
