mod agent_module;
mod core_ir_evaluator;
mod data_module;
mod execution_module;
mod fill_engine;
mod intent_module;
mod plugin_runtime_registry;
mod risk_checker;
mod sandbox;

use anyhow::Result;
use qrpc_core::{
    AgentDecision, CompiledRuntimeProtocol, CoreStrategyIr, Exchange, ExchangeExposure,
    ExecutionPlan, FillReport, IntentKind, IntentSignal, NormalizedMarketData, PortfolioState,
    RiskDecision, RuntimeCycleOutput, RuntimeEvent, RuntimeEventType, SessionOutput, Symbol,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub use agent_module::{
    AgentEvaluationOutput, AgentEvaluationRequest, AgentModuleProvider, BuiltinAgentModule,
};
pub use core_ir_evaluator::{
    evaluate_indicator_signal, CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError,
};
pub use data_module::{
    BuiltinDataModule, DataCollectionOutput, DataCollectionRequest, DataModuleProvider,
};
pub use execution_module::{
    BuiltinExecutionModule, ExecutionModuleProvider, ExecutionPlanningOutput,
    ExecutionPlanningRequest,
};
pub use intent_module::{
    BuiltinIntentModule, IntentEvaluationOutput, IntentEvaluationRequest, IntentModuleProvider,
};
pub use plugin_runtime_registry::{
    PluginLifecycleState, RuntimePluginLifecycle, RuntimePluginRegistry,
};
pub use risk_checker::{RiskCheckOutput, RiskCheckRequest, RiskChecker, RiskCheckerProvider};
pub use sandbox::{
    runtime_support_boundary, DeterministicClockMode, DeterministicEventOrdering,
    DeterministicParallelismPolicy, DeterministicTestMode, FastBacktestSandbox, RealTimeSandbox,
    RuntimeSupportBoundary, Sandbox, SandboxMode, SandboxSnapshot,
    SUPPORTED_RUNTIME_EXECUTION_MODULE_KEYS, SUPPORTED_RUNTIME_MODE_KEYS,
};

#[derive(Clone)]
pub struct RuntimeCoordinator {
    core_ir: CoreStrategyIr,
    portfolio: PortfolioState,
    data_fetch_counts: BTreeMap<String, u32>,
    last_action_at_ms: BTreeMap<String, u64>,
    last_rebalance_at_ms: BTreeMap<String, u64>,
    data_module: Arc<dyn DataModuleProvider>,
    intent_module: Arc<dyn IntentModuleProvider>,
    agent_module: Arc<dyn AgentModuleProvider>,
    execution_module: Arc<Mutex<dyn ExecutionModuleProvider>>,
    risk_checker: Arc<dyn RiskCheckerProvider>,
}

impl std::fmt::Debug for RuntimeCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeCoordinator")
            .field("core_ir", &self.core_ir)
            .field("portfolio", &self.portfolio)
            .field("data_fetch_counts", &self.data_fetch_counts)
            .field("last_action_at_ms", &self.last_action_at_ms)
            .field("last_rebalance_at_ms", &self.last_rebalance_at_ms)
            .field("data_provider_key", &self.data_module.provider_key())
            .field("intent_provider_key", &self.intent_module.provider_key())
            .field("agent_provider_key", &self.agent_module.provider_key())
            .field(
                "execution_provider_key",
                &self.execution_module_provider_key(),
            )
            .field("risk_provider_key", &self.risk_checker.provider_key())
            .finish()
    }
}

impl RuntimeCoordinator {
    pub fn from_core_ir(core_ir: CoreStrategyIr) -> Self {
        Self::with_modules_from_core_ir(
            core_ir,
            BuiltinDataModule::default(),
            BuiltinIntentModule::default(),
            BuiltinAgentModule::default(),
            RiskChecker::default(),
            BuiltinExecutionModule::default(),
        )
    }

    pub fn new(compiled: CompiledRuntimeProtocol) -> Self {
        Self::from_core_ir(compiled.core_ir)
    }

    pub fn with_modules_from_core_ir<D, I, A, R, E>(
        core_ir: CoreStrategyIr,
        data_module: D,
        intent_module: I,
        agent_module: A,
        risk_checker: R,
        execution_module: E,
    ) -> Self
    where
        D: DataModuleProvider + 'static,
        I: IntentModuleProvider + 'static,
        A: AgentModuleProvider + 'static,
        R: RiskCheckerProvider + 'static,
        E: ExecutionModuleProvider + 'static,
    {
        Self::with_module_providers_from_core_ir(
            core_ir,
            Arc::new(data_module),
            Arc::new(intent_module),
            Arc::new(agent_module),
            Arc::new(risk_checker),
            Arc::new(Mutex::new(execution_module)),
        )
    }

    pub fn with_modules<D, I, A, R, E>(
        compiled: CompiledRuntimeProtocol,
        data_module: D,
        intent_module: I,
        agent_module: A,
        risk_checker: R,
        execution_module: E,
    ) -> Self
    where
        D: DataModuleProvider + 'static,
        I: IntentModuleProvider + 'static,
        A: AgentModuleProvider + 'static,
        R: RiskCheckerProvider + 'static,
        E: ExecutionModuleProvider + 'static,
    {
        Self::with_modules_from_core_ir(
            compiled.core_ir,
            data_module,
            intent_module,
            agent_module,
            risk_checker,
            execution_module,
        )
    }

    pub fn with_data_module_from_core_ir<P>(core_ir: CoreStrategyIr, data_module: P) -> Self
    where
        P: DataModuleProvider + 'static,
    {
        Self::with_modules_from_core_ir(
            core_ir,
            data_module,
            BuiltinIntentModule::default(),
            BuiltinAgentModule::default(),
            RiskChecker::default(),
            BuiltinExecutionModule::default(),
        )
    }

    pub fn with_data_module<P>(compiled: CompiledRuntimeProtocol, data_module: P) -> Self
    where
        P: DataModuleProvider + 'static,
    {
        Self::with_data_module_from_core_ir(compiled.core_ir, data_module)
    }

    pub fn with_intent_module_from_core_ir<P>(core_ir: CoreStrategyIr, intent_module: P) -> Self
    where
        P: IntentModuleProvider + 'static,
    {
        Self::with_modules_from_core_ir(
            core_ir,
            BuiltinDataModule::default(),
            intent_module,
            BuiltinAgentModule::default(),
            RiskChecker::default(),
            BuiltinExecutionModule::default(),
        )
    }

    pub fn with_intent_module<P>(compiled: CompiledRuntimeProtocol, intent_module: P) -> Self
    where
        P: IntentModuleProvider + 'static,
    {
        Self::with_intent_module_from_core_ir(compiled.core_ir, intent_module)
    }

    pub fn with_agent_module_from_core_ir<P>(core_ir: CoreStrategyIr, agent_module: P) -> Self
    where
        P: AgentModuleProvider + 'static,
    {
        Self::with_modules_from_core_ir(
            core_ir,
            BuiltinDataModule::default(),
            BuiltinIntentModule::default(),
            agent_module,
            RiskChecker::default(),
            BuiltinExecutionModule::default(),
        )
    }

    pub fn with_agent_module<P>(compiled: CompiledRuntimeProtocol, agent_module: P) -> Self
    where
        P: AgentModuleProvider + 'static,
    {
        Self::with_agent_module_from_core_ir(compiled.core_ir, agent_module)
    }

    pub fn with_risk_checker_from_core_ir<P>(core_ir: CoreStrategyIr, risk_checker: P) -> Self
    where
        P: RiskCheckerProvider + 'static,
    {
        Self::with_modules_from_core_ir(
            core_ir,
            BuiltinDataModule::default(),
            BuiltinIntentModule::default(),
            BuiltinAgentModule::default(),
            risk_checker,
            BuiltinExecutionModule::default(),
        )
    }

    pub fn with_risk_checker<P>(compiled: CompiledRuntimeProtocol, risk_checker: P) -> Self
    where
        P: RiskCheckerProvider + 'static,
    {
        Self::with_risk_checker_from_core_ir(compiled.core_ir, risk_checker)
    }

    pub fn with_execution_module_from_core_ir<P>(
        core_ir: CoreStrategyIr,
        execution_module: P,
    ) -> Self
    where
        P: ExecutionModuleProvider + 'static,
    {
        Self::with_modules_from_core_ir(
            core_ir,
            BuiltinDataModule::default(),
            BuiltinIntentModule::default(),
            BuiltinAgentModule::default(),
            RiskChecker::default(),
            execution_module,
        )
    }

    pub fn with_execution_module<P>(compiled: CompiledRuntimeProtocol, execution_module: P) -> Self
    where
        P: ExecutionModuleProvider + 'static,
    {
        Self::with_execution_module_from_core_ir(compiled.core_ir, execution_module)
    }

    pub fn with_module_providers_from_core_ir(
        core_ir: CoreStrategyIr,
        data_module: Arc<dyn DataModuleProvider>,
        intent_module: Arc<dyn IntentModuleProvider>,
        agent_module: Arc<dyn AgentModuleProvider>,
        risk_checker: Arc<dyn RiskCheckerProvider>,
        execution_module: Arc<Mutex<dyn ExecutionModuleProvider>>,
    ) -> Self {
        let initial_cash_balance = core_ir
            .execution
            .params
            .get("initial_cash_balance")
            .and_then(|value| value.as_f64())
            .unwrap_or(100_000.0);
        let portfolio = PortfolioState::new(initial_cash_balance, 0);
        Self {
            core_ir,
            portfolio,
            data_fetch_counts: BTreeMap::new(),
            last_action_at_ms: BTreeMap::new(),
            last_rebalance_at_ms: BTreeMap::new(),
            data_module,
            intent_module,
            agent_module,
            execution_module,
            risk_checker,
        }
    }

    pub fn with_module_providers(
        compiled: CompiledRuntimeProtocol,
        data_module: Arc<dyn DataModuleProvider>,
        intent_module: Arc<dyn IntentModuleProvider>,
        agent_module: Arc<dyn AgentModuleProvider>,
        risk_checker: Arc<dyn RiskCheckerProvider>,
        execution_module: Arc<Mutex<dyn ExecutionModuleProvider>>,
    ) -> Self {
        Self::with_module_providers_from_core_ir(
            compiled.core_ir,
            data_module,
            intent_module,
            agent_module,
            risk_checker,
            execution_module,
        )
    }

    pub fn run_session(&mut self, slow_now_ms: u64, fast_now_ms: u64) -> Result<SessionOutput> {
        let slow_cycle = self.run_slow_cycle(slow_now_ms)?;
        let fast_cycle = self.run_fast_cycle(fast_now_ms)?;

        Ok(SessionOutput {
            slow_cycle,
            fast_cycle,
            final_portfolio: self.portfolio.clone(),
            data_fetch_counts: self.data_fetch_counts.clone(),
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

    pub fn submit_execution_plan(
        &mut self,
        plan: &ExecutionPlan,
        normalized_data: &[NormalizedMarketData],
        now_ms: u64,
        trace_id: &str,
    ) -> Result<qrpc_core::FillResult> {
        let result = self
            .execution_module
            .lock()
            .expect("execution module lock should not be poisoned")
            .submit_plan(plan, normalized_data, &mut self.portfolio, now_ms, trace_id);
        self.refresh_portfolio_state(normalized_data, now_ms);
        Ok(result)
    }

    pub fn on_market_data(
        &mut self,
        normalized_data: &[NormalizedMarketData],
        now_ms: u64,
        trace_id: &str,
    ) -> Result<qrpc_core::FillResult> {
        let result = self
            .execution_module
            .lock()
            .expect("execution module lock should not be poisoned")
            .on_market_update(normalized_data, &mut self.portfolio, now_ms, trace_id);
        self.refresh_portfolio_state(normalized_data, now_ms);
        Ok(result)
    }

    pub fn portfolio_state(&self) -> &PortfolioState {
        &self.portfolio
    }

    pub fn data_fetch_counts(&self) -> &BTreeMap<String, u32> {
        &self.data_fetch_counts
    }

    pub fn last_action_at_ms(&self) -> &BTreeMap<String, u64> {
        &self.last_action_at_ms
    }

    pub fn last_rebalance_at_ms(&self) -> &BTreeMap<String, u64> {
        &self.last_rebalance_at_ms
    }

    pub fn data_module(&self) -> &(dyn DataModuleProvider + Send + Sync) {
        self.data_module.as_ref()
    }

    pub fn intent_module(&self) -> &(dyn IntentModuleProvider + Send + Sync) {
        self.intent_module.as_ref()
    }

    pub fn agent_module(&self) -> &(dyn AgentModuleProvider + Send + Sync) {
        self.agent_module.as_ref()
    }

    pub fn risk_checker(&self) -> &(dyn RiskCheckerProvider + Send + Sync) {
        self.risk_checker.as_ref()
    }

    pub fn execution_module_provider_key(&self) -> &'static str {
        self.execution_module
            .lock()
            .expect("execution module lock should not be poisoned")
            .provider_key()
    }

    pub fn portfolio_update_event(
        &self,
        source_id: &str,
        trace_id: &str,
        now_ms: u64,
    ) -> RuntimeEvent {
        let equity_estimate = portfolio_equity_estimate(&self.portfolio);
        let open_orders = self
            .portfolio
            .open_orders
            .iter()
            .map(|order| {
                json!({
                    "order_id": order.order_id,
                    "side": format!("{:?}", order.side),
                    "remaining_qty": order.remaining_qty,
                    "limit_price": order.limit_price,
                    "reserved_cash": order.reserved_cash,
                    "reserved_qty": order.reserved_qty,
                })
            })
            .collect::<Vec<_>>();
        RuntimeEvent {
            event_id: format!("evt-portfolio-{source_id}-{now_ms}"),
            event_type: RuntimeEventType::PortfolioUpdated,
            trace_id: trace_id.to_string(),
            source_id: source_id.to_string(),
            ts_ms: now_ms,
            payload: json!({
                "cash_balance": self.portfolio.cash_balance,
                "available_cash_balance": self.portfolio.available_cash_balance,
                "frozen_cash_balance": self.portfolio.frozen_cash_balance,
                "total_gross_notional": self.portfolio.total_gross_notional,
                "total_net_notional": self.portfolio.total_net_notional,
                "total_leverage": self.portfolio.total_leverage,
                "equity_estimate": equity_estimate,
                "positions": self.portfolio.positions.len(),
                "open_order_count": self.portfolio.open_orders.len(),
                "open_orders": open_orders,
            }),
        }
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
        let intent_signals = self.evaluate_intents(
            intent_kinds,
            &normalized_data,
            now_ms,
            &trace_id,
            &mut runtime_events,
        );
        let agent_decisions = self.evaluate_agents(
            cycle_name,
            &intent_signals,
            now_ms,
            &trace_id,
            &mut runtime_events,
        );
        let risk_decisions =
            self.evaluate_risks(&agent_decisions, now_ms, &trace_id, &mut runtime_events);
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
            agent_decisions,
            risk_decisions,
            execution_plans,
            fill_reports,
            portfolio_state: self.portfolio.clone(),
            runtime_events,
            data_fetch_counts: self.data_fetch_counts.clone(),
        })
    }

    fn collect_normalized_data(
        &mut self,
        cycle_name: &str,
        now_ms: u64,
        trace_id: &str,
        runtime_events: &mut Vec<RuntimeEvent>,
    ) -> Result<Vec<NormalizedMarketData>> {
        let output = self.data_module.collect(DataCollectionRequest {
            cycle_name,
            core_ir: &self.core_ir,
            data_fetch_counts: &mut self.data_fetch_counts,
            now_ms,
            trace_id,
        })?;
        runtime_events.extend(output.events);
        Ok(output.normalized_data)
    }

    fn evaluate_intents(
        &self,
        intent_kinds: &[IntentKind],
        normalized_data: &[NormalizedMarketData],
        now_ms: u64,
        trace_id: &str,
        runtime_events: &mut Vec<RuntimeEvent>,
    ) -> Vec<IntentSignal> {
        let output = self
            .intent_module
            .evaluate_intents(IntentEvaluationRequest {
                intent_kinds,
                core_ir: &self.core_ir,
                normalized_data,
                now_ms,
                trace_id,
            });
        runtime_events.extend(output.events);
        output.signals
    }

    fn evaluate_agents(
        &mut self,
        cycle_name: &str,
        signals: &[IntentSignal],
        now_ms: u64,
        trace_id: &str,
        runtime_events: &mut Vec<RuntimeEvent>,
    ) -> Vec<AgentDecision> {
        let output = self.agent_module.evaluate_agents(AgentEvaluationRequest {
            cycle_name,
            signals,
            core_ir: &self.core_ir,
            portfolio: &self.portfolio,
            last_rebalance_at_ms: &self.last_rebalance_at_ms,
            now_ms,
            trace_id,
        });
        for agent_id in &output.evaluated_rebalance_agent_ids {
            self.last_rebalance_at_ms.insert(agent_id.clone(), now_ms);
        }
        runtime_events.extend(output.events);
        output.decisions
    }

    fn evaluate_risks(
        &mut self,
        decisions: &[AgentDecision],
        now_ms: u64,
        trace_id: &str,
        runtime_events: &mut Vec<RuntimeEvent>,
    ) -> Vec<RiskDecision> {
        let output = self
            .risk_checker
            .evaluate(RiskCheckRequest {
                decisions,
                core_ir: &self.core_ir,
                portfolio: &self.portfolio,
                last_action_at_ms: &self.last_action_at_ms,
                now_ms,
                trace_id,
            })
            .expect("risk checker evaluation should not fail");

        for agent_id in &output.approved_agent_ids {
            self.last_action_at_ms.insert(agent_id.clone(), now_ms);
        }
        runtime_events.extend(output.events);
        output.decisions
    }

    fn plan_execution(
        &self,
        risk_decisions: &[RiskDecision],
        normalized_data: &[NormalizedMarketData],
        now_ms: u64,
        trace_id: &str,
        runtime_events: &mut Vec<RuntimeEvent>,
    ) -> Vec<ExecutionPlan> {
        let output = self
            .execution_module
            .lock()
            .expect("execution module lock should not be poisoned")
            .plan_execution(ExecutionPlanningRequest {
                risk_decisions,
                core_ir: &self.core_ir,
                normalized_data,
                portfolio: &self.portfolio,
                now_ms,
                trace_id,
            });
        runtime_events.extend(output.events);
        output.plans
    }

    fn execute_plans(
        &mut self,
        plans: &[ExecutionPlan],
        normalized_data: &[NormalizedMarketData],
        now_ms: u64,
        trace_id: &str,
        runtime_events: &mut Vec<RuntimeEvent>,
    ) -> Result<Vec<FillReport>> {
        let mut fills = Vec::new();

        for plan in plans {
            let result = self
                .execution_module
                .lock()
                .expect("execution module lock should not be poisoned")
                .submit_plan(plan, normalized_data, &mut self.portfolio, now_ms, trace_id);
            runtime_events.extend(result.events.clone());
            fills.extend(result.fills);
        }

        Ok(fills)
    }
    fn process_open_orders(
        &mut self,
        normalized_data: &[NormalizedMarketData],
        now_ms: u64,
        trace_id: &str,
        runtime_events: &mut Vec<RuntimeEvent>,
    ) -> Result<Vec<FillReport>> {
        let result = self
            .execution_module
            .lock()
            .expect("execution module lock should not be poisoned")
            .on_market_update(normalized_data, &mut self.portfolio, now_ms, trace_id);
        runtime_events.extend(result.events.clone());
        Ok(result.fills)
    }
    fn refresh_portfolio_state(&mut self, normalized_data: &[NormalizedMarketData], now_ms: u64) {
        let quotes = quote_price_map(normalized_data);
        let mut exposures: BTreeMap<Exchange, (f64, f64)> = BTreeMap::new();

        for position in &mut self.portfolio.positions {
            position.mark_price = quotes
                .get(&(position.exchange.clone(), position.symbol.clone()))
                .copied()
                .unwrap_or(position.mark_price.max(position.avg_entry_price));
            position.unrealized_pnl =
                (position.mark_price - position.avg_entry_price) * position.net_qty;
            let gross = position.net_qty.abs() * position.mark_price;
            let net = position.net_qty * position.mark_price;
            let entry = exposures.entry(position.exchange.clone()).or_default();
            entry.0 += gross;
            entry.1 += net;
        }

        self.portfolio.exchange_exposures = exposures
            .into_iter()
            .map(
                |(exchange, (gross_notional, net_notional))| ExchangeExposure {
                    leverage: 0.0,
                    exchange,
                    gross_notional,
                    net_notional,
                },
            )
            .collect();

        self.portfolio.total_gross_notional = self
            .portfolio
            .exchange_exposures
            .iter()
            .map(|item| item.gross_notional)
            .sum();
        self.portfolio.total_net_notional = self
            .portfolio
            .exchange_exposures
            .iter()
            .map(|item| item.net_notional)
            .sum();
        let equity_estimate = portfolio_equity_estimate(&self.portfolio);
        self.portfolio.total_leverage = if equity_estimate.abs() > f64::EPSILON {
            self.portfolio.total_gross_notional / equity_estimate.abs().max(1.0)
        } else {
            0.0
        };
        for exposure in &mut self.portfolio.exchange_exposures {
            exposure.leverage = if equity_estimate.abs() > f64::EPSILON {
                exposure.gross_notional / equity_estimate.abs().max(1.0)
            } else {
                0.0
            };
        }
        self.portfolio.updated_at_ms = now_ms;
    }
}

fn portfolio_equity_estimate(portfolio: &PortfolioState) -> f64 {
    portfolio.cash_balance + portfolio.total_net_notional
}

fn quote_price_map(normalized_data: &[NormalizedMarketData]) -> BTreeMap<(Exchange, Symbol), f64> {
    normalized_data
        .iter()
        .filter_map(|item| match item {
            NormalizedMarketData::Quote(quote) => Some((
                (quote.exchange.clone(), quote.symbol.clone()),
                quote.mid_price,
            )),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_compiler::compile_runtime_protocol_config;
    use qrpc_core::{
        AgentConfig, DataKind, DataSourceConfig, DecisionStatus, IntentConfig, MarketType,
        RiskConfig, RuntimeProtocolCoreConfig,
    };
    use std::collections::BTreeSet;

    #[derive(Default)]
    struct RejectAllRiskChecker;

    #[derive(Default)]
    struct NoopExecutionModule;

    #[derive(Default)]
    struct NoopAgentModule;

    #[derive(Default)]
    struct NoopIntentModule;

    #[derive(Default)]
    struct NoopDataModule;

    impl DataModuleProvider for NoopDataModule {
        fn provider_key(&self) -> &'static str {
            "test.data.noop"
        }

        fn collect(&self, _request: DataCollectionRequest<'_>) -> Result<DataCollectionOutput> {
            Ok(DataCollectionOutput {
                normalized_data: Vec::new(),
                events: Vec::new(),
            })
        }
    }

    impl IntentModuleProvider for NoopIntentModule {
        fn provider_key(&self) -> &'static str {
            "test.intent.noop"
        }

        fn evaluate_intents(
            &self,
            _request: IntentEvaluationRequest<'_>,
        ) -> IntentEvaluationOutput {
            IntentEvaluationOutput {
                signals: Vec::new(),
                events: Vec::new(),
            }
        }
    }

    impl AgentModuleProvider for NoopAgentModule {
        fn provider_key(&self) -> &'static str {
            "test.agent.noop"
        }

        fn evaluate_agents(&self, _request: AgentEvaluationRequest<'_>) -> AgentEvaluationOutput {
            AgentEvaluationOutput {
                decisions: Vec::new(),
                events: Vec::new(),
                evaluated_rebalance_agent_ids: BTreeSet::new(),
            }
        }
    }

    impl ExecutionModuleProvider for NoopExecutionModule {
        fn provider_key(&self) -> &'static str {
            "test.execution.noop"
        }

        fn plan_execution(
            &self,
            _request: ExecutionPlanningRequest<'_>,
        ) -> ExecutionPlanningOutput {
            ExecutionPlanningOutput {
                plans: Vec::new(),
                events: Vec::new(),
            }
        }

        fn submit_plan(
            &mut self,
            plan: &ExecutionPlan,
            _normalized_data: &[NormalizedMarketData],
            _portfolio: &mut PortfolioState,
            _now_ms: u64,
            _trace_id: &str,
        ) -> qrpc_core::FillResult {
            qrpc_core::FillResult {
                plan_id: plan.plan_id.clone(),
                status: qrpc_core::ExecutionStatus::Rejected,
                fills: Vec::new(),
                open_orders: Vec::new(),
                events: Vec::new(),
            }
        }

        fn on_market_update(
            &mut self,
            _normalized_data: &[NormalizedMarketData],
            _portfolio: &mut PortfolioState,
            now_ms: u64,
            _trace_id: &str,
        ) -> qrpc_core::FillResult {
            qrpc_core::FillResult {
                plan_id: format!("noop-market-{now_ms}"),
                status: qrpc_core::ExecutionStatus::Open,
                fills: Vec::new(),
                open_orders: Vec::new(),
                events: Vec::new(),
            }
        }
    }

    impl RiskCheckerProvider for RejectAllRiskChecker {
        fn provider_key(&self) -> &'static str {
            "test.risk.reject_all"
        }

        fn evaluate(&self, request: RiskCheckRequest<'_>) -> Result<RiskCheckOutput> {
            Ok(RiskCheckOutput {
                decisions: request
                    .decisions
                    .iter()
                    .map(|decision| RiskDecision {
                        risk_decision_id: format!(
                            "reject-{}-{}",
                            decision.decision_id, request.now_ms
                        ),
                        risk_id: request
                            .core_ir
                            .risk_policies
                            .first()
                            .map(|risk| risk.policy_id.clone())
                            .unwrap_or_else(|| "risk_override".into()),
                        agent_decision_id: decision.decision_id.clone(),
                        symbol: decision.symbol.clone(),
                        status: DecisionStatus::Reject,
                        adjusted_portfolio_target_decision: None,
                        adjusted_actions: Vec::new(),
                        reason_codes: vec![qrpc_core::RiskReasonCode::InvalidAction],
                        reason_text: "rejected by test provider".into(),
                        produced_at_ms: request.now_ms,
                        trace_id: request.trace_id.to_string(),
                    })
                    .collect(),
                events: Vec::new(),
                approved_agent_ids: BTreeSet::new(),
            })
        }
    }

    #[test]
    fn session_runs_both_slow_and_fast_cycles() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::new(compiled);
        let output = runtime
            .run_session(1_700_000_000_000, 1_700_000_005_000)
            .unwrap();

        assert!(!output.slow_cycle.intent_signals.is_empty());
        assert!(!output.fast_cycle.intent_signals.is_empty());
        assert!(!output.slow_cycle.execution_plans.is_empty());
        assert!(
            output
                .data_fetch_counts
                .get("binance_btc_quote")
                .copied()
                .unwrap_or_default()
                >= 1
        );
    }

    #[test]
    fn risk_clamps_when_leverage_limit_is_tight() {
        let mut config = sample_config();
        config.risks[0].max_total_leverage = 0.05;
        config.risks[0].max_exchange_leverage = 0.05;
        let compiled = compile_runtime_protocol_config(&config).unwrap();
        let mut runtime = RuntimeCoordinator::new(compiled);
        let output = runtime.run_slow_cycle(1_700_000_005_000).unwrap();

        assert!(output
            .risk_decisions
            .iter()
            .any(|item| matches!(item.status, DecisionStatus::Clamp)));
    }

    #[test]
    fn custom_data_module_provider_can_override_runtime_inputs() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::with_data_module(compiled, NoopDataModule);
        let output = runtime.run_fast_cycle(1_700_000_005_000).unwrap();

        assert!(output.normalized_data.is_empty());
        assert!(output.intent_signals.is_empty());
        assert!(output.agent_decisions.is_empty());
        assert_eq!(runtime.data_module().provider_key(), "test.data.noop");
    }

    #[test]
    fn custom_intent_module_provider_can_override_runtime_signals() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::with_intent_module(compiled, NoopIntentModule);
        let output = runtime.run_fast_cycle(1_700_000_005_000).unwrap();

        assert!(output
            .normalized_data
            .iter()
            .any(|item| matches!(item, NormalizedMarketData::Quote(_))));
        assert!(output.intent_signals.is_empty());
        assert!(output.agent_decisions.is_empty());
        assert_eq!(runtime.intent_module().provider_key(), "test.intent.noop");
    }

    #[test]
    fn custom_agent_module_provider_can_override_runtime_decisions() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::with_agent_module(compiled, NoopAgentModule);
        let output = runtime.run_fast_cycle(1_700_000_005_000).unwrap();

        assert!(!output.intent_signals.is_empty());
        assert!(output.agent_decisions.is_empty());
        assert!(output.risk_decisions.is_empty());
        assert_eq!(runtime.agent_module().provider_key(), "test.agent.noop");
    }

    #[test]
    fn custom_risk_checker_provider_can_override_runtime_decisions() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::with_risk_checker(compiled, RejectAllRiskChecker);
        let output = runtime.run_slow_cycle(1_700_000_005_000).unwrap();

        assert!(!output.agent_decisions.is_empty());
        assert!(output
            .risk_decisions
            .iter()
            .all(|item| matches!(item.status, DecisionStatus::Reject)));
        assert!(output.execution_plans.is_empty());
        assert_eq!(
            runtime.risk_checker().provider_key(),
            "test.risk.reject_all"
        );
    }

    #[test]
    fn custom_execution_module_provider_can_override_runtime_plans() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::with_execution_module(compiled, NoopExecutionModule);
        let output = runtime.run_slow_cycle(1_700_000_005_000).unwrap();

        assert!(!output.risk_decisions.is_empty());
        assert!(output.execution_plans.is_empty());
        assert!(output.fill_reports.is_empty());
        assert_eq!(
            runtime.execution_module_provider_key(),
            "test.execution.noop"
        );
    }

    #[test]
    fn fills_are_traceable_back_to_runtime_chain() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::new(compiled);
        let output = runtime
            .run_session(1_700_000_000_000, 1_700_000_005_000)
            .unwrap();

        let trace_id = output.slow_cycle.trace_id.clone();
        assert!(!output.slow_cycle.fill_reports.is_empty());
        assert!(output
            .slow_cycle
            .fill_reports
            .iter()
            .all(|fill| fill.trace_id == trace_id));
        assert!(output
            .slow_cycle
            .runtime_events
            .iter()
            .any(|evt| evt.trace_id == trace_id
                && evt.event_type == RuntimeEventType::ExecutionFilled));
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
                    data_id: "okx_btc_150d_1d".into(),
                    exchange: Exchange::Okx,
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
