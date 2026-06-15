use crate::executor_state::{ActiveStrategy, TriggerEvent};
use crate::ws_client::WsEvent;
use std::collections::{BTreeMap, HashMap};
use tokio::sync::{broadcast, mpsc};

use super::{RunnerInstance, V4ExecutorEvidenceEvent};

pub struct RunnerPool {
    pub runners: BTreeMap<String, RunnerInstance>,
    pub trigger_broadcast: broadcast::Sender<TriggerEvent>,
    pub v4_evidence_broadcast: broadcast::Sender<V4ExecutorEvidenceEvent>,
    pub ws_tx_map: HashMap<String, mpsc::UnboundedSender<WsEvent>>,
    pub symbol_index: HashMap<String, Vec<String>>,
}

impl RunnerPool {
    pub fn new(bc: broadcast::Sender<TriggerEvent>) -> Self {
        let (v4_evidence_broadcast, _) = broadcast::channel(256);
        Self {
            runners: BTreeMap::new(),
            trigger_broadcast: bc,
            v4_evidence_broadcast,
            ws_tx_map: HashMap::new(),
            symbol_index: HashMap::new(),
        }
    }

    pub fn register_exchange(&mut self, exchange: &str, tx: mpsc::UnboundedSender<WsEvent>) {
        self.ws_tx_map.insert(exchange.into(), tx);
    }

    pub fn register(&mut self, s: &ActiveStrategy) -> anyhow::Result<()> {
        let sid = s.strategy_id.clone();
        for sym in &s.subscribed_symbols {
            let key = sym.as_str().to_uppercase();
            self.symbol_index.entry(key).or_default().push(sid.clone());
        }
        self.runners.insert(
            sid,
            RunnerInstance::from_strategy(
                s,
                self.trigger_broadcast.clone(),
                self.v4_evidence_broadcast.clone(),
            )?,
        );
        Ok(())
    }

    pub fn remove(&mut self, strategy_id: &str) {
        for ids in self.symbol_index.values_mut() {
            ids.retain(|id| id != strategy_id);
        }
        if let Some(runner) = self.runners.get_mut(strategy_id) {
            runner.set_stopped();
        }
        self.runners.remove(strategy_id);
    }

    pub fn broadcast_ws_event(&mut self, event: WsEvent) {
        match &event {
            WsEvent::Ticker { symbol, .. } | WsEvent::Kline { symbol, .. } => {
                let key = symbol.to_uppercase();
                let mut target_ids: Vec<String> =
                    self.symbol_index.get(&key).cloned().unwrap_or_default();
                for (id, runner) in &self.runners {
                    if runner.subscribed_symbols().is_empty() && !target_ids.contains(id) {
                        target_ids.push(id.clone());
                    }
                }
                for id in &target_ids {
                    if let Some(runner) = self.runners.get_mut(id) {
                        runner.handle_ws_event(event.clone());
                    }
                }
            }
            WsEvent::Connected { .. } => {
                for runner in self.runners.values_mut() {
                    runner.handle_ws_event(event.clone());
                }
            }
            #[cfg(test)]
            WsEvent::Disconnected { .. } => {
                for runner in self.runners.values_mut() {
                    runner.handle_ws_event(event.clone());
                }
            }
        }
    }
}
