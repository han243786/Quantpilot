// ── 运行时可变状态 (v2.2.0) ──
// 从 RuntimeCoordinator 提取: portfolio + counts + timestamps

use qrpc_core::PortfolioState;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct RuntimeState {
    pub portfolio: PortfolioState,
    pub data_fetch_counts: BTreeMap<String, u32>,
    pub last_action_at_ms: BTreeMap<String, u64>,
    pub last_rebalance_at_ms: BTreeMap<String, u64>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            portfolio: PortfolioState::new(100_000.0, 0),
            data_fetch_counts: BTreeMap::new(),
            last_action_at_ms: BTreeMap::new(),
            last_rebalance_at_ms: BTreeMap::new(),
        }
    }
}
