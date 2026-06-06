use super::RuntimeCoordinator;
use crate::{
    agent_module::AgentEvaluationRequest,
    data_module::DataCollectionRequest,
    execution_module::ExecutionPlanningRequest,
    intent_module::IntentEvaluationRequest,
    merge::{MergeDecisionRecord, StrategyInput},
    risk_checker::{RiskCheckOutput, RiskCheckRequest},
};
use anyhow::Result;
use qrpc_core::{
    AgentDecision, ExecutionPlan, FillReport, IntentKind, IntentSignal, NormalizedMarketData,
    RiskDecision, RuntimeEvent, RuntimeEventType,
};

impl RuntimeCoordinator {
    pub(super) fn collect_normalized_data(
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

    pub(super) fn evaluate_intents(
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

    pub(super) fn evaluate_agents(
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
            self.state
                .last_rebalance_at_ms
                .insert(agent_id.clone(), now_ms);
        }
        runtime_events.extend(output.events);
        output.decisions
    }

    pub(super) fn merge_agent_decisions(
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
        if decisions.len() <= 1 {
            return (decisions.to_vec(), None);
        }
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
                    ts_ms: 0,
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
                    ts_ms: 0,
                    payload: serde_json::json!({
                        "message": "merge engine fallback to pass-through",
                    }),
                });
                (decisions.to_vec(), None)
            }
        }
    }

    pub fn merge_records(&self) -> &[MergeDecisionRecord] {
        &self.merge.records
    }

    pub(super) fn evaluate_risks(
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
            self.state
                .last_action_at_ms
                .insert(agent_id.clone(), now_ms);
        }
        runtime_events.extend(output.events);
        output.decisions
    }

    pub(super) fn plan_execution(
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

    pub(super) fn execute_plans(
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
                .submit_plan(
                    plan,
                    normalized_data,
                    &mut self.state.portfolio,
                    now_ms,
                    trace_id,
                );
            let mut result = result;
            runtime_events.append(&mut result.events);
            fills.extend(result.fills);
        }

        Ok(fills)
    }

    pub(super) fn process_open_orders(
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
}
