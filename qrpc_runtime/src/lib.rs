mod agent_module;
pub mod backtest_metrics;
mod compat;
pub mod circuit_breaker;
mod config_tracker;
mod core_ir_evaluator;
pub(crate) mod data_module;
mod execution_module;
mod fill_engine;
mod intent_module;
pub mod live_execution;
mod merge;
mod merge_coordinator;
mod plugin_runtime_registry;
mod runtime_state;
pub mod plugin_market;
pub mod plugin_sandbox;
mod reconcile;
mod risk_checker;
pub mod risk_monitor;
mod sandbox;
pub mod slippage;
pub mod hotswap;

pub use slippage::{
    compute_fill_price, estimate_spread, spread_from_quote, ExecutionAssumptions,
    ExtendedMarketState, LatencyModel, MarketImpactModel, SlippageModel, SpreadEstimate,
    SpreadEstimateSource,
};

use anyhow::Result;
use qrpc_core::{
    AgentDecision, CompiledRuntimeProtocol, CoreStrategyIr, Exchange, ExchangeExposure,
    ExecutionPlan, FillReport, IntentKind, IntentSignal, NormalizedMarketData, PortfolioState,
    RiskDecision, RiskDecisionMode, RuntimeCycleOutput, RuntimeEvent, RuntimeEventType,
    SessionOutput, Symbol,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub use agent_module::{
    AgentEvaluationOutput, AgentEvaluationRequest, AgentModuleProvider, BuiltinAgentModule,
};
pub use core_ir_evaluator::{
    evaluate_indicator_signal, CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError,
};
/// v1.1.x: 设置 Mock 数据生成器的波动率(0=使用默认1.5%)
pub fn set_mock_volatility(vol: f64) {
    data_module::MOCK_VOLATILITY.store(vol.to_bits(), std::sync::atomic::Ordering::Relaxed);
}

pub use data_module::{
    BuiltinDataModule, DataCollectionOutput, DataCollectionRequest, DataModuleProvider,
};
pub use execution_module::{
    BuiltinExecutionModule, ExecutionModuleProvider, ExecutionPlanner,
    ExecutionPlanningOutput, ExecutionPlanningRequest, ExecutionSubmitter,
};
pub use intent_module::{
    BuiltinIntentModule, IntentEvaluationOutput, IntentEvaluationRequest, IntentModuleProvider,
};
pub use plugin_runtime_registry::{
    PluginLifecycleState, PluginSecurityAction, RuntimePluginLifecycle, RuntimePluginRegistry,
};
pub use plugin_market::{MarketMetadata, PluginMarketClient, PluginSummary};
pub use risk_checker::{RiskCheckOutput, RiskCheckRequest, RiskChecker, RiskCheckerProvider};
pub use hotswap::{
    DefaultHotSwapValidator, HotSwapModuleTarget, HotSwapOrchestrator, HotSwapRequest,
    HotSwapResult, HotSwapState, HotSwapStep, HotSwapValidationResult, HotSwapValidator,
};
pub use compat::{
    CompatibilityChecker, CompatibilityContext, CompatibilityReport, CompatibilityVerdict,
    ModuleSurface,
};
pub use reconcile::{
    OrderReconciler, ReconciliationDiscrepancy, ReconciliationResult, ReconcileStrategy,
};
pub use merge::{
    MergeDecisionRecord, MergePolicy, MergedOutput, StrategyInput, StrategyMergeEngine,
};
pub use sandbox::{
    runtime_support_boundary, DeterministicClockMode, DeterministicEventOrdering,
    DeterministicParallelismPolicy, DeterministicTestMode, FastBacktestSandbox, RealTimeSandbox,
    RuntimeSupportBoundary, Sandbox, SandboxMode, SandboxSnapshot,
    SUPPORTED_RUNTIME_EXECUTION_MODULE_KEYS, SUPPORTED_RUNTIME_MODE_KEYS,
};
pub use data_module::MOCK_VOLATILITY;

/// 配置代际记录
#[derive(Debug, Clone)]
pub struct ConfigGenerationEntry {
    pub generation: u64,
    pub activated_at_ms: u64,
    pub deployment_revision: String,
    pub parameter_version: String,
}

#[derive(Clone)]
pub struct RuntimeCoordinator {
    // v2.2.0: 提取为子结构体的可变状态
    pub(crate) state: runtime_state::RuntimeState,
    pub(crate) config: config_tracker::ConfigTracker,
    pub(crate) merge: merge_coordinator::MergeCoordinator,
    // 核心编译产物 (不可变引用)
    core_ir: CoreStrategyIr,
    // 5 个模块提供者 (trait objects, 依赖注入)
    data_module: Arc<dyn DataModuleProvider>,
    intent_module: Arc<dyn IntentModuleProvider>,
    agent_module: Arc<dyn AgentModuleProvider>,
    execution_module: Arc<Mutex<dyn ExecutionModuleProvider>>,
    risk_checker: Arc<dyn RiskCheckerProvider>,
    // 杂项状态
    risk_mode: RiskDecisionMode,
    pending_module_configs: BTreeMap<String, serde_json::Value>,
    /// v1.2.0: 独立实时风控监控器（可选）
    risk_monitor: Option<risk_monitor::RiskMonitor>,
    /// v1.2.0: RiskMonitor 是否已触发停止
    risk_stopped: bool,
}

impl std::fmt::Debug for RuntimeCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeCoordinator")
            .field("core_ir", &self.core_ir)
            .field("portfolio", &self.state.portfolio)
            .field("data_fetch_counts", &self.state.data_fetch_counts)
            .field("last_action_at_ms", &self.state.last_action_at_ms)
            .field("last_rebalance_at_ms", &self.state.last_rebalance_at_ms)
            .field("data_provider_key", &self.data_module.provider_key())
            .field("intent_provider_key", &self.intent_module.provider_key())
            .field("agent_provider_key", &self.agent_module.provider_key())
            .field(
                "execution_provider_key",
                &self.execution_module_provider_key(),
            )
            .field("risk_provider_key", &self.risk_checker.provider_key())
            .field("risk_stopped", &self.risk_stopped)
            .field("risk_monitor_enabled", &self.risk_monitor.is_some())
            .finish()
    }
}

impl RuntimeCoordinator {
    pub fn from_core_ir(core_ir: CoreStrategyIr) -> Self {
        Self::with_modules_from_core_ir(
            core_ir,
            BuiltinDataModule::default(),
            BuiltinIntentModule,
            BuiltinAgentModule,
            RiskChecker,
            BuiltinExecutionModule::default(),
        )
    }

    pub fn new(compiled: CompiledRuntimeProtocol) -> Self {
        // v2.3.0 TODO: 从 RiskPolicy 接线 RiskMonitor (需在 RiskPolicy 中添加 max_drawdown_ratio/max_daily_loss_ratio 字段)
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
            BuiltinIntentModule,
            BuiltinAgentModule,
            RiskChecker,
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
            BuiltinAgentModule,
            RiskChecker,
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
            BuiltinIntentModule,
            agent_module,
            RiskChecker,
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
            BuiltinIntentModule,
            BuiltinAgentModule,
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
            BuiltinIntentModule,
            BuiltinAgentModule,
            RiskChecker,
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
            state: runtime_state::RuntimeState {
                portfolio,
                data_fetch_counts: BTreeMap::new(),
                last_action_at_ms: BTreeMap::new(),
                last_rebalance_at_ms: BTreeMap::new(),
            },
            config: config_tracker::ConfigTracker {
                applied_deployment_revisions: Vec::new(),
                config_generation: Arc::new(AtomicU64::new(1)),
                config_generation_history: Arc::new(std::sync::Mutex::new(Vec::new())),
            },
            merge: merge_coordinator::MergeCoordinator {
                engine: StrategyMergeEngine::default(),
                policy: MergePolicy::WeightedMerge,
                records: Vec::new(),
            },
            data_module,
            intent_module,
            agent_module,
            execution_module,
            risk_checker,
            risk_mode: RiskDecisionMode::Normal,
            pending_module_configs: BTreeMap::new(),
            risk_monitor: None,
            risk_stopped: false,
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
            .unwrap_or_else(|e| e.into_inner())
            .submit_plan(plan, normalized_data, &mut self.state.portfolio, now_ms, trace_id);
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
            .unwrap_or_else(|e| e.into_inner())
            .on_market_update(normalized_data, &mut self.state.portfolio, now_ms, trace_id);
        self.refresh_portfolio_state(normalized_data, now_ms);
        Ok(result)
    }

    pub fn portfolio_state(&self) -> &PortfolioState {
        &self.state.portfolio
    }

    pub fn data_fetch_counts(&self) -> &BTreeMap<String, u32> {
        &self.state.data_fetch_counts
    }

    pub fn last_action_at_ms(&self) -> &BTreeMap<String, u64> {
        &self.state.last_action_at_ms
    }

    pub fn last_rebalance_at_ms(&self) -> &BTreeMap<String, u64> {
        &self.state.last_rebalance_at_ms
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

    pub fn risk_mode(&self) -> RiskDecisionMode {
        self.risk_mode
    }

    pub fn set_risk_mode(&mut self, mode: RiskDecisionMode) {
        self.risk_mode = mode;
    }

    /// v1.2.0: 设置 RiskMonitor 实时风控监控器
    pub fn set_risk_monitor(&mut self, monitor: risk_monitor::RiskMonitor) {
        self.risk_monitor = Some(monitor);
        self.risk_stopped = false;
    }

    /// v1.2.0: 查询 RiskMonitor 是否已触发停止
    pub fn is_risk_stopped(&self) -> bool {
        self.risk_stopped
    }

    /// v1.2.0: 重置 RiskMonitor 停止标志（用于恢复运行）
    pub fn reset_risk_stopped(&mut self) {
        self.risk_stopped = false;
    }

    /// 存储候选模块配置，在下一次 epoch barrier 激活
    /// 返回新的 deployment_revision（sha256 哈希）
    pub fn swap_module_config(
        &mut self,
        module_key: &str,
        config: serde_json::Value,
    ) -> Result<String> {
        self.pending_module_configs
            .insert(module_key.to_string(), config);
        // v1.2.1: 使用序号替代墙钟时间戳，保证回测确定性
        let revision_input = serde_json::json!({
            "module_key": module_key,
            "revision_seq": self.config.applied_deployment_revisions.len(),
        });
        let digest = qrpc_core::canonical_json_sha256_digest(&revision_input)?;
        let revision = format!("rev-hotswap-{}", &digest.value[..16]);
        self.config.applied_deployment_revisions.push(revision.clone());
        // v1.1.4: 窗口截断保留最近 1000 条，防止长期运行无界增长
        const MAX_REVISIONS: usize = 1000;
        if self.config.applied_deployment_revisions.len() > MAX_REVISIONS {
            let excess = self.config.applied_deployment_revisions.len() - MAX_REVISIONS;
            self.config.applied_deployment_revisions.drain(0..excess);
        }
        Ok(revision)
    }

    /// 应用所有待处理的模块配置（在 epoch barrier 调用）
    pub fn apply_pending_module_configs(&mut self) -> Vec<String> {
        let count = self.pending_module_configs.len();
        self.pending_module_configs.clear();
        if count > 0 {
            let gen = self.config.config_generation.fetch_add(1, Ordering::SeqCst);
            // v1.2.1: 使用代际序号代替墙钟，保证回测确定性
            let now_ms = gen;
            let rev = self.config
                .applied_deployment_revisions
                .last()
                .cloned()
                .unwrap_or_else(|| "rev-unknown".to_string());
            if let Ok(mut history) = self.config.config_generation_history.lock() {
                // v2.1.0: 限制历史条目数防止无界增长
                const MAX_CONFIG_HISTORY: usize = 1000;
                let len = history.len();
                if len >= MAX_CONFIG_HISTORY {
                    history.drain(0..len - MAX_CONFIG_HISTORY + 1);
                }
                history.push(ConfigGenerationEntry {
                    generation: gen,
                    activated_at_ms: now_ms,
                    deployment_revision: rev.clone(),
                    parameter_version: format!("gen-{}", gen),
                });
            }
            vec![rev]
        } else {
            Vec::new()
        }
    }

    /// 当前配置代际号
    pub fn current_generation(&self) -> u64 {
        self.config.config_generation.load(Ordering::Relaxed)
    }

    /// 配置代际历史
    pub fn generation_history(&self) -> Vec<ConfigGenerationEntry> {
        self.config.config_generation_history
            .lock()
            .map(|h| h.clone())
            .unwrap_or_default()
    }

    pub fn execution_module_provider_key(&self) -> &'static str {
        self.execution_module
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .provider_key()
    }

    /// v1.1.0: 设置执行假设（滑点/冲击/价差/延迟模型）
    pub fn set_execution_assumptions(&self, assumptions: crate::slippage::ExecutionAssumptions) {
        self.execution_module
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .set_execution_assumptions(assumptions);
    }

    pub fn portfolio_update_event(
        &self,
        source_id: &str,
        trace_id: &str,
        now_ms: u64,
    ) -> RuntimeEvent {
        let equity_estimate = portfolio_equity_estimate(&self.state.portfolio);
        let open_orders = self.state
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
                "cash_balance": self.state.portfolio.cash_balance,
                "available_cash_balance": self.state.portfolio.available_cash_balance,
                "frozen_cash_balance": self.state.portfolio.frozen_cash_balance,
                "total_gross_notional": self.state.portfolio.total_gross_notional,
                "total_net_notional": self.state.portfolio.total_net_notional,
                "total_leverage": self.state.portfolio.total_leverage,
                "equity_estimate": equity_estimate,
                "positions": self.state.portfolio.positions.len(),
                "open_order_count": self.state.portfolio.open_orders.len(),
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
            data_fetch_counts: &mut self.state.data_fetch_counts,
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
            portfolio: &self.state.portfolio,
            last_rebalance_at_ms: &self.state.last_rebalance_at_ms,
            now_ms,
            trace_id,
        });
        for agent_id in &output.evaluated_rebalance_agent_ids {
            self.state.last_rebalance_at_ms.insert(agent_id.clone(), now_ms);
        }
        runtime_events.extend(output.events);
        output.decisions
    }

    /// 合并引擎：多策略场景下汇聚 Agent 决策为统一候选
    fn merge_agent_decisions(
        &mut self,
        cycle_name: &str,
        decisions: &[AgentDecision],
        _signals: &[IntentSignal],
        trace_id: &str,
        runtime_events: &mut Vec<RuntimeEvent>,
    ) -> (Vec<AgentDecision>, Option<MergeDecisionRecord>) {
        if decisions.is_empty() {
            return (Vec::new(), None);
        }
        // 单策略场景：直接透传
        if decisions.len() <= 1 {
            return (decisions.to_vec(), None);
        }
        // 多策略场景：调用合并引擎
        let strategy_input = StrategyInput {
            strategy_id: cycle_name.to_string(),
            weight: 1.0,
            agent_decisions: decisions.to_vec(),
        };
        match self.merge.engine.merge(&[strategy_input]) {
            Ok(output) => {
                runtime_events.push(RuntimeEvent {
                    event_id: format!("evt-merge-{}-{}", cycle_name, runtime_events.len()),
                    event_type: RuntimeEventType::AgentDecisionProduced,
                    trace_id: trace_id.to_string(),
                    source_id: "merge_engine".to_string(),
                    ts_ms: 0, // v1.2.1: 合成事件不使用墙钟，保证回测确定性
                    payload: serde_json::json!({
                        "message": "merge engine produced unified decisions",
                        "input_count": decisions.len(),
                        "output_count": output.decisions.len(),
                        "merge_policy": format!("{:?}", self.merge.policy),
                        "conflicts": output.conflict_count,
                        "suppressed": output.suppressed_count,
                    }),
                });
                let record = output.merge_records.first().cloned();
                (output.decisions, record)
            }
            Err(_err) => {
                runtime_events.push(RuntimeEvent {
                    event_id: format!("evt-merge-err-{}-{}", cycle_name, runtime_events.len()),
                    event_type: RuntimeEventType::RuntimeWarning,
                    trace_id: trace_id.to_string(),
                    source_id: "merge_engine".to_string(),
                    ts_ms: 0, // v1.2.1: 合成事件不使用墙钟，保证回测确定性
                    payload: serde_json::json!({
                        "message": "merge engine fallback to pass-through",
                    }),
                });
                (decisions.to_vec(), None)
            }
        }
    }

    /// 查询合并记录（供 API 使用）
    pub fn merge_records(&self) -> &[MergeDecisionRecord] {
        &self.merge.records
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
                portfolio: &self.state.portfolio,
                last_action_at_ms: &self.state.last_action_at_ms,
                now_ms,
                trace_id,
                mode: self.risk_mode,
            })
            .unwrap_or_else(|_| RiskCheckOutput {
                decisions: vec![],
                events: vec![],
                approved_agent_ids: std::collections::BTreeSet::new(),
            });

        for agent_id in &output.approved_agent_ids {
            self.state.last_action_at_ms.insert(agent_id.clone(), now_ms);
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
            .unwrap_or_else(|e| e.into_inner())
            .plan_execution(ExecutionPlanningRequest {
                risk_decisions,
                core_ir: &self.core_ir,
                normalized_data,
                portfolio: &self.state.portfolio,
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
                .unwrap_or_else(|e| e.into_inner())
                .submit_plan(plan, normalized_data, &mut self.state.portfolio, now_ms, trace_id);
            let mut result = result;
            runtime_events.append(&mut result.events);
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
            .unwrap_or_else(|e| e.into_inner())
            .on_market_update(normalized_data, &mut self.state.portfolio, now_ms, trace_id);
        let mut result = result;
        runtime_events.append(&mut result.events);
        Ok(result.fills)
    }
    fn refresh_portfolio_state(&mut self, normalized_data: &[NormalizedMarketData], now_ms: u64) {
        let quotes = quote_price_map(normalized_data);
        let mut exposures: BTreeMap<Exchange, (f64, f64)> = BTreeMap::new();

        for position in &mut self.state.portfolio.positions {
            // v2.3.0: 若无新行情, 保持上次市价并记录陈旧状态
            position.mark_price = quotes
                .get(&(position.exchange.clone(), position.symbol.clone()))
                .copied()
                .unwrap_or(position.mark_price);
            // 无行情时标记: 使用 avg_entry_price 作为保守估计 (不做多, 也不做空)
            if position.mark_price <= 0.0 {
                position.mark_price = position.avg_entry_price.max(0.01);
            }
            position.unrealized_pnl =
                (position.mark_price - position.avg_entry_price) * position.net_qty;
            let gross = position.net_qty.abs() * position.mark_price;
            let net = position.net_qty * position.mark_price;
            let entry = exposures.entry(position.exchange.clone()).or_default();
            entry.0 += gross;
            entry.1 += net;
        }

        self.state.portfolio.exchange_exposures = exposures
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

        self.state.portfolio.total_gross_notional = self.state
            .portfolio
            .exchange_exposures
            .iter()
            .map(|item| item.gross_notional)
            .sum();
        self.state.portfolio.total_net_notional = self.state
            .portfolio
            .exchange_exposures
            .iter()
            .map(|item| item.net_notional)
            .sum();
        let equity_estimate = portfolio_equity_estimate(&self.state.portfolio);
        // v2.3.0: 使用比例下限替代固定 floor(1.0), 避免小额权益时杠杆被低估
        let initial_equity = 100_000.0;
        let equity_floor = 1_000.0_f64.max(initial_equity * 0.001);
        self.state.portfolio.total_leverage = if equity_estimate.abs() > f64::EPSILON {
            self.state.portfolio.total_gross_notional / equity_estimate.abs().max(equity_floor)
        } else {
            0.0
        };
        for exposure in &mut self.state.portfolio.exchange_exposures {
            exposure.leverage = if equity_estimate.abs() > f64::EPSILON {
                exposure.gross_notional / equity_estimate.abs().max(equity_floor)
            } else {
                0.0
            };
        }
        self.state.portfolio.updated_at_ms = now_ms;
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

    impl ExecutionPlanner for NoopExecutionModule {
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
    }

    impl ExecutionSubmitter for NoopExecutionModule {
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
                        mode: request.mode,
                        adjusted_portfolio_target_decision: None,
                        adjusted_actions: Vec::new(),
                        reason_codes: vec![qrpc_core::RiskReasonCode::InvalidAction],
                        reason_text: "rejected by test provider".into(),
                        produced_at_ms: request.now_ms,
                        trace_id: request.trace_id.to_string(),
                    })
                    .collect(),
                events: Vec::new(),
                approved_agent_ids: std::collections::BTreeSet::new(),
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
