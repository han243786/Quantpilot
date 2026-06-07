use anyhow::Result;
use qrpc_core::{
    ExecutionPlan, FillResult, HandoffSnapshot, NormalizedMarketData, PortfolioState, RuntimeEvent,
    SessionOutput,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SUPPORTED_RUNTIME_MODE_KEYS: [&str; 1] = ["paper"];
pub const SUPPORTED_RUNTIME_EXECUTION_MODULE_KEYS: [&str; 2] =
    ["builtin.execution.paper", "live.okx"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSupportBoundary {
    pub runtime_modes: &'static [&'static str],
    pub execution_module_keys: &'static [&'static str],
}

pub fn runtime_support_boundary() -> RuntimeSupportBoundary {
    RuntimeSupportBoundary {
        runtime_modes: &SUPPORTED_RUNTIME_MODE_KEYS,
        execution_module_keys: &SUPPORTED_RUNTIME_EXECUTION_MODULE_KEYS,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeterministicClockMode {
    WallClock,
    SimulatedClock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeterministicEventOrdering {
    Stable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeterministicParallelismPolicy {
    RuntimeDefault,
    SingleThreaded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicTestMode {
    pub enabled: bool,
    pub seed: Option<u64>,
    pub clock_mode: DeterministicClockMode,
    pub start_time_ms: Option<u64>,
    pub event_ordering: DeterministicEventOrdering,
    pub parallelism_policy: DeterministicParallelismPolicy,
}

impl Default for DeterministicTestMode {
    fn default() -> Self {
        Self {
            enabled: false,
            seed: None,
            clock_mode: DeterministicClockMode::WallClock,
            start_time_ms: None,
            event_ordering: DeterministicEventOrdering::Stable,
            parallelism_policy: DeterministicParallelismPolicy::RuntimeDefault,
        }
    }
}

impl DeterministicTestMode {
    pub fn enabled_with_seed(seed: u64) -> Self {
        Self {
            enabled: true,
            seed: Some(seed),
            ..Self::default()
        }
    }

    pub fn replay_defaults(start_time_ms: u64, seed: u64) -> Self {
        Self {
            enabled: true,
            seed: Some(seed),
            clock_mode: DeterministicClockMode::SimulatedClock,
            start_time_ms: Some(start_time_ms),
            event_ordering: DeterministicEventOrdering::Stable,
            parallelism_policy: DeterministicParallelismPolicy::SingleThreaded,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxMode {
    RealTimeSimulation,
    FastBacktest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSnapshot {
    pub mode: SandboxMode,
    pub is_running: bool,
    pub captured_at_ms: u64,
    pub deterministic_test_mode: DeterministicTestMode,
    pub portfolio: PortfolioState,
    pub data_fetch_counts: BTreeMap<String, u32>,
    pub last_action_at_ms: BTreeMap<String, u64>,
}

pub trait Sandbox {
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
    fn mode(&self) -> SandboxMode;
    fn run_session(&mut self, slow_now_ms: u64, fast_now_ms: u64) -> Result<SessionOutput>;
    fn submit_execution_plan(
        &mut self,
        plan: ExecutionPlan,
        normalized_data: Vec<NormalizedMarketData>,
        now_ms: u64,
    ) -> Result<FillResult>;
    fn on_market_data(
        &mut self,
        normalized_data: Vec<NormalizedMarketData>,
        now_ms: u64,
    ) -> Result<Vec<RuntimeEvent>>;
    fn snapshot(&self, now_ms: u64) -> SandboxSnapshot;
    fn swap_module_config(&mut self, module_key: &str, config: serde_json::Value)
        -> Result<String>;

    fn handoff(&mut self, _snapshot: &HandoffSnapshot) -> Result<()> {
        Err(anyhow::anyhow!("当前 Sandbox 不支持热接管"))
    }

    /// v2.1.0: 从 SandboxSnapshot 恢复运行时状态 (checkpoint/restore)
    fn restore(&mut self, snapshot: &SandboxSnapshot) -> Result<()> {
        // 验证 snapshot 兼容性
        if snapshot.mode != self.mode() {
            anyhow::bail!(
                "快照模式 ({:?}) 与当前沙箱模式 ({:?}) 不匹配",
                snapshot.mode,
                self.mode()
            );
        }
        Ok(()) // 默认不恢复, 由各实现覆盖
    }
}
