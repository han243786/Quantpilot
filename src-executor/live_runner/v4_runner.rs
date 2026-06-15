use crate::executor_state::ActiveStrategy;
use crate::kline_buffer::KlinePool;
use crate::ws_client::WsEvent;
use qrpc_core::Symbol;
use qrpc_core_ir::v4::MachineEventSourceKind;
use qrpc_runtime::{
    V4PaperSimulatedRunOutput, V4PaperSimulatedRuntime, V4RuntimeMemorySnapshot,
    V4SimulatedExecutionConfig,
};
use tokio::sync::broadcast;

use super::{
    executor_v4_market_matrix, resolve_v4_runner_default_symbol, resolve_v4_runner_venue_id,
    RunnerStatus,
};

pub struct V4Runner {
    pub strategy_id: String,
    pub runtime: V4PaperSimulatedRuntime,
    pub venue_id: String,
    pub subscribed_symbols: Vec<Symbol>,
    pub kline_pool: KlinePool,
    pub status: RunnerStatus,
    pub market_event_type: String,
    pub evidence_broadcast: broadcast::Sender<V4ExecutorEvidenceEvent>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct V4ExecutorEvidenceEvent {
    pub strategy_id: String,
    pub event_type: String,
    pub memory_snapshot: V4RuntimeMemorySnapshot,
    pub runtime_events: Vec<qrpc_runtime::V4RuntimeEventEnvelope>,
}

impl V4Runner {
    const KLINE_POOL_CAPACITY: usize = 1000;
    pub(super) const DEFAULT_REALTIME_PAPER_VENUE_ID: &'static str = "paper-simulated";

    pub fn from_strategy(
        s: &ActiveStrategy,
        evidence_broadcast: broadcast::Sender<V4ExecutorEvidenceEvent>,
    ) -> anyhow::Result<Self> {
        let graph = s
            .v4_graph
            .clone()
            .ok_or_else(|| anyhow::anyhow!("v4 runner requires v4_graph"))?;
        let market_event_type = graph
            .event_catalog
            .as_ref()
            .and_then(|catalog| {
                catalog
                    .events
                    .iter()
                    .find(|event| event.event_type == "price_tick")
                    .or_else(|| {
                        catalog
                            .events
                            .iter()
                            .find(|event| event.event_type == "market.tick")
                    })
                    .or_else(|| {
                        catalog
                            .events
                            .iter()
                            .find(|event| event.source_kind == MachineEventSourceKind::MarketData)
                    })
            })
            .map(|event| event.event_type.clone())
            .unwrap_or_else(|| "price_tick".to_string());
        let venue_id = resolve_v4_runner_venue_id(&graph);
        let default_symbol = resolve_v4_runner_default_symbol(&graph, &s.subscribed_symbols);
        let runtime = V4PaperSimulatedRuntime::new_with_execution_capabilities(
            graph,
            executor_v4_market_matrix(&venue_id),
            vec![qrpc_core_ir::v4::ExecutionCapabilityKind::Market],
        )?
        .with_simulated_execution_config(V4SimulatedExecutionConfig {
            default_venue_id: venue_id.clone(),
            default_symbol,
            ..V4SimulatedExecutionConfig::default()
        })?;

        Ok(Self {
            strategy_id: s.strategy_id.clone(),
            runtime,
            venue_id,
            subscribed_symbols: s.subscribed_symbols.clone(),
            kline_pool: KlinePool::new(Self::KLINE_POOL_CAPACITY),
            status: if s.execution_mode.starts_without_provider_connection() {
                RunnerStatus::Running
            } else {
                RunnerStatus::Idle
            },
            market_event_type,
            evidence_broadcast,
        })
    }

    pub fn handle_ws_event(&mut self, event: WsEvent) {
        match event {
            WsEvent::Ticker {
                symbol,
                price,
                ts_ms,
            } => {
                if self.status != RunnerStatus::Running || !self.is_subscribed_to(&symbol) {
                    return;
                }
                self.kline_pool.update_from_ticker(&symbol, price, ts_ms);
                match self.runtime.submit_market_price_tick(
                    &self.venue_id,
                    &symbol,
                    price,
                    ts_ms,
                    &self.market_event_type,
                ) {
                    Ok(output) => self.broadcast_evidence(output),
                    Err(e) => {
                        self.status = RunnerStatus::Faulted(e.to_string());
                        eprintln!("[runner:{}] v4 price_tick error: {:?}", self.strategy_id, e);
                    }
                }
            }
            WsEvent::Kline { symbol, bar } => {
                if self.status != RunnerStatus::Running || !self.is_subscribed_to(&symbol) {
                    return;
                }
                let close_ms = bar.close_time_ms;
                let close = bar.close;
                self.kline_pool.update_kline(&symbol, bar);
                match self.runtime.submit_market_bar_closed(
                    &self.venue_id,
                    &symbol,
                    close,
                    close_ms,
                    &self.market_event_type,
                ) {
                    Ok(output) => self.broadcast_evidence(output),
                    Err(e) => {
                        self.status = RunnerStatus::Faulted(e.to_string());
                        eprintln!("[runner:{}] v4 bar_closed error: {:?}", self.strategy_id, e);
                    }
                }
            }
            WsEvent::Connected { exchange } => {
                eprintln!(
                    "[runner:{}] {} websocket connected for v4 runtime",
                    self.strategy_id, exchange
                );
                if self.status == RunnerStatus::Idle {
                    self.status = RunnerStatus::Running;
                }
                let snapshot = self.runtime.memory_snapshot(0);
                let _ = self.evidence_broadcast.send(V4ExecutorEvidenceEvent {
                    strategy_id: self.strategy_id.clone(),
                    event_type: "v4RuntimeMemorySnapshot".to_string(),
                    memory_snapshot: snapshot,
                    runtime_events: Vec::new(),
                });
            }
            #[cfg(test)]
            WsEvent::Disconnected { exchange, reason } => {
                eprintln!(
                    "[runner:{}] {} websocket disconnected: {}",
                    self.strategy_id, exchange, reason
                );
                self.status = RunnerStatus::Faulted(reason);
            }
        }
    }

    fn broadcast_evidence(&self, output: V4PaperSimulatedRunOutput) {
        let _ = self.evidence_broadcast.send(V4ExecutorEvidenceEvent {
            strategy_id: self.strategy_id.clone(),
            event_type: "v4RuntimeMemorySnapshot".to_string(),
            memory_snapshot: output.memory_snapshot,
            runtime_events: output.events,
        });
    }

    fn is_subscribed_to(&self, symbol: &str) -> bool {
        self.subscribed_symbols.is_empty()
            || self
                .subscribed_symbols
                .iter()
                .any(|item| item.as_str().eq_ignore_ascii_case(symbol))
    }
}
