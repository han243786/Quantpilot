use super::RuntimeCoordinator;
use crate::{
    agent_module::{AgentModuleProvider, BuiltinAgentModule},
    config_tracker,
    data_module::{BuiltinDataModule, DataModuleProvider},
    execution_module::{BuiltinExecutionModule, ExecutionModuleProvider},
    intent_module::{BuiltinIntentModule, IntentModuleProvider},
    merge::{MergePolicy, StrategyMergeEngine},
    merge_coordinator,
    risk_checker::{RiskChecker, RiskCheckerProvider},
    runtime_state,
};
use qrpc_core::{CompiledRuntimeProtocol, CoreStrategyIr, PortfolioState, RiskDecisionMode};
use std::collections::BTreeMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

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
}
