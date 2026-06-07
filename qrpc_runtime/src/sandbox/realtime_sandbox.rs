use super::*;

// ── RealTimeSandbox ──────────────────────────────────────────────────────

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
        ensure_running(self.running, "实时沙箱")?;
        self.coordinator.run_session(slow_now_ms, fast_now_ms)
    }

    fn submit_execution_plan(
        &mut self,
        plan: ExecutionPlan,
        normalized_data: Vec<NormalizedMarketData>,
        now_ms: u64,
    ) -> Result<FillResult> {
        ensure_running(self.running, "实时沙箱")?;
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
        ensure_running(self.running, "实时沙箱")?;
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

    fn swap_module_config(
        &mut self,
        module_key: &str,
        config: serde_json::Value,
    ) -> Result<String> {
        self.coordinator.swap_module_config(module_key, config)
    }

    fn handoff(&mut self, snapshot: &HandoffSnapshot) -> Result<()> {
        snapshot
            .validate_completeness()
            .map_err(|errs| anyhow::anyhow!("热接管快照校验失败: {:?}", errs))?;
        // v2.1.0: 实际恢复 portfolio 状态
        self.coordinator.state.portfolio.cash_balance = snapshot.cash_balance;
        self.coordinator.state.portfolio.available_cash_balance = snapshot.available_cash_balance;
        self.coordinator.state.portfolio.frozen_cash_balance = snapshot.frozen_cash_balance;
        // 恢复持仓: HandoffSnapshot.positions 是 BTreeMap<Symbol, f64>
        self.coordinator
            .state
            .portfolio
            .positions
            .retain(|p| !snapshot.positions.contains_key(&p.symbol));
        for (symbol, qty) in &snapshot.positions {
            let found = self
                .coordinator
                .state
                .portfolio
                .positions
                .iter_mut()
                .find(|p| p.symbol == *symbol);
            if let Some(pos) = found {
                pos.net_qty = *qty;
            } else {
                self.coordinator
                    .state
                    .portfolio
                    .positions
                    .push(qrpc_core::Position {
                        exchange: qrpc_core::Exchange::Binance,
                        symbol: symbol.clone(),
                        net_qty: *qty,
                        frozen_qty: 0.0,
                        avg_entry_price: 0.0,
                        mark_price: 0.0,
                        unrealized_pnl: 0.0,
                        realized_pnl: 0.0,
                    });
            }
        }
        // 清除未结订单
        self.coordinator.state.portfolio.open_orders.clear();
        self.coordinator.state.data_fetch_counts.clear();
        self.coordinator.state.last_action_at_ms.clear();
        Ok(())
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

impl RealTimeSandbox {
    /// v1.2.0: 设置 RiskMonitor 实时风控监控器
    pub fn set_risk_monitor(&mut self, monitor: crate::risk_monitor::RiskMonitor) {
        self.coordinator.set_risk_monitor(monitor);
    }

    /// v1.2.0: 查询 RiskMonitor 是否已触发停止
    pub fn is_risk_stopped(&self) -> bool {
        self.coordinator.is_risk_stopped()
    }
}
