use crate::executor_state::{ActiveStrategy, ExecutionMode, TriggerEvent};
use crate::kline_buffer::KlinePool;
use crate::ws_client::WsEvent;
use qrpc_core::{RuntimeEvent, RuntimeEventType, Symbol};
use qrpc_runtime::RuntimeCoordinator;
use tokio::sync::broadcast;

pub struct LiveRunner {
    pub strategy_id: String,
    pub coordinator: RuntimeCoordinator,
    pub subscribed_symbols: Vec<Symbol>,
    pub kline_pool: KlinePool,
    pub trigger_broadcast: broadcast::Sender<TriggerEvent>,
    pub status: RunnerStatus,
    pub execution_mode: ExecutionMode,
    /// v3.0.0 A-2: 速率限制 — 最后周期执行时间戳
    pub last_cycle_at_ms: u64,
    /// v3.0.0 A-2: 每日订单计数器
    pub daily_order_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerStatus {
    Idle,
    Running,
    Stopped,
    Faulted(String),
}

impl LiveRunner {
    const KLINE_POOL_CAPACITY: usize = 1000;
    const MAX_DAILY_ORDER_COUNT: u32 = 5_000;

    pub fn from_strategy(s: &ActiveStrategy, bc: broadcast::Sender<TriggerEvent>) -> Self {
        Self {
            coordinator: RuntimeCoordinator::from_core_ir(s.core_ir.clone()),
            strategy_id: s.strategy_id.clone(),
            subscribed_symbols: s.subscribed_symbols.clone(),
            kline_pool: KlinePool::new(Self::KLINE_POOL_CAPACITY),
            trigger_broadcast: bc,
            // v3.6.x: Paper模式启动即Running, 不依赖Connected事件
            status: if s.execution_mode.starts_without_provider_connection() {
                RunnerStatus::Running
            } else {
                RunnerStatus::Idle
            },
            execution_mode: s.execution_mode,
            last_cycle_at_ms: 0,
            daily_order_count: 0,
        }
    }

    pub fn handle_ws_event(&mut self, event: WsEvent) {
        match event {
            WsEvent::Ticker {
                symbol,
                price,
                ts_ms,
            } => {
                if !self.is_subscribed_to(&symbol) {
                    return;
                }
                self.kline_pool.update_from_ticker(&symbol, price, ts_ms);
                self.run_fast_cycle(ts_ms);
            }
            WsEvent::Kline { symbol, bar } => {
                if !self.is_subscribed_to(&symbol) {
                    return;
                }
                let close_ms = bar.close_time_ms;
                self.kline_pool.update_kline(&symbol, bar);
                self.run_slow_cycle(close_ms);
            }
            WsEvent::Connected { exchange } => {
                eprintln!(
                    "[runner:{}] {} websocket connected",
                    self.strategy_id, exchange
                );
                if self.status == RunnerStatus::Idle {
                    self.status = RunnerStatus::Running;
                }
            }
            #[cfg(test)]
            WsEvent::Disconnected { exchange, reason } => {
                eprintln!(
                    "[runner:{}] {} websocket disconnected: {}",
                    self.strategy_id, exchange, reason
                );
                self.status = RunnerStatus::Faulted(reason.clone());
            }
        }
    }

    const MIN_CYCLE_INTERVAL_MS: u64 = 200;

    fn run_fast_cycle(&mut self, now_ms: u64) {
        if self.status != RunnerStatus::Running {
            return;
        }
        // v3.0.0 A-2: 速率限制 (≥200ms 间隔)
        if now_ms.saturating_sub(self.last_cycle_at_ms) < Self::MIN_CYCLE_INTERVAL_MS {
            return;
        }
        self.last_cycle_at_ms = now_ms;
        if let Err(e) = self.coordinator.run_fast_cycle(now_ms) {
            eprintln!("[runner:{}] fast_cycle error: {:?}", self.strategy_id, e);
        }
    }

    fn run_slow_cycle(&mut self, now_ms: u64) {
        if self.status != RunnerStatus::Running {
            return;
        }
        if self.execution_mode.provider_order_submission_attached()
            && self.daily_order_count >= Self::MAX_DAILY_ORDER_COUNT
        {
            self.status = RunnerStatus::Faulted("daily_order_count_exceeded".to_string());
            return;
        }
        match self.coordinator.run_slow_cycle(now_ms) {
            Ok(cycle) => {
                for event in &cycle.runtime_events {
                    if matches!(event.event_type, RuntimeEventType::ExecutionFilled) {
                        self.daily_order_count = self.daily_order_count.saturating_add(1);
                    }
                    if let Some(t) = Self::detect_trigger(&self.strategy_id, event) {
                        let _ = self.trigger_broadcast.send(t);
                    }
                }
            }
            Err(e) => {
                eprintln!("[runner:{}] slow_cycle error: {:?}", self.strategy_id, e);
            }
        }
    }

    pub(super) fn detect_trigger(sid: &str, event: &RuntimeEvent) -> Option<TriggerEvent> {
        let (tt, strength) = match event.event_type {
            RuntimeEventType::IntentTriggered => (
                "intent_triggered",
                event
                    .payload
                    .get("strength")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5),
            ),
            RuntimeEventType::AgentDecisionProduced => (
                "agent_decided",
                event
                    .payload
                    .get("net_strength")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5),
            ),
            RuntimeEventType::ExecutionFilled => ("order_filled", 1.0),
            _ => return None,
        };
        Some(TriggerEvent {
            strategy_id: sid.to_string(),
            trigger_type: tt.into(),
            node_id: event
                .payload
                .get("indicator_id")
                .or_else(|| event.payload.get("agent_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .into(),
            strength,
            occurred_at_ms: event.ts_ms,
        })
    }

    fn is_subscribed_to(&self, symbol: &str) -> bool {
        self.subscribed_symbols.is_empty()
            || self
                .subscribed_symbols
                .iter()
                .any(|item| item.as_str().eq_ignore_ascii_case(symbol))
    }
}
