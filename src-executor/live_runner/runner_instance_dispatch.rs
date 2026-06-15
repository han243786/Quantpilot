use crate::executor_state::{ActiveStrategy, RuntimeKind, TriggerEvent};
use crate::kline_buffer::KlinePool;
use crate::ws_client::WsEvent;
use qrpc_core::Symbol;
use qrpc_runtime::V4RuntimeMemorySnapshot;
use tokio::sync::broadcast;

use super::{LiveRunner, RunnerStatus, V4ExecutorEvidenceEvent, V4Runner};

#[allow(clippy::large_enum_variant)]
pub enum RunnerInstance {
    V3(LiveRunner),
    V4(V4Runner),
}

impl RunnerInstance {
    pub(super) fn from_strategy(
        s: &ActiveStrategy,
        trigger_broadcast: broadcast::Sender<TriggerEvent>,
        v4_evidence_broadcast: broadcast::Sender<V4ExecutorEvidenceEvent>,
    ) -> anyhow::Result<Self> {
        match s.runtime_kind {
            RuntimeKind::V3 => Ok(Self::V3(LiveRunner::from_strategy(s, trigger_broadcast))),
            RuntimeKind::V4 => Ok(Self::V4(V4Runner::from_strategy(s, v4_evidence_broadcast)?)),
        }
    }

    pub(super) fn handle_ws_event(&mut self, event: WsEvent) {
        match self {
            Self::V3(runner) => runner.handle_ws_event(event),
            Self::V4(runner) => runner.handle_ws_event(event),
        }
    }

    pub(super) fn subscribed_symbols(&self) -> &[Symbol] {
        match self {
            Self::V3(runner) => &runner.subscribed_symbols,
            Self::V4(runner) => &runner.subscribed_symbols,
        }
    }

    pub fn kline_pool(&self) -> Option<&KlinePool> {
        match self {
            Self::V3(runner) => Some(&runner.kline_pool),
            Self::V4(runner) => Some(&runner.kline_pool),
        }
    }

    pub fn v4_memory_snapshot(&self, now_ms: u64) -> Option<V4RuntimeMemorySnapshot> {
        match self {
            Self::V3(_) => None,
            Self::V4(runner) => Some(runner.runtime.memory_snapshot(now_ms)),
        }
    }

    pub(super) fn set_stopped(&mut self) {
        match self {
            Self::V3(runner) => runner.status = RunnerStatus::Stopped,
            Self::V4(runner) => runner.status = RunnerStatus::Stopped,
        }
    }
}
