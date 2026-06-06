use super::RuntimeCoordinator;
use crate::{
    agent_module::AgentModuleProvider, data_module::DataModuleProvider,
    intent_module::IntentModuleProvider, risk_checker::RiskCheckerProvider, risk_monitor,
    slippage::ExecutionAssumptions,
};
use qrpc_core::{PortfolioState, RiskDecisionMode};
use std::collections::BTreeMap;

impl RuntimeCoordinator {
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

    /// Sets the live risk monitor and clears the stopped flag.
    pub fn set_risk_monitor(&mut self, monitor: risk_monitor::RiskMonitor) {
        self.risk_monitor = Some(monitor);
        self.risk_stopped = false;
    }

    /// Returns whether the risk monitor has triggered a stop.
    pub fn is_risk_stopped(&self) -> bool {
        self.risk_stopped
    }

    /// Clears the risk monitor stopped flag.
    pub fn reset_risk_stopped(&mut self) {
        self.risk_stopped = false;
    }

    pub fn execution_module_provider_key(&self) -> &'static str {
        self.execution_module
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .provider_key()
    }

    /// Updates execution assumptions for slippage, impact, spread, and latency.
    pub fn set_execution_assumptions(&self, assumptions: ExecutionAssumptions) {
        self.execution_module
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .set_execution_assumptions(assumptions);
    }
}
