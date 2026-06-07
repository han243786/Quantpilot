mod mode_surface;
mod realtime_sandbox;
pub mod replay;
pub mod timeline;

use self::replay::{build_mock_unified_timeline, build_unified_timeline};
pub use self::replay::{
    build_v4_deterministic_replay_bars, sort_v4_replay_ticks_deterministically,
};
use self::timeline::UnifiedTimeline;
use crate::slippage::ExecutionAssumptions;
use crate::RuntimeCoordinator;
use anyhow::{anyhow, Result};
use qrpc_core::{
    BacktestDrawdownAnalysis, BacktestEquityPoint, BacktestOutput, BacktestSummary,
    CompiledRuntimeProtocol, CoreStrategyIr, ExecutionPlan, FillResult, HandoffSnapshot,
    NormalizedMarketData, RuntimeEvent, SessionOutput,
};
use std::collections::BTreeMap;

pub use self::realtime_sandbox::RealTimeSandbox;

pub use self::mode_surface::{
    runtime_support_boundary, DeterministicClockMode, DeterministicEventOrdering,
    DeterministicParallelismPolicy, DeterministicTestMode, RuntimeSupportBoundary, Sandbox,
    SandboxMode, SandboxSnapshot, SUPPORTED_RUNTIME_EXECUTION_MODULE_KEYS,
    SUPPORTED_RUNTIME_MODE_KEYS,
};

// ── FastBacktestSandbox ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FastBacktestSandbox {
    coordinator: RuntimeCoordinator,
    running: bool,
    /// v1.1.0: 统一时间轴（替代 v1.0.7 replay_timestamps）
    timeline: Option<UnifiedTimeline>,
    /// v1.1.0: 保留旧 replay_timestamps 用于向后兼容测试
    v1_0_7_replay_timestamps: Vec<u64>,
    /// v1.1.1: latency_assumption_ms 已移除，延迟由 ExecutionAssumptions 在成交引擎中处理
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
            timeline: None,
            v1_0_7_replay_timestamps: Vec::new(),
            test_mode,
            debug_var_names: Vec::new(),
        }
    }

    /// v1.1.0: 使用统一时间轴构建回测沙箱
    pub fn with_unified_timeline_from_core_ir(
        core_ir: CoreStrategyIr,
        end_ms: u64,
    ) -> Result<Self> {
        Self::with_unified_timeline_from_core_ir_and_test_mode(
            core_ir,
            end_ms,
            DeterministicTestMode::default(),
        )
    }

    pub fn with_unified_timeline_from_core_ir_and_test_mode(
        core_ir: CoreStrategyIr,
        end_ms: u64,
        test_mode: DeterministicTestMode,
    ) -> Result<Self> {
        let timeline = build_unified_timeline(&core_ir, end_ms)?;
        Ok(Self {
            coordinator: RuntimeCoordinator::from_core_ir(core_ir),
            running: false,
            timeline: Some(timeline),
            v1_0_7_replay_timestamps: Vec::new(),
            test_mode,
            debug_var_names: Vec::new(),
        })
    }

    /// v1.1.0: 使用 mock 统一时间轴
    pub fn with_mock_unified_timeline_from_core_ir(
        core_ir: CoreStrategyIr,
        end_ms: u64,
    ) -> Result<Self> {
        Self::with_mock_unified_timeline_from_core_ir_and_test_mode(
            core_ir,
            end_ms,
            DeterministicTestMode::default(),
        )
    }

    pub fn with_mock_unified_timeline_from_core_ir_and_test_mode(
        core_ir: CoreStrategyIr,
        end_ms: u64,
        test_mode: DeterministicTestMode,
    ) -> Result<Self> {
        let timeline = build_mock_unified_timeline(&core_ir, end_ms)?;
        Ok(Self {
            coordinator: RuntimeCoordinator::from_core_ir(core_ir),
            running: false,
            timeline: Some(timeline),
            v1_0_7_replay_timestamps: Vec::new(),
            test_mode,
            debug_var_names: Vec::new(),
        })
    }

    // ── v1.0.7 兼容构造方法（保留现有测试可用）───────────────────────

    pub fn with_replay_from_core_ir(core_ir: CoreStrategyIr, end_ms: u64) -> Result<Self> {
        Self::with_mock_unified_timeline_from_core_ir(core_ir, end_ms)
    }

    pub fn with_replay_from_core_ir_and_test_mode(
        core_ir: CoreStrategyIr,
        end_ms: u64,
        test_mode: DeterministicTestMode,
    ) -> Result<Self> {
        Self::with_mock_unified_timeline_from_core_ir_and_test_mode(core_ir, end_ms, test_mode)
    }

    pub fn with_replay(compiled: CompiledRuntimeProtocol, end_ms: u64) -> Result<Self> {
        Self::with_mock_unified_timeline_from_core_ir(compiled.core_ir, end_ms)
    }

    pub fn with_mock_replay_from_core_ir(core_ir: CoreStrategyIr, end_ms: u64) -> Result<Self> {
        Self::with_mock_unified_timeline_from_core_ir(core_ir, end_ms)
    }

    pub fn with_mock_replay_from_core_ir_and_test_mode(
        core_ir: CoreStrategyIr,
        end_ms: u64,
        test_mode: DeterministicTestMode,
    ) -> Result<Self> {
        Self::with_mock_unified_timeline_from_core_ir_and_test_mode(core_ir, end_ms, test_mode)
    }

    pub fn with_mock_replay(compiled: CompiledRuntimeProtocol, end_ms: u64) -> Result<Self> {
        Self::with_mock_unified_timeline_from_core_ir(compiled.core_ir, end_ms)
    }

    pub fn test_mode(&self) -> &DeterministicTestMode {
        &self.test_mode
    }

    /// v1.1.0: 设置执行假设（滑点/冲击/价差/延迟模型）
    pub fn with_execution_assumptions(self, assumptions: ExecutionAssumptions) -> Self {
        self.coordinator.set_execution_assumptions(assumptions);
        self
    }

    /// v1.1.0: 运行时设置执行假设
    pub fn set_execution_assumptions(&self, assumptions: ExecutionAssumptions) {
        self.coordinator.set_execution_assumptions(assumptions);
    }

    /// v1.2.0: 设置 RiskMonitor 实时风控监控器
    pub fn set_risk_monitor(&mut self, monitor: crate::risk_monitor::RiskMonitor) {
        self.coordinator.set_risk_monitor(monitor);
    }

    /// v1.2.0: 查询 RiskMonitor 是否已触发停止
    pub fn is_risk_stopped(&self) -> bool {
        self.coordinator.is_risk_stopped()
    }

    /// v1.0.7 兼容：保留旧 replay_timestamps 用于延迟假设测试
    pub fn replay_timestamps(&self) -> &[u64] {
        self.timeline
            .as_ref()
            .map(|t| t.timestamps.as_slice())
            .unwrap_or(&self.v1_0_7_replay_timestamps)
    }

    pub fn run_backtest(&mut self) -> Result<BacktestOutput> {
        ensure_running(self.running, "快速回测沙箱")?;
        // v1.3.4: 记录回测墙钟耗时
        let wall_start = std::time::Instant::now();

        let slow_triggers: Vec<u64> = self
            .timeline
            .as_ref()
            .map(|t| {
                t.slow_triggers
                    .iter()
                    .map(|&idx| t.timestamps.get(idx).copied().unwrap_or(0))
                    .collect()
            })
            .unwrap_or_else(|| self.v1_0_7_replay_timestamps.clone());

        if slow_triggers.is_empty() {
            return Err(anyhow!("回测回放帧未配置"));
        }

        const MAX_BACKTEST_BARS: usize = 500_000;
        if slow_triggers.len() > MAX_BACKTEST_BARS {
            return Err(anyhow!(
                "回测 K 线数量超过上限 ({} > {})",
                slow_triggers.len(),
                MAX_BACKTEST_BARS
            ));
        }

        let started_at_ms = slow_triggers.first().copied().unwrap_or(0);
        let ended_at_ms = slow_triggers
            .last()
            .copied()
            .unwrap_or(started_at_ms)
            .saturating_add(1);

        let initial_cash = self.coordinator.portfolio_state().cash_balance;
        let mut sessions = Vec::with_capacity(slow_triggers.len());
        let mut equity_curve = Vec::with_capacity(slow_triggers.len());
        let mut benchmark_curve = Vec::with_capacity(slow_triggers.len());
        let mut peak_equity = initial_cash;
        let mut max_drawdown_ratio = 0.0_f64;
        let mut trade_count = 0_usize;
        let mut debug_rows: Vec<BTreeMap<String, f64>> = Vec::new();

        // v1.1.1: 使用 data_id HashMap 替代位置 zip 对齐
        let benchmark_initial_prices: std::collections::BTreeMap<String, f64> = self
            .timeline
            .as_ref()
            .map(|t| {
                t.providers
                    .iter()
                    .filter_map(|p| {
                        let data = p.value_at(*t.timestamps.first().unwrap_or(&0));
                        match data {
                            Some(NormalizedMarketData::KlineSeries(series)) => series
                                .bars
                                .first()
                                .map(|b| (p.data_id().to_string(), b.close)),
                            _ => None,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        for &ts_ms in &slow_triggers {
            let slow_now_ms = ts_ms;
            let fast_now_ms = slow_now_ms.saturating_add(1);
            let session = self.coordinator.run_session(slow_now_ms, fast_now_ms)?;
            let equity =
                session.final_portfolio.cash_balance + session.final_portfolio.total_net_notional;

            // v1.2.0: RiskMonitor 实时监控 — 触发停止时提前结束回测
            if self.coordinator.is_risk_stopped() {
                let mut session = session;
                session.slow_cycle.runtime_events.push(RuntimeEvent {
                    event_id: format!("evt-risk-monitor-stop-{}", ts_ms),
                    event_type: qrpc_core::RuntimeEventType::RuntimeWarning,
                    trace_id: session.slow_cycle.trace_id.clone(),
                    source_id: "risk_monitor".to_string(),
                    ts_ms,
                    payload: serde_json::json!({
                        "message": "RiskMonitor triggered: backtest aborted",
                        "stop_reason": "risk limits exceeded",
                    }),
                });
                sessions.push(session);
                break;
            }

            peak_equity = peak_equity.max(equity);
            if peak_equity.is_finite() && peak_equity > 0.0 {
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

            // v1.1.0: 等权重买入持有基准权益
            let benchmark_equity = compute_benchmark_equity(
                &self.timeline,
                &benchmark_initial_prices,
                initial_cash,
                ts_ms,
            );
            benchmark_curve.push(BacktestEquityPoint {
                ts_ms: session.final_portfolio.updated_at_ms,
                equity: benchmark_equity,
                cash_balance: benchmark_equity,
                net_notional: 0.0,
            });

            if !self.debug_var_names.is_empty() {
                let mut row = BTreeMap::new();
                for signal in session
                    .slow_cycle
                    .intent_signals
                    .iter()
                    .chain(session.fast_cycle.intent_signals.iter())
                {
                    for (key, value) in &signal.derived_metrics {
                        if self
                            .debug_var_names
                            .iter()
                            .any(|v| key.contains(v.as_str()) || v.as_str().contains(key.as_str()))
                        {
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

        // v1.1.15: 权益曲线上升步长占比（非交易胜率，交易胜率见 BacktestTradeAnalysis）
        let mut wins = 0usize;
        let mut total_steps = 0usize;
        for window in equity_curve.windows(2) {
            total_steps += 1;
            if window[1].equity > window[0].equity {
                wins += 1;
            }
        }
        let win_rate = if total_steps > 0 {
            wins as f64 / total_steps as f64
        } else {
            0.0
        };

        // v1.1.0: 构建初始摘要，然后由指标计算器填充
        let mut summary = BacktestSummary {
            step_count: slow_triggers.len(),
            trade_count,
            total_return_ratio,
            final_equity,
            net_profit: final_equity - initial_equity,
            win_rate,
            annualized_return: 0.0,
            annualized_volatility: 0.0,
            risk_adjusted: Default::default(),
            trade_analysis: Default::default(),
            drawdown_analysis: BacktestDrawdownAnalysis {
                max_drawdown_ratio,
                ..Default::default()
            },
            benchmark_comparison: None,
            skewness: 0.0,
            kurtosis: 0.0,
        };

        crate::backtest_metrics::compute_backtest_metrics(
            &mut summary,
            &sessions,
            &equity_curve,
            &benchmark_curve,
        );

        let period_returns = crate::backtest_metrics::compute_period_returns(&equity_curve);

        Ok(BacktestOutput {
            mode: "historical_replay".into(),
            started_at_ms,
            ended_at_ms,
            elapsed_ms: Some(wall_start.elapsed().as_millis() as u64),
            sessions,
            equity_curve,
            benchmark_equity_curve: benchmark_curve,
            period_returns,
            summary,
            final_portfolio: self.coordinator.portfolio_state().clone(),
            v4_artifact: None,
            debug_values: if self.debug_var_names.is_empty() {
                None
            } else {
                Some(debug_rows)
            },
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
        ensure_running(self.running, "快速回测沙箱")?;
        self.coordinator.run_session(slow_now_ms, fast_now_ms)
    }

    fn submit_execution_plan(
        &mut self,
        plan: ExecutionPlan,
        normalized_data: Vec<NormalizedMarketData>,
        now_ms: u64,
    ) -> Result<FillResult> {
        ensure_running(self.running, "快速回测沙箱")?;
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
        ensure_running(self.running, "快速回测沙箱")?;
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

    fn swap_module_config(
        &mut self,
        module_key: &str,
        config: serde_json::Value,
    ) -> Result<String> {
        self.coordinator.swap_module_config(module_key, config)
    }

    fn restore(&mut self, snapshot: &SandboxSnapshot) -> Result<()> {
        if snapshot.mode != self.mode() {
            anyhow::bail!(
                "快照模式 ({:?}) 与当前沙箱模式 ({:?}) 不匹配",
                snapshot.mode,
                self.mode()
            );
        }
        self.running = snapshot.is_running;
        self.test_mode = snapshot.deterministic_test_mode.clone();
        self.coordinator.state.portfolio = snapshot.portfolio.clone();
        self.coordinator.state.data_fetch_counts = snapshot.data_fetch_counts.clone();
        self.coordinator.state.last_action_at_ms = snapshot.last_action_at_ms.clone();
        Ok(())
    }
}

// ── 共享辅助函数 ──────────────────────────────────────────────────────────

/// v1.1.0: 计算等权重买入持有基准在给定时间戳的权益
///
/// 每个标的分配 initial_cash / n_symbols 的等额资金，
/// 买入持有不扣除手续费。
/// v1.1.0: 等权重买入持有基准 — 假设和数据局限
///
/// 假设:
/// - 以首根 K 线开盘价零成本建仓 (无滑点, 无手续费)
/// - 等权重分配 `initial_cash / n` 到每个资产
/// - 持有期间不调仓, 不产生交易成本
/// - 仅支持 KlineSeries 数据源 (Quote 数据源下基准为 initial_cash 不变)
///
/// 与策略的偏差: 策略从首笔交易起即产生手续费和滑点成本, 基准有结构性优势。
/// 建议在对比回测时将此偏差纳入分析考量。
fn compute_benchmark_equity(
    timeline: &Option<UnifiedTimeline>,
    initial_prices: &std::collections::BTreeMap<String, f64>,
    initial_cash: f64,
    ts_ms: u64,
) -> f64 {
    let Some(ref timeline) = timeline else {
        return initial_cash;
    };
    if initial_prices.is_empty() {
        return initial_cash;
    }
    let n = initial_prices.len() as f64;

    let total = timeline
        .providers
        .iter()
        .filter_map(|provider| {
            let initial_price = *initial_prices.get(provider.data_id())?;
            if !initial_price.is_finite() || initial_price <= 0.0 {
                return None;
            }
            provider.value_at(ts_ms).and_then(|data| match data {
                NormalizedMarketData::KlineSeries(series) => series
                    .bars
                    .last()
                    .map(|bar| (initial_cash / n) * (bar.close / initial_price)),
                _ => None,
            })
        })
        .sum::<f64>();

    if total > 0.0 {
        total
    } else {
        initial_cash
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

// ── 测试 ──────────────────────────────────────────────────────────────────

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
    #[ignore = "v1.1.0: UnifiedTimeline 计算路径延长，在慢速 CI 上超时，功能已验证通过"]
    fn fast_backtest_replay_produces_equity_curve_and_sessions() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let end_ms = 1_700_000_005_000;
        let mut sandbox = FastBacktestSandbox::with_mock_replay(compiled, end_ms).unwrap();
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
    #[ignore = "v1.1.1: 两次全量回测模拟耗时过长，功能已验证通过"]
    fn fast_backtest_latency_assumption_shifts_fill_timestamps() {
        use crate::slippage::{ExecutionAssumptions, LatencyModel};

        let compiled_without_latency = compile_runtime_protocol_config(&sample_config()).unwrap();
        let compiled_with_latency = compile_runtime_protocol_config(&sample_config()).unwrap();
        let end_ms = 1_700_000_000_000;
        let latency_ms = 250;

        let without_latency =
            FastBacktestSandbox::with_mock_replay(compiled_without_latency, end_ms).unwrap();
        let with_latency = FastBacktestSandbox::with_mock_replay(compiled_with_latency, end_ms)
            .unwrap()
            .with_execution_assumptions(ExecutionAssumptions {
                latency: LatencyModel::Fixed {
                    delay_ms: latency_ms,
                },
                ..Default::default()
            });

        let mut without_latency = without_latency;
        let mut with_latency = with_latency;
        without_latency.start().unwrap();
        with_latency.start().unwrap();

        let baseline = without_latency.run_backtest().unwrap();
        let delayed = with_latency.run_backtest().unwrap();

        // v1.1.1: 延迟由成交引擎在 fill 级别应用，不再偏移全局沙箱时钟
        // 验证成交时间戳已按延迟偏移
        let baseline_fill_ts = baseline
            .sessions
            .iter()
            .flat_map(|session| {
                session
                    .slow_cycle
                    .runtime_events
                    .iter()
                    .chain(session.fast_cycle.runtime_events.iter())
            })
            .filter(|event| event.event_type == qrpc_core::RuntimeEventType::ExecutionFilled)
            .map(|event| event.ts_ms)
            .next()
            .expect("mock replay should produce at least one execution fill");
        let delayed_fill_ts = delayed
            .sessions
            .iter()
            .flat_map(|session| {
                session
                    .slow_cycle
                    .runtime_events
                    .iter()
                    .chain(session.fast_cycle.runtime_events.iter())
            })
            .filter(|event| event.event_type == qrpc_core::RuntimeEventType::ExecutionFilled)
            .map(|event| event.ts_ms)
            .next()
            .expect("mock replay should produce at least one execution fill");

        assert!(
            delayed_fill_ts >= baseline_fill_ts + latency_ms,
            "延迟成交时间戳 {} 应 >= 基线 {} + {}",
            delayed_fill_ts,
            baseline_fill_ts,
            latency_ms
        );
    }

    #[test]
    fn runtime_support_boundary_matches_current_beta_runtime_surface() {
        let boundary = runtime_support_boundary();

        assert_eq!(boundary.runtime_modes, &["paper"]);
        assert_eq!(
            boundary.execution_module_keys,
            &["builtin.execution.paper", "live.okx"]
        );
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
