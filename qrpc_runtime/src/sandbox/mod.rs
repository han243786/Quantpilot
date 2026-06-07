mod fast_backtest_sandbox;
mod mode_surface;
mod realtime_sandbox;
pub mod replay;
pub mod timeline;

pub use self::replay::{
    build_v4_deterministic_replay_bars, sort_v4_replay_ticks_deterministically,
};
use crate::RuntimeCoordinator;
use anyhow::{anyhow, Result};
use qrpc_core::{
    CompiledRuntimeProtocol, CoreStrategyIr, ExecutionPlan, FillResult, HandoffSnapshot,
    NormalizedMarketData, RuntimeEvent, SessionOutput,
};

pub use self::fast_backtest_sandbox::FastBacktestSandbox;
pub use self::realtime_sandbox::RealTimeSandbox;

pub use self::mode_surface::{
    runtime_support_boundary, DeterministicClockMode, DeterministicEventOrdering,
    DeterministicParallelismPolicy, DeterministicTestMode, RuntimeSupportBoundary, Sandbox,
    SandboxMode, SandboxSnapshot, SUPPORTED_RUNTIME_EXECUTION_MODULE_KEYS,
    SUPPORTED_RUNTIME_MODE_KEYS,
};

// Parent-owned shared sandbox helpers.
fn snapshot_from(
    coordinator: &RuntimeCoordinator,
    mode: SandboxMode,
    is_running: bool,
    test_mode: &DeterministicTestMode,
    now_ms: u64,
) -> SandboxSnapshot {
    SandboxSnapshot {
        mode,
        is_running,
        captured_at_ms: now_ms,
        deterministic_test_mode: test_mode.clone(),
        portfolio: coordinator.portfolio_state().clone(),
        data_fetch_counts: coordinator.data_fetch_counts().clone(),
        last_action_at_ms: coordinator.last_action_at_ms().clone(),
    }
}

fn ensure_running(running: bool, label: &str) -> Result<()> {
    if running {
        Ok(())
    } else {
        Err(anyhow!("{label} 未在运行"))
    }
}

fn trace_id(prefix: &str, now_ms: u64) -> String {
    format!("trace-{prefix}-{now_ms}")
}

// ── 测试 ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod test_harness;
