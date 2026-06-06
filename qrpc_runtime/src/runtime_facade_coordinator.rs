use crate::{
    agent_module::AgentModuleProvider, config_tracker, data_module::DataModuleProvider,
    execution_module::ExecutionModuleProvider, intent_module::IntentModuleProvider,
    merge_coordinator, risk_checker::RiskCheckerProvider, risk_monitor, runtime_state,
};
use qrpc_core::{CoreStrategyIr, RiskDecisionMode};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

mod config_generation;
mod constructor_provider_wiring;
mod execution_market_entrypoints;
mod portfolio_projection;
mod provider_delegation_helpers;
mod session_cycle_orchestration;
mod state_config_accessors;

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

#[cfg(test)]
mod coordinator_test_harness;
