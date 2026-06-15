/// v3.7.0/v4.8.0: 实时策略运行器 — PaperSimulated 本地撮合 / PaperActual provider 回执
#[cfg(test)]
use crate::executor_state::{ExecutionMode, RuntimeKind};
#[cfg(test)]
use qrpc_core::RuntimeEventType;
#[cfg(test)]
use std::collections::BTreeMap;
#[cfg(test)]
use tokio::sync::broadcast;

mod runner_instance_dispatch;
mod runner_pool_orchestration;
mod v3_live_runner;
mod v4_market_metadata_helpers;
mod v4_runner;
pub use runner_instance_dispatch::RunnerInstance;
pub use runner_pool_orchestration::RunnerPool;
pub use v3_live_runner::{LiveRunner, RunnerStatus};
use v4_market_metadata_helpers::{
    executor_v4_market_matrix, resolve_v4_runner_default_symbol, resolve_v4_runner_venue_id,
};
pub use v4_runner::{V4ExecutorEvidenceEvent, V4Runner};

#[cfg(test)]
mod test_harness;
