/// v3.7.0/v4.8.0: 实时策略运行器 — PaperSimulated 本地撮合 / PaperActual provider 回执
use crate::executor_state::{ActiveStrategy, ExecutionMode, RuntimeKind, TriggerEvent};
use crate::kline_buffer::KlinePool;
use crate::ws_client::WsEvent;
use qrpc_core::{RuntimeEvent, RuntimeEventType, Symbol};
use qrpc_core_ir::v4::MachineEventSourceKind;
use qrpc_runtime::{
    RuntimeCoordinator, V4PaperSimulatedRunOutput, V4PaperSimulatedRuntime,
    V4RuntimeMemorySnapshot, V4SimulatedExecutionConfig,
};
use std::collections::{BTreeMap, HashMap};
use tokio::sync::{broadcast, mpsc};

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
            execution_mode: s.execution_mode.clone(),
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

    fn detect_trigger(sid: &str, event: &RuntimeEvent) -> Option<TriggerEvent> {
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

pub struct RunnerPool {
    pub runners: BTreeMap<String, RunnerInstance>,
    pub trigger_broadcast: broadcast::Sender<TriggerEvent>,
    pub v4_evidence_broadcast: broadcast::Sender<V4ExecutorEvidenceEvent>,
    pub ws_tx_map: HashMap<String, mpsc::UnboundedSender<WsEvent>>,
    /// v3.3.0: symbol→[strategy_id] 反向索引, O(1)查找替代O(N*M)遍历
    pub symbol_index: HashMap<String, Vec<String>>,
}

pub enum RunnerInstance {
    V3(LiveRunner),
    V4(V4Runner),
}

impl RunnerInstance {
    fn from_strategy(
        s: &ActiveStrategy,
        trigger_broadcast: broadcast::Sender<TriggerEvent>,
        v4_evidence_broadcast: broadcast::Sender<V4ExecutorEvidenceEvent>,
    ) -> anyhow::Result<Self> {
        match s.runtime_kind {
            RuntimeKind::V3 => Ok(Self::V3(LiveRunner::from_strategy(s, trigger_broadcast))),
            RuntimeKind::V4 => Ok(Self::V4(V4Runner::from_strategy(s, v4_evidence_broadcast)?)),
        }
    }

    fn handle_ws_event(&mut self, event: WsEvent) {
        match self {
            Self::V3(runner) => runner.handle_ws_event(event),
            Self::V4(runner) => runner.handle_ws_event(event),
        }
    }

    fn subscribed_symbols(&self) -> &[Symbol] {
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

    fn set_stopped(&mut self) {
        match self {
            Self::V3(runner) => runner.status = RunnerStatus::Stopped,
            Self::V4(runner) => runner.status = RunnerStatus::Stopped,
        }
    }
}

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
    const DEFAULT_REALTIME_PAPER_VENUE_ID: &'static str = "paper-simulated";

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
        // v3.3.0: 建立反向索引
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
    /// v3.2.2: 从RunnerPool移除停止的策略
    pub fn remove(&mut self, strategy_id: &str) {
        // v3.3.0: 清理反向索引
        for ids in self.symbol_index.values_mut() {
            ids.retain(|id| id != strategy_id);
        }
        if let Some(runner) = self.runners.get_mut(strategy_id) {
            runner.set_stopped();
        }
        self.runners.remove(strategy_id);
    }
    pub fn broadcast_ws_event(&mut self, event: WsEvent) {
        // v3.3.0: 反向索引 O(1) — 仅遍历订阅了对应symbol的策略
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

fn executor_v4_market_matrix(
    venue_id: impl Into<String>,
) -> qrpc_core_ir::v4::VenueCapabilityMatrix {
    let mut matrix = qrpc_core_ir::v4::unsupported_v4_first_wave_matrix(venue_id);
    for entry in &mut matrix.capabilities {
        if matches!(
            entry.capability,
            qrpc_core_ir::v4::ExecutionCapabilityKind::Market
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Gtc
                | qrpc_core_ir::v4::ExecutionCapabilityKind::ClientOrderId
        ) {
            entry.source = qrpc_core_ir::v4::CapabilitySupportSource::RuntimeSimulated;
            entry.supported_modes = vec![qrpc_core_ir::v4::RuntimeTradingMode::PaperSimulated];
        }
    }
    matrix
}

fn resolve_v4_runner_venue_id(graph: &qrpc_core_ir::v4::V4MachineGraphContract) -> String {
    graph_metadata_string(graph, "default_venue_id")
        .or_else(|| graph_metadata_string(graph, "core_venue_kind"))
        .unwrap_or_else(|| V4Runner::DEFAULT_REALTIME_PAPER_VENUE_ID.to_string())
}

fn resolve_v4_runner_default_symbol(
    graph: &qrpc_core_ir::v4::V4MachineGraphContract,
    subscribed_symbols: &[Symbol],
) -> String {
    graph_metadata_string(graph, "default_symbol")
        .or_else(|| {
            graph
                .metadata
                .get("symbols")
                .and_then(|value| value.as_array())
                .and_then(|symbols| symbols.first())
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
        .or_else(|| {
            subscribed_symbols
                .first()
                .map(|symbol| symbol.as_str().to_string())
        })
        .unwrap_or_else(|| "BTCUSDT".to_string())
}

fn graph_metadata_string(
    graph: &qrpc_core_ir::v4::V4MachineGraphContract,
    key: &str,
) -> Option<String> {
    graph
        .metadata
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::{CoreStrategyIr, RuntimeEvent};
    use qrpc_core_ir::{
        v4::{RuntimeTradingMode, V4MachineGraphContract, V4StaticContractBundle},
        CoreMetadata, CoreSourceKind, CoreTimeInForce, ExecutionRule, ExecutionSizingKind,
    };
    use qrpc_runtime::{
        EVENT_EXECUTION_FEE_CHARGED, EVENT_EXECUTION_ORDER_ACKNOWLEDGED,
        EVENT_EXECUTION_ORDER_FILLED, V4_DEFAULT_MARKET_DATA_SOURCE,
    };

    const SAMPLE_REALTIME_V4_QS: &str = r#"
v4_strategy strategy.v4.w0_1 {
  venue paper-simulated
  mode paper_simulated
  require capability market

  machine data.market observation priority 8000 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on market.tick from idle to ready emit bar_closed write last_signal_at
  }

  machine risk.guard decision priority 9500 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on bar_closed from idle to ready emit risk.approved write last_signal_at
  }

  machine execution.router execution priority 4000 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on risk.approved from idle to ready write last_signal_at
  }

  edge data.market -> risk.guard on bar_closed
  edge risk.guard -> execution.router on risk.approved
  risk_plane risk.guard priority 9000
}
"#;

    fn sample_realtime_graph() -> V4MachineGraphContract {
        let bundle = V4StaticContractBundle {
            venue_matrices: vec![executor_v4_market_matrix("paper-simulated")],
            ..V4StaticContractBundle::default()
        };
        let report = quantscript::audit_v4_quant_script_static(SAMPLE_REALTIME_V4_QS, &bundle);
        let handoff = quantscript::build_v4_qs_runtime_handoff(&report);
        assert!(
            handoff.accepted_for_runtime_handoff,
            "expected realtime sample graph to pass handoff: {:?}",
            handoff.diagnostics
        );
        report.parsed_graph.expect("sample v4 graph should parse")
    }

    fn empty_core_ir(strategy_id: &str) -> CoreStrategyIr {
        CoreStrategyIr::new(
            CoreMetadata {
                strategy_id: strategy_id.to_string(),
                name: strategy_id.to_string(),
                source_kind: CoreSourceKind::RuntimeProtocol,
            },
            ExecutionRule {
                execution_id: format!("exec_{strategy_id}"),
                venue_kind: "paper".into(),
                sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
                slippage_bps: 0.0,
                taker_fee_bps: 0.0,
                total_cost_buffer_bps: 0.0,
                time_in_force: CoreTimeInForce::Gtc,
                params: BTreeMap::new(),
            },
        )
    }

    #[test]
    fn detect_trigger_intent_signal() {
        let event = RuntimeEvent {
            event_id: "evt-1".into(),
            event_type: RuntimeEventType::IntentTriggered,
            trace_id: "t1".into(),
            source_id: "ind_1".into(),
            ts_ms: 1000,
            payload: serde_json::json!({"strength": 0.85, "indicator_id": "ma_cross"}),
        };
        let t = LiveRunner::detect_trigger("s1", &event).unwrap();
        assert_eq!(t.strategy_id, "s1");
        assert_eq!(t.trigger_type, "intent_triggered");
        assert_eq!(t.node_id, "ma_cross");
        assert_eq!(t.strength, 0.85);
    }

    #[test]
    fn detect_trigger_agent_decision() {
        let event = RuntimeEvent {
            event_id: "evt-2".into(),
            event_type: RuntimeEventType::AgentDecisionProduced,
            trace_id: "t2".into(),
            source_id: "agent_1".into(),
            ts_ms: 2000,
            payload: serde_json::json!({"net_strength": 0.6, "agent_id": "a1"}),
        };
        let t = LiveRunner::detect_trigger("s1", &event).unwrap();
        assert_eq!(t.trigger_type, "agent_decided");
        assert_eq!(t.node_id, "a1");
    }

    #[test]
    fn detect_trigger_unknown_returns_none() {
        let event = RuntimeEvent {
            event_id: "evt-3".into(),
            event_type: RuntimeEventType::DataUpdated,
            trace_id: "t3".into(),
            source_id: "d1".into(),
            ts_ms: 3000,
            payload: serde_json::json!({}),
        };
        assert!(LiveRunner::detect_trigger("s1", &event).is_none());
    }

    #[test]
    fn v4_runner_realtime_paper_simulated_tick_closes_local_execution_loop() {
        let strategy_id = "w0_1_realtime_paper_simulated";
        let strategy = ActiveStrategy {
            strategy_id: strategy_id.to_string(),
            name: "W0-1 realtime paper simulated".to_string(),
            runtime_kind: RuntimeKind::V4,
            core_ir: empty_core_ir(strategy_id),
            v4_graph: Some(sample_realtime_graph()),
            graph_json: serde_json::Value::Null,
            params: BTreeMap::new(),
            status: crate::executor_state::StrategyStatus::Loaded,
            subscribed_symbols: vec![Symbol::Other("BTCUSDT".to_string())],
            execution_mode: ExecutionMode::PaperSimulated,
        };
        let (trigger_broadcast, _) = broadcast::channel(16);
        let mut pool = RunnerPool::new(trigger_broadcast);
        pool.register(&strategy).unwrap();
        match pool.runners.get(strategy_id).unwrap() {
            RunnerInstance::V4(runner) => assert_eq!(runner.venue_id, "paper-simulated"),
            RunnerInstance::V3(_) => panic!("expected v4 runner"),
        }
        let mut evidence_rx = pool.v4_evidence_broadcast.subscribe();

        pool.broadcast_ws_event(WsEvent::Ticker {
            symbol: "BTCUSDT".to_string(),
            price: 70_000.0,
            ts_ms: 123,
        });

        let evidence = evidence_rx
            .try_recv()
            .expect("v4 runner should broadcast evidence after realtime tick");
        assert_eq!(evidence.strategy_id, strategy_id);
        assert_eq!(
            evidence.memory_snapshot.runtime_mode,
            RuntimeTradingMode::PaperSimulated
        );
        assert!(!evidence.memory_snapshot.provider_order_submission_attached);
        assert!(
            !evidence
                .memory_snapshot
                .venue_adapter_boundary
                .provider_order_submission_attached
        );
        assert!(
            evidence
                .memory_snapshot
                .venue_adapter_boundary
                .rejection_before_provider_submit
        );
        assert_eq!(evidence.memory_snapshot.simulated_execution.order_count, 1);
        assert_eq!(evidence.memory_snapshot.simulated_execution.fill_count, 1);
        assert_eq!(
            evidence
                .memory_snapshot
                .simulated_execution
                .last_fill
                .as_ref()
                .map(|fill| fill.venue_id.as_str()),
            Some("paper-simulated")
        );
        assert!(evidence.runtime_events.iter().any(|event| {
            event.event_type == "market.tick" && event.source == V4_DEFAULT_MARKET_DATA_SOURCE
        }));
        assert!(evidence
            .runtime_events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_ORDER_ACKNOWLEDGED));
        assert!(evidence
            .runtime_events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_ORDER_FILLED));
        assert!(evidence
            .runtime_events
            .iter()
            .any(|event| event.event_type == EVENT_EXECUTION_FEE_CHARGED));
    }
}
