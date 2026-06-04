use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{
    AgentDecision, ExecutionPlan, FillReport, IntentSignal, NormalizedMarketData, PortfolioState,
    RiskDecision,
};
use crate::RuntimeEventType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub event_id: String,
    pub event_type: RuntimeEventType,
    pub trace_id: String,
    pub source_id: String,
    pub ts_ms: u64,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCycleOutput {
    pub cycle_name: String,
    pub trace_id: String,
    pub normalized_data: Vec<NormalizedMarketData>,
    pub intent_signals: Vec<IntentSignal>,
    pub agent_decisions: Vec<AgentDecision>,
    pub risk_decisions: Vec<RiskDecision>,
    pub execution_plans: Vec<ExecutionPlan>,
    pub fill_reports: Vec<FillReport>,
    pub portfolio_state: PortfolioState,
    pub runtime_events: Vec<RuntimeEvent>,
    pub data_fetch_counts: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOutput {
    pub slow_cycle: RuntimeCycleOutput,
    pub fast_cycle: RuntimeCycleOutput,
    pub final_portfolio: PortfolioState,
    pub data_fetch_counts: BTreeMap<String, u32>,
}
