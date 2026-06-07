use super::replay::{build_mock_unified_timeline, build_unified_timeline};
use super::timeline::UnifiedTimeline;
use super::{
    ensure_running, snapshot_from, trace_id, DeterministicTestMode, Sandbox, SandboxMode,
    SandboxSnapshot,
};
use crate::slippage::ExecutionAssumptions;
use crate::RuntimeCoordinator;
use anyhow::{anyhow, Result};
use qrpc_core::{
    BacktestDrawdownAnalysis, BacktestEquityPoint, BacktestOutput, BacktestSummary,
    CompiledRuntimeProtocol, CoreStrategyIr, ExecutionPlan, FillResult, NormalizedMarketData,
    RuntimeEvent, SessionOutput,
};
use std::collections::BTreeMap;

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

// Fast-backtest benchmark equity projection.
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
