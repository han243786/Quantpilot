// ── 策略合并协调器 (v2.2.0) ──
// 从 RuntimeCoordinator 提取: merge engine + policy + records

use crate::merge::{MergeDecisionRecord, MergePolicy, StrategyMergeEngine};

#[derive(Clone)]
pub struct MergeCoordinator {
    pub engine: StrategyMergeEngine,
    pub policy: MergePolicy,
    pub records: Vec<MergeDecisionRecord>,
}

impl Default for MergeCoordinator {
    fn default() -> Self {
        Self {
            engine: StrategyMergeEngine::new(MergePolicy::WeightedMerge),
            policy: MergePolicy::WeightedMerge,
            records: Vec::new(),
        }
    }
}
