use crate::data_module::mock_kline_bars_for_backtest;
use crate::data_module::{
    attach_data_quality_snapshot, build_data_quality_summary, data_sources_from_core_ir,
    historical_kline_bars_for_backtest, market_data_preview, market_data_quality,
    quote_snapshot_from_price, DataCollectionOutput, DataCollectionRequest, DataModuleProvider,
    FetchDiagnostics,
};
use crate::RuntimeCoordinator;
use anyhow::{anyhow, Result};
use qrpc_core::{
    BacktestEquityPoint, BacktestOutput, BacktestSummary, CompiledRuntimeProtocol, CoreStrategyIr,
    DataKind, DataSourceConfig, ExecutionPlan, FillResult, HandoffSnapshot, KlineSeriesSnapshot,
    NormalizedMarketData, PortfolioState, QuoteSnapshot, RuntimeEvent, SessionOutput,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Mutex;

pub const SUPPORTED_RUNTIME_MODE_KEYS: [&str; 1] = ["paper"];
pub const SUPPORTED_RUNTIME_EXECUTION_MODULE_KEYS: [&str; 1] = ["builtin.execution.paper"];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxMode {
    RealTimeSimulation,
    FastBacktest,
}

#[derive(Debug, Clone)]
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
    /// 存储候选模块配置，返回新的 deployment_revision
    fn swap_module_config(&mut self, module_key: &str, config: serde_json::Value) -> Result<String>;

    /// v1.0.0 热接管: 策略 A 提交快照 → Sandbox 校验 → 策略 B 接管
    /// 默认返回错误，仅支持热接管的 Sandbox 实现重写
    fn handoff(&mut self, _snapshot: &HandoffSnapshot) -> Result<()> {
        Err(anyhow::anyhow!("当前 Sandbox 不支持热接管"))
    }
}

#[derive(Debug, Clone)]
struct ReplayFrame {
    ts_ms: u64,
    kline_data: BTreeMap<String, KlineSeriesSnapshot>,
    quote_data: BTreeMap<String, QuoteSnapshot>,
}

#[derive(Debug)]
struct ReplayDataModule {
    frames: Vec<ReplayFrame>,
    next_step: Mutex<usize>,
}

impl ReplayDataModule {
    fn new(core_ir: &qrpc_core::CoreStrategyIr, end_ms: u64) -> Result<(Self, Vec<u64>)> {
        let data_sources = data_sources_from_core_ir(core_ir);
        let kline_bars = data_sources
            .iter()
            .filter(|source| source.enabled && matches!(source.kind, DataKind::KlineSeries))
            .cloned()
            .map(|source| {
                Ok((
                    source.data_id.clone(),
                    source.clone(),
                    historical_kline_bars_for_backtest(&source, end_ms)?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Self::from_kline_bars(core_ir, kline_bars, end_ms)
    }

    fn from_kline_bars(
        core_ir: &qrpc_core::CoreStrategyIr,
        kline_bars: Vec<(String, DataSourceConfig, Vec<qrpc_core::NormalizedKline>)>,
        end_ms: u64,
    ) -> Result<(Self, Vec<u64>)> {
        let data_sources = data_sources_from_core_ir(core_ir);
        let kline_sources = data_sources
            .iter()
            .filter(|source| source.enabled && matches!(source.kind, DataKind::KlineSeries))
            .cloned()
            .collect::<Vec<_>>();
        if kline_sources.is_empty() {
            return Err(anyhow!(
                "回测需要至少一个启用的 K 线数据源"
            ));
        }
        let step_count = kline_bars
            .iter()
            .map(|(_, _, bars)| bars.len())
            .min()
            .unwrap_or(0);
        if step_count == 0 {
            return Err(anyhow!("回测需要非空的历史 K 线数据"));
        }

        let quote_sources = data_sources
            .iter()
            .filter(|source| source.enabled && matches!(source.kind, DataKind::Quote))
            .cloned()
            .collect::<Vec<_>>();

        let mut frames = Vec::with_capacity(step_count);
        let mut timestamps = Vec::with_capacity(step_count);
        for step in 0..step_count {
            let ts_ms = kline_bars
                .iter()
                .filter_map(|(_, _, bars)| bars.get(step).map(|bar| bar.close_time_ms))
                .max()
                .unwrap_or(end_ms);
            let mut kline_data = BTreeMap::new();
            for (data_id, source, bars) in &kline_bars {
                kline_data.insert(
                    data_id.clone(),
                    KlineSeriesSnapshot {
                        data_id: data_id.clone(),
                        exchange: source.exchange.clone(),
                        symbol: source.symbol.clone(),
                        market_type: source.market_type.clone(),
                        interval: source.interval.clone().unwrap_or_else(|| "1d".into()),
                        bars: bars[..=step].to_vec(),
                        window_len: step + 1,
                        ts_ms,
                        source_latency_ms: 0,
                        source_status: qrpc_core::SourceStatus::Healthy,
                        data_quality: qrpc_core::DataQualitySnapshot::default(),
                    },
                );
            }

            let mut quote_data = BTreeMap::new();
            for source in &quote_sources {
                let fallback_price = kline_bars
                    .iter()
                    .find(|(_, kline_source, _)| {
                        kline_source.exchange == source.exchange
                            && kline_source.symbol == source.symbol
                    })
                    .and_then(|(_, _, bars)| bars.get(step).map(|bar| bar.close))
                    .unwrap_or(50_000.0);
                quote_data.insert(
                    source.data_id.clone(),
                    quote_snapshot_from_price(source, fallback_price, ts_ms),
                );
            }

            frames.push(ReplayFrame {
                ts_ms,
                kline_data,
                quote_data,
            });
            timestamps.push(ts_ms);
        }

        Ok((
            Self {
                frames,
                next_step: Mutex::new(0),
            },
            timestamps,
        ))
    }
}

impl DataModuleProvider for ReplayDataModule {
    fn provider_key(&self) -> &'static str {
        "builtin.data.backtest_replay"
    }

    fn collect(&self, request: DataCollectionRequest<'_>) -> Result<DataCollectionOutput> {
        let step = *self
            .next_step
            .lock()
            .map_err(|_| anyhow!("回放数据模块步骤互斥锁中毒"))?;
        let frame = self
            .frames
            .get(step)
            .or_else(|| self.frames.last())
            .ok_or_else(|| anyhow!("回放数据模块没有帧数据"))?;

        let mut normalized_data = Vec::new();
        let mut events = Vec::new();
        let data_sources = data_sources_from_core_ir(request.core_ir);
        for source in data_sources.iter().filter(|item| item.enabled) {
            *request
                .data_fetch_counts
                .entry(source.data_id.clone())
                .or_default() += 1;

            let payload = match source.kind {
                DataKind::KlineSeries => frame
                    .kline_data
                    .get(&source.data_id)
                    .cloned()
                    .map(NormalizedMarketData::KlineSeries),
                DataKind::Quote => frame
                    .quote_data
                    .get(&source.data_id)
                    .cloned()
                    .map(NormalizedMarketData::Quote),
            }
            .ok_or_else(|| anyhow!("缺失 {} 的回放帧数据", source.data_id))?;
            let diagnostics = FetchDiagnostics {
                provider_key: self.provider_key(),
                source_status: qrpc_core::SourceStatus::Healthy,
                source_latency_ms: 0,
                endpoint: None,
                ping_latency_ms: None,
                ping_endpoint: None,
                ping_error: None,
                fallback: None,
                error: None,
            };
            let payload =
                attach_data_quality_snapshot(source, payload, &diagnostics, request.now_ms);
            let preview = market_data_preview(&payload);
            let quality = market_data_quality(&payload);
            let quality_summary =
                build_data_quality_summary(source, &quality, &diagnostics, preview.latest_price);

            events.push(RuntimeEvent {
                event_id: format!("evt-backtest-data-{}-{}", source.data_id, frame.ts_ms),
                event_type: qrpc_core::RuntimeEventType::DataUpdated,
                trace_id: request.trace_id.to_string(),
                source_id: source.data_id.clone(),
                ts_ms: frame.ts_ms,
                payload: json!({
                    "provider_key": self.provider_key(),
                    "replay_step": step,
                    "ts_ms": frame.ts_ms,
                    "exchange": format!("{:?}", source.exchange),
                    "kind": format!("{:?}", source.kind),
                    "source_status": format!("{:?}", diagnostics.source_status),
                    "source_latency_ms": diagnostics.source_latency_ms,
                    "source_health": format!("{:?}", quality.source_health),
                    "freshness_ms": quality.freshness_ms,
                    "stale_after_ms": quality.stale_after_ms,
                    "gap_count": quality.gap_count,
                    "quality_flags": quality.quality_flags,
                    "latest_price": preview.latest_price,
                    "latest_bar_time": preview.latest_bar_time,
                    "bid_price": preview.bid_price,
                    "ask_price": preview.ask_price,
                    "endpoint": diagnostics.endpoint,
                    "ping_latency_ms": diagnostics.ping_latency_ms,
                    "ping_endpoint": diagnostics.ping_endpoint,
                    "ping_error": diagnostics.ping_error,
                    "fallback": diagnostics.fallback,
                    "error": diagnostics.error,
                    "explanation_summary": quality_summary,
                }),
            });
            normalized_data.push(payload);
        }

        if request.cycle_name == "fast" {
            let mut guard = self
                .next_step
                .lock()
                .map_err(|_| anyhow!("回放数据模块步骤互斥锁中毒"))?;
            if *guard + 1 < self.frames.len() {
                *guard += 1;
            }
        }

        Ok(DataCollectionOutput {
            normalized_data,
            events,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RealTimeSandbox {
    coordinator: RuntimeCoordinator,
    running: bool,
    test_mode: DeterministicTestMode,
}

impl RealTimeSandbox {
    pub fn new(coordinator: RuntimeCoordinator) -> Self {
        Self::with_test_mode(coordinator, DeterministicTestMode::default())
    }

    pub fn with_test_mode(
        coordinator: RuntimeCoordinator,
        test_mode: DeterministicTestMode,
    ) -> Self {
        Self {
            coordinator,
            running: false,
            test_mode,
        }
    }

    pub fn from_core_ir(core_ir: CoreStrategyIr) -> Self {
        Self::new(RuntimeCoordinator::from_core_ir(core_ir))
    }

    pub fn from_compiled(compiled: CompiledRuntimeProtocol) -> Self {
        Self::from_core_ir(compiled.core_ir)
    }

    pub fn test_mode(&self) -> &DeterministicTestMode {
        &self.test_mode
    }
}

impl Sandbox for RealTimeSandbox {
    fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn mode(&self) -> SandboxMode {
        SandboxMode::RealTimeSimulation
    }

    fn run_session(&mut self, slow_now_ms: u64, fast_now_ms: u64) -> Result<SessionOutput> {
        ensure_running(self.running, "real-time sandbox")?;
        self.coordinator.run_session(slow_now_ms, fast_now_ms)
    }

    fn submit_execution_plan(
        &mut self,
        plan: ExecutionPlan,
        normalized_data: Vec<NormalizedMarketData>,
        now_ms: u64,
    ) -> Result<FillResult> {
        ensure_running(self.running, "real-time sandbox")?;
        self.coordinator.submit_execution_plan(
            &plan,
            &normalized_data,
            now_ms,
            &trace_id("rt-plan", now_ms),
        )
    }

    fn on_market_data(
        &mut self,
        normalized_data: Vec<NormalizedMarketData>,
        now_ms: u64,
    ) -> Result<Vec<RuntimeEvent>> {
        ensure_running(self.running, "real-time sandbox")?;
        let trace_id = trace_id("rt-market", now_ms);
        let mut events = self
            .coordinator
            .on_market_data(&normalized_data, now_ms, &trace_id)?
            .events;
        events.push(self.coordinator.portfolio_update_event(
            "sandbox_market_data",
            &trace_id,
            now_ms,
        ));
        Ok(events)
    }

    fn snapshot(&self, now_ms: u64) -> SandboxSnapshot {
        snapshot_from(
            &self.coordinator,
            self.mode(),
            self.running,
            &self.test_mode,
            now_ms,
        )
    }

    fn swap_module_config(&mut self, module_key: &str, config: serde_json::Value) -> Result<String> {
        self.coordinator.swap_module_config(module_key, config)
    }

    /// v1.0.0 热接管: 校验快照完整性, 交由调用方使用快照启动接管策略
    fn handoff(&mut self, snapshot: &HandoffSnapshot) -> Result<()> {
        snapshot.validate_completeness().map_err(|errs| {
            anyhow::anyhow!("热接管快照校验失败: {:?}", errs)
        })?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FastBacktestSandbox {
    coordinator: RuntimeCoordinator,
    running: bool,
    replay_timestamps: Vec<u64>,
    latency_assumption_ms: u64,
    test_mode: DeterministicTestMode,
    pub debug_var_names: Vec<String>,
}

impl FastBacktestSandbox {
    pub fn new(coordinator: RuntimeCoordinator) -> Self {
        Self::with_test_mode(coordinator, DeterministicTestMode::default())
    }

    pub fn with_test_mode(
        coordinator: RuntimeCoordinator,
        test_mode: DeterministicTestMode,
    ) -> Self {
        Self {
            coordinator,
            running: false,
            replay_timestamps: Vec::new(),
            latency_assumption_ms: 0,
            test_mode,
            debug_var_names: Vec::new(),
        }
    }

    pub fn with_replay_from_core_ir(core_ir: CoreStrategyIr, end_ms: u64) -> Result<Self> {
        Self::with_replay_from_core_ir_and_test_mode(
            core_ir,
            end_ms,
            DeterministicTestMode::default(),
        )
    }

    pub fn with_replay_from_core_ir_and_test_mode(
        core_ir: CoreStrategyIr,
        end_ms: u64,
        test_mode: DeterministicTestMode,
    ) -> Result<Self> {
        let (replay_module, replay_timestamps) = ReplayDataModule::new(&core_ir, end_ms)?;
        Ok(Self {
            coordinator: RuntimeCoordinator::with_data_module_from_core_ir(core_ir, replay_module),
            running: false,
            replay_timestamps,
            latency_assumption_ms: 0,
            test_mode,
            debug_var_names: Vec::new(),
        })
    }

    pub fn with_replay(compiled: CompiledRuntimeProtocol, end_ms: u64) -> Result<Self> {
        Self::with_replay_from_core_ir(compiled.core_ir, end_ms)
    }

    pub fn with_mock_replay_from_core_ir(core_ir: CoreStrategyIr, end_ms: u64) -> Result<Self> {
        Self::with_mock_replay_from_core_ir_and_test_mode(
            core_ir,
            end_ms,
            DeterministicTestMode::default(),
        )
    }

    pub fn with_mock_replay_from_core_ir_and_test_mode(
        core_ir: CoreStrategyIr,
        end_ms: u64,
        test_mode: DeterministicTestMode,
    ) -> Result<Self> {
        let core_data_sources = data_sources_from_core_ir(&core_ir);
        let kline_bars = core_data_sources
            .iter()
            .filter(|source| source.enabled && matches!(source.kind, DataKind::KlineSeries))
            .cloned()
            .map(|source| {
                Ok((
                    source.data_id.clone(),
                    source.clone(),
                    mock_kline_bars_for_backtest(&source, end_ms)?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let (replay_module, replay_timestamps) =
            ReplayDataModule::from_kline_bars(&core_ir, kline_bars, end_ms)?;
        Ok(Self {
            coordinator: RuntimeCoordinator::with_data_module_from_core_ir(core_ir, replay_module),
            running: false,
            replay_timestamps,
            latency_assumption_ms: 0,
            test_mode,
            debug_var_names: Vec::new(),
        })
    }

    pub fn with_mock_replay(compiled: CompiledRuntimeProtocol, end_ms: u64) -> Result<Self> {
        Self::with_mock_replay_from_core_ir(compiled.core_ir, end_ms)
    }

    pub fn test_mode(&self) -> &DeterministicTestMode {
        &self.test_mode
    }

    pub fn set_latency_assumption_ms(&mut self, latency_ms: u64) {
        self.latency_assumption_ms = latency_ms;
    }

    pub fn run_backtest(&mut self) -> Result<BacktestOutput> {
        ensure_running(self.running, "fast backtest sandbox")?;
        if self.replay_timestamps.is_empty() {
            return Err(anyhow!("回测回放帧未配置"));
        }

        let started_at_ms = self
            .replay_timestamps
            .first()
            .copied()
            .unwrap_or(0)
            .saturating_add(self.latency_assumption_ms);
        let ended_at_ms = self
            .replay_timestamps
            .last()
            .copied()
            .unwrap_or(started_at_ms)
            .saturating_add(self.latency_assumption_ms)
            .saturating_add(1);
        let mut sessions = Vec::with_capacity(self.replay_timestamps.len());
        let mut equity_curve = Vec::with_capacity(self.replay_timestamps.len());
        let mut peak_equity = self.coordinator.portfolio_state().cash_balance;
        let mut max_drawdown_ratio = 0.0_f64;
        let mut trade_count = 0_usize;
        let mut debug_rows: Vec<BTreeMap<String, f64>> = Vec::new();

        for ts_ms in self.replay_timestamps.clone() {
            let slow_now_ms = ts_ms.saturating_add(self.latency_assumption_ms);
            let fast_now_ms = slow_now_ms.saturating_add(1);
            let session = self.coordinator.run_session(slow_now_ms, fast_now_ms)?;
            let equity =
                session.final_portfolio.cash_balance + session.final_portfolio.total_net_notional;
            peak_equity = peak_equity.max(equity);
            if peak_equity > 0.0 {
                max_drawdown_ratio = max_drawdown_ratio.max((peak_equity - equity) / peak_equity);
            }
            trade_count +=
                session.slow_cycle.fill_reports.len() + session.fast_cycle.fill_reports.len();
            equity_curve.push(BacktestEquityPoint {
                ts_ms: session.final_portfolio.updated_at_ms,
                equity,
                cash_balance: session.final_portfolio.cash_balance,
                net_notional: session.final_portfolio.total_net_notional,
            });
            if !self.debug_var_names.is_empty() {
                let mut row = BTreeMap::new();
                for signal in session.slow_cycle.intent_signals.iter().chain(session.fast_cycle.intent_signals.iter()) {
                    for (key, value) in &signal.derived_metrics {
                        if self.debug_var_names.iter().any(|v| key.contains(v.as_str()) || v.as_str().contains(key.as_str())) {
                            row.insert(key.clone(), *value);
                        }
                    }
                }
                debug_rows.push(row);
            }
            sessions.push(session);
        }

        let initial_equity = equity_curve
            .first()
            .map(|point| point.equity)
            .unwrap_or(self.coordinator.portfolio_state().cash_balance);
        let final_equity = equity_curve
            .last()
            .map(|point| point.equity)
            .unwrap_or(initial_equity);
        let total_return_ratio = if initial_equity.abs() > f64::EPSILON {
            (final_equity - initial_equity) / initial_equity
        } else {
            0.0
        };

        // Compute win_rate from equity curve
        let mut wins = 0usize;
        let mut total_steps = 0usize;
        for window in equity_curve.windows(2) {
            total_steps += 1;
            if window[1].equity > window[0].equity { wins += 1; }
        }
        let win_rate = if total_steps > 0 { wins as f64 / total_steps as f64 } else { 0.0 };

        Ok(BacktestOutput {
            mode: "historical_replay".into(),
            started_at_ms,
            ended_at_ms,
            sessions,
            equity_curve,
            summary: BacktestSummary {
                step_count: self.replay_timestamps.len(),
                trade_count,
                total_return_ratio,
                max_drawdown_ratio,
                final_equity,
                net_profit: final_equity - initial_equity,
                turnover_ratio: 0.0,
                average_trade_notional: 0.0,
                fee_drag_ratio: 0.0,
                win_rate,
            },
            final_portfolio: self.coordinator.portfolio_state().clone(),
            debug_values: if self.debug_var_names.is_empty() { None } else { Some(debug_rows) },
        })
    }
}

impl Sandbox for FastBacktestSandbox {
    fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn mode(&self) -> SandboxMode {
        SandboxMode::FastBacktest
    }

    fn run_session(&mut self, slow_now_ms: u64, fast_now_ms: u64) -> Result<SessionOutput> {
        ensure_running(self.running, "fast backtest sandbox")?;
        self.coordinator.run_session(slow_now_ms, fast_now_ms)
    }

    fn submit_execution_plan(
        &mut self,
        plan: ExecutionPlan,
        normalized_data: Vec<NormalizedMarketData>,
        now_ms: u64,
    ) -> Result<FillResult> {
        ensure_running(self.running, "fast backtest sandbox")?;
        self.coordinator.submit_execution_plan(
            &plan,
            &normalized_data,
            now_ms,
            &trace_id("bt-plan", now_ms),
        )
    }

    fn on_market_data(
        &mut self,
        normalized_data: Vec<NormalizedMarketData>,
        now_ms: u64,
    ) -> Result<Vec<RuntimeEvent>> {
        ensure_running(self.running, "fast backtest sandbox")?;
        let trace_id = trace_id("bt-market", now_ms);
        let mut events = self
            .coordinator
            .on_market_data(&normalized_data, now_ms, &trace_id)?
            .events;
        events.push(self.coordinator.portfolio_update_event(
            "sandbox_market_data",
            &trace_id,
            now_ms,
        ));
        Ok(events)
    }

    fn snapshot(&self, now_ms: u64) -> SandboxSnapshot {
        snapshot_from(
            &self.coordinator,
            self.mode(),
            self.running,
            &self.test_mode,
            now_ms,
        )
    }

    fn swap_module_config(&mut self, module_key: &str, config: serde_json::Value) -> Result<String> {
        self.coordinator.swap_module_config(module_key, config)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_compiler::compile_runtime_protocol_config;
    use qrpc_core::{
        AgentConfig, DataKind, DataSourceConfig, Exchange, IntentConfig, IntentKind, MarketType,
        QuoteSnapshot, RiskConfig, RuntimeProtocolCoreConfig, SourceStatus, Symbol,
    };
    use std::collections::BTreeMap;

    #[test]
    fn real_time_sandbox_runs_session_when_started() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut sandbox = RealTimeSandbox::new(RuntimeCoordinator::new(compiled));
        sandbox.start().unwrap();

        let session = sandbox
            .run_session(1_700_000_000_000, 1_700_000_005_000)
            .unwrap();

        assert!(!session.slow_cycle.execution_plans.is_empty());
        assert!(sandbox.snapshot(1_700_000_005_000).is_running);
    }

    #[test]
    fn fast_backtest_sandbox_rejects_calls_before_start() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut sandbox = FastBacktestSandbox::new(RuntimeCoordinator::new(compiled));
        let err = sandbox.run_session(1, 2).unwrap_err();

        assert!(err.to_string().contains("未在运行"));
    }

    #[test]
    fn market_update_returns_portfolio_event() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut sandbox = RealTimeSandbox::new(RuntimeCoordinator::new(compiled));
        sandbox.start().unwrap();

        let events = sandbox
            .on_market_data(
                vec![NormalizedMarketData::Quote(QuoteSnapshot {
                    data_id: "binance_btc_quote".into(),
                    exchange: Exchange::Binance,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    best_bid: 49_999.0,
                    best_ask: 50_001.0,
                    bid_size: 5.0,
                    ask_size: 5.0,
                    mid_price: 50_000.0,
                    ts_ms: 10,
                    source_latency_ms: 0,
                    source_status: SourceStatus::Healthy,
                    data_quality: qrpc_core::DataQualitySnapshot::default(),
                })],
                10,
            )
            .unwrap();

        assert!(events
            .iter()
            .any(|event| event.event_type == qrpc_core::RuntimeEventType::PortfolioUpdated));
    }

    #[test]
    fn fast_backtest_replay_produces_equity_curve_and_sessions() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut sandbox =
            FastBacktestSandbox::with_mock_replay(compiled, 1_700_000_000_000).unwrap();
        sandbox.start().unwrap();

        let output = sandbox.run_backtest().unwrap();

        assert_eq!(output.mode, "historical_replay");
        assert!(!output.sessions.is_empty());
        assert_eq!(output.sessions.len(), output.equity_curve.len());
        assert_eq!(output.summary.step_count, output.sessions.len());
        assert!(output.ended_at_ms >= output.started_at_ms);
        assert!(output
            .equity_curve
            .windows(2)
            .all(|window| window[0].ts_ms <= window[1].ts_ms));
    }

    #[test]
    fn fast_backtest_latency_assumption_shifts_execution_clock() {
        let compiled_without_latency = compile_runtime_protocol_config(&sample_config()).unwrap();
        let compiled_with_latency = compile_runtime_protocol_config(&sample_config()).unwrap();
        let end_ms = 1_700_000_000_000;
        let latency_ms = 250;

        let mut without_latency =
            FastBacktestSandbox::with_mock_replay(compiled_without_latency, end_ms).unwrap();
        let mut with_latency =
            FastBacktestSandbox::with_mock_replay(compiled_with_latency, end_ms).unwrap();
        with_latency.set_latency_assumption_ms(latency_ms);

        without_latency.start().unwrap();
        with_latency.start().unwrap();

        let baseline = without_latency.run_backtest().unwrap();
        let delayed = with_latency.run_backtest().unwrap();

        assert_eq!(delayed.started_at_ms, baseline.started_at_ms + latency_ms);
        assert_eq!(delayed.ended_at_ms, baseline.ended_at_ms + latency_ms);
        assert_eq!(
            delayed.equity_curve.first().map(|point| point.ts_ms),
            baseline
                .equity_curve
                .first()
                .map(|point| point.ts_ms + latency_ms)
        );

        let baseline_fill = baseline
            .sessions
            .iter()
            .flat_map(|session| {
                session
                    .slow_cycle
                    .runtime_events
                    .iter()
                    .chain(session.fast_cycle.runtime_events.iter())
            })
            .find(|event| event.event_type == qrpc_core::RuntimeEventType::ExecutionFilled)
            .expect("mock replay should produce at least one execution fill");
        let delayed_fill = delayed
            .sessions
            .iter()
            .flat_map(|session| {
                session
                    .slow_cycle
                    .runtime_events
                    .iter()
                    .chain(session.fast_cycle.runtime_events.iter())
            })
            .find(|event| event.event_type == qrpc_core::RuntimeEventType::ExecutionFilled)
            .expect("mock replay should produce at least one execution fill");

        assert_eq!(delayed_fill.ts_ms, baseline_fill.ts_ms + latency_ms);
        assert_eq!(
            delayed_fill.payload["filled_at_ms"].as_u64(),
            baseline_fill.payload["filled_at_ms"]
                .as_u64()
                .map(|filled_at_ms| filled_at_ms + latency_ms)
        );
    }

    #[test]
    fn runtime_support_boundary_matches_current_beta_runtime_surface() {
        let boundary = runtime_support_boundary();

        assert_eq!(boundary.runtime_modes, &["paper"]);
        assert_eq!(boundary.execution_module_keys, &["builtin.execution.paper"]);
    }

    #[test]
    fn real_time_sandbox_snapshot_exposes_explicit_test_mode_configuration() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let test_mode = DeterministicTestMode::enabled_with_seed(7);
        let sandbox =
            RealTimeSandbox::with_test_mode(RuntimeCoordinator::new(compiled), test_mode.clone());

        let snapshot = sandbox.snapshot(1_700_000_000_000);

        assert_eq!(snapshot.deterministic_test_mode, test_mode);
        assert_eq!(sandbox.test_mode(), &test_mode);
    }

    #[test]
    fn fast_backtest_sandbox_keeps_test_mode_isolated_per_instance() {
        let compiled_a = compile_runtime_protocol_config(&sample_config()).unwrap();
        let compiled_b = compile_runtime_protocol_config(&sample_config()).unwrap();
        let replay_mode = DeterministicTestMode::replay_defaults(1_700_000_000_000, 11);
        let default_mode = DeterministicTestMode::default();

        let sandbox_a = FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode(
            compiled_a.core_ir,
            1_700_000_000_000,
            replay_mode.clone(),
        )
        .unwrap();
        let sandbox_b = FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode(
            compiled_b.core_ir,
            1_700_000_000_000,
            default_mode.clone(),
        )
        .unwrap();

        assert_eq!(sandbox_a.test_mode(), &replay_mode);
        assert_eq!(sandbox_b.test_mode(), &default_mode);
        assert_ne!(sandbox_a.test_mode(), sandbox_b.test_mode());
    }

    fn sample_config() -> RuntimeProtocolCoreConfig {
        RuntimeProtocolCoreConfig {
            data_sources: vec![
                DataSourceConfig {
                    data_id: "binance_btc_150d_1d".into(),
                    exchange: Exchange::Binance,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    kind: DataKind::KlineSeries,
                    days: Some(150),
                    interval: Some("1d".into()),
                    ping_enabled: false,
                    request_interval_ms: None,
                    enabled: true,
                },
                DataSourceConfig {
                    data_id: "binance_btc_quote".into(),
                    exchange: Exchange::Binance,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    kind: DataKind::Quote,
                    days: None,
                    interval: None,
                    ping_enabled: false,
                    request_interval_ms: None,
                    enabled: true,
                },
                DataSourceConfig {
                    data_id: "okx_btc_quote".into(),
                    exchange: Exchange::Okx,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    kind: DataKind::Quote,
                    days: None,
                    interval: None,
                    ping_enabled: false,
                    request_interval_ms: None,
                    enabled: true,
                },
            ],
            intents: vec![
                IntentConfig {
                    intent_id: "intent_long_buy".into(),
                    name: "Long Buy".into(),
                    kind: IntentKind::LongTermBuy,
                    input_data_ids: vec!["binance_btc_150d_1d".into()],
                    params: BTreeMap::new(),
                    enabled: true,
                },
                IntentConfig {
                    intent_id: "intent_long_sell".into(),
                    name: "Long Sell".into(),
                    kind: IntentKind::LongTermSell,
                    input_data_ids: vec!["binance_btc_150d_1d".into()],
                    params: BTreeMap::new(),
                    enabled: true,
                },
                IntentConfig {
                    intent_id: "intent_binance_quote".into(),
                    name: "Binance Quote".into(),
                    kind: IntentKind::QuoteObserve,
                    input_data_ids: vec!["binance_btc_quote".into()],
                    params: BTreeMap::new(),
                    enabled: true,
                },
                IntentConfig {
                    intent_id: "intent_okx_quote".into(),
                    name: "OKX Quote".into(),
                    kind: IntentKind::QuoteObserve,
                    input_data_ids: vec!["okx_btc_quote".into()],
                    params: BTreeMap::new(),
                    enabled: true,
                },
            ],
            agents: vec![
                AgentConfig {
                    agent_id: "agent_long_term".into(),
                    name: "Long Term Agent".into(),
                    input_intent_ids: vec!["intent_long_buy".into(), "intent_long_sell".into()],
                    rebalance_symbols: vec![],
                    rebalance_schedule: None,
                    rebalance_allocation_kind: None,
                    rebalance_rank_method: None,
                    rebalance_score_normalize: None,
                    rebalance_target_weights: vec![],
                    params: BTreeMap::new(),
                    enabled: true,
                },
                AgentConfig {
                    agent_id: "agent_arb".into(),
                    name: "Arb Agent".into(),
                    input_intent_ids: vec![
                        "intent_binance_quote".into(),
                        "intent_okx_quote".into(),
                    ],
                    rebalance_symbols: vec![],
                    rebalance_schedule: None,
                    rebalance_allocation_kind: None,
                    rebalance_rank_method: None,
                    rebalance_score_normalize: None,
                    rebalance_target_weights: vec![],
                    params: BTreeMap::new(),
                    enabled: true,
                },
            ],
            risks: vec![RiskConfig {
                risk_id: "risk_global".into(),
                name: "Global Risk".into(),
                observed_agent_ids: vec!["agent_long_term".into(), "agent_arb".into()],
                max_position_ratio: 0.2,
                max_single_weight: None,
                max_concentration_ratio: None,
                max_symbol_net_exposure_ratio: None,
                max_portfolio_net_exposure_ratio: None,
                max_turnover: None,
                min_trade_weight: None,
                max_new_positions_per_rebalance: None,
                max_total_leverage: 3.0,
                max_exchange_leverage: 3.0,
                min_action_interval_ms: 100,
                enabled: true,
            }],
            initial_cash_balance: 100_000.0,
            taker_fee_bps: 10.0,
            default_slippage_bps: 5.0,
            total_cost_buffer_bps: 20.0,
        }
    }
}
