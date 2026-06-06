use super::{portfolio_equity_estimate, RuntimeCoordinator};
use anyhow::Result;
use qrpc_core::{IntentKind, RuntimeCycleOutput, SessionOutput};

impl RuntimeCoordinator {
    pub fn run_session(&mut self, slow_now_ms: u64, fast_now_ms: u64) -> Result<SessionOutput> {
        // 在 epoch barrier 应用待处理的模块配置
        let activated = self.apply_pending_module_configs();
        if !activated.is_empty() {
            // 已激活新配置，后续 cycle 使用更新后的模块
        }

        let slow_cycle = self.run_slow_cycle(slow_now_ms)?;
        let fast_cycle = self.run_fast_cycle(fast_now_ms)?;

        // v1.2.0: RiskMonitor 连续风控 — 每个 session 后检查
        if let Some(ref mut monitor) = self.risk_monitor {
            let equity = portfolio_equity_estimate(&self.state.portfolio);
            if monitor.check(equity).stop {
                self.risk_stopped = true;
            }
        }

        Ok(SessionOutput {
            slow_cycle,
            fast_cycle,
            final_portfolio: self.state.portfolio.clone(),
            data_fetch_counts: self.state.data_fetch_counts.clone(),
        })
    }

    pub fn run_slow_cycle(&mut self, now_ms: u64) -> Result<RuntimeCycleOutput> {
        self.run_cycle(
            "slow",
            now_ms,
            &[
                IntentKind::LongTermBuy,
                IntentKind::LongTermSell,
                IntentKind::Rsi,
                IntentKind::Macd,
                IntentKind::Momentum,
                IntentKind::ZScore,
            ],
        )
    }

    pub fn run_fast_cycle(&mut self, now_ms: u64) -> Result<RuntimeCycleOutput> {
        self.run_cycle("fast", now_ms, &[IntentKind::QuoteObserve])
    }
    fn run_cycle(
        &mut self,
        cycle_name: &str,
        now_ms: u64,
        intent_kinds: &[IntentKind],
    ) -> Result<RuntimeCycleOutput> {
        let trace_id = format!("trace-{cycle_name}-{now_ms}");
        let mut runtime_events = Vec::new();
        let normalized_data =
            self.collect_normalized_data(cycle_name, now_ms, &trace_id, &mut runtime_events)?;
        let resting_fills =
            self.process_open_orders(&normalized_data, now_ms, &trace_id, &mut runtime_events)?;
        self.refresh_portfolio_state(&normalized_data, now_ms);
        let intent_signals = self.evaluate_intents(
            intent_kinds,
            &normalized_data,
            now_ms,
            &trace_id,
            &mut runtime_events,
        );
        let raw_agent_decisions = self.evaluate_agents(
            cycle_name,
            &intent_signals,
            now_ms,
            &trace_id,
            &mut runtime_events,
        );
        // 合并引擎：多策略场景下统一汇聚 Agent 决策
        let (merged_decisions, merge_record) = self.merge_agent_decisions(
            cycle_name,
            &raw_agent_decisions,
            &intent_signals,
            &trace_id,
            &mut runtime_events,
        );
        if let Some(record) = merge_record {
            // v2.1.0: 裁剪超出上限的旧记录，防止无界增长
            const MAX_MERGE_RECORDS: usize = 500;
            let len = self.merge.records.len();
            if len >= MAX_MERGE_RECORDS {
                self.merge.records.drain(0..len - MAX_MERGE_RECORDS + 1);
            }
            self.merge.records.push(record);
        }

        let risk_decisions =
            self.evaluate_risks(&merged_decisions, now_ms, &trace_id, &mut runtime_events);
        let execution_plans = self.plan_execution(
            &risk_decisions,
            &normalized_data,
            now_ms,
            &trace_id,
            &mut runtime_events,
        );
        let mut fill_reports = resting_fills;
        fill_reports.extend(self.execute_plans(
            &execution_plans,
            &normalized_data,
            now_ms,
            &trace_id,
            &mut runtime_events,
        )?);
        self.refresh_portfolio_state(&normalized_data, now_ms);

        runtime_events.push(self.portfolio_update_event("portfolio", &trace_id, now_ms));

        Ok(RuntimeCycleOutput {
            cycle_name: cycle_name.to_string(),
            trace_id,
            normalized_data,
            intent_signals,
            agent_decisions: merged_decisions,
            risk_decisions,
            execution_plans,
            fill_reports,
            portfolio_state: self.state.portfolio.clone(),
            runtime_events,
            data_fetch_counts: self.state.data_fetch_counts.clone(),
        })
    }
}
