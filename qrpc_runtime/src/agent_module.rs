use qrpc_core::{
    AgentDecision, CoreStrategyIr, Exchange, IntentSignal, PortfolioState, RuntimeEvent,
    RuntimeEventType, SignalSide, Symbol,
};
use qrpc_core_ir::{AgentPolicyKind, SignalKind};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

mod cross_venue_arbitrage;
mod portfolio_rebalance;
mod weighted_signal_decisions;

// Shared sizing constants stay parent-owned for child-module mediation.
const MIN_QUANTITY_RATIO: f64 = 0.01;
const DEFAULT_DECISION_THRESHOLD: f64 = 0.05;
const SPREAD_MULTIPLIER: f64 = 20.0;
#[cfg(test)]
const DEFAULT_COST_BUFFER_BPS: f64 = 20.0;

#[derive(Debug, Clone)]
pub struct AgentEvaluationRequest<'a> {
    pub cycle_name: &'a str,
    pub signals: &'a [IntentSignal],
    pub core_ir: &'a CoreStrategyIr,
    pub portfolio: &'a PortfolioState,
    pub last_rebalance_at_ms: &'a BTreeMap<String, u64>,
    pub now_ms: u64,
    pub trace_id: &'a str,
}

#[derive(Debug, Clone)]
pub struct AgentEvaluationOutput {
    pub decisions: Vec<AgentDecision>,
    pub events: Vec<RuntimeEvent>,
    pub evaluated_rebalance_agent_ids: BTreeSet<String>,
}

pub trait AgentModuleProvider: Send + Sync {
    fn provider_key(&self) -> &'static str {
        "builtin.agent.default"
    }

    fn evaluate_agents(&self, request: AgentEvaluationRequest<'_>) -> AgentEvaluationOutput;
}

#[derive(Debug, Clone, Default)]
pub struct BuiltinAgentModule;

impl AgentModuleProvider for BuiltinAgentModule {
    fn evaluate_agents(&self, request: AgentEvaluationRequest<'_>) -> AgentEvaluationOutput {
        let agent_policies = &request.core_ir.agent_policies;
        let mut decisions = Vec::with_capacity(agent_policies.len());
        let mut events = Vec::with_capacity(agent_policies.len());
        let mut evaluated_rebalance_agent_ids = BTreeSet::new();

        for agent in agent_policies.iter().filter(|item| item.enabled) {
            let related = request
                .signals
                .iter()
                .filter(|signal| agent.input_signal_ids.contains(&signal.intent_id))
                .cloned()
                .collect::<Vec<_>>();

            let agent_decisions = match (&agent.kind, request.cycle_name) {
                (AgentPolicyKind::WeightedSignals, "slow") => {
                    weighted_signal_decisions::build_weighted_agent_decisions(
                        agent,
                        &related,
                        request.core_ir,
                        request.portfolio,
                        request.now_ms,
                        request.trace_id,
                    )
                }
                (AgentPolicyKind::PortfolioRebalance, "slow") => {
                    if !portfolio_rebalance::portfolio_rebalance_due(
                        agent,
                        request.last_rebalance_at_ms,
                        request.now_ms,
                    ) {
                        Vec::new()
                    } else {
                        evaluated_rebalance_agent_ids.insert(agent.agent_id.clone());
                        portfolio_rebalance::build_portfolio_rebalance_decision(
                            agent,
                            &related,
                            request.core_ir,
                            request.portfolio,
                            request.now_ms,
                            request.trace_id,
                        )
                        .into_iter()
                        .collect()
                    }
                }
                (AgentPolicyKind::CrossVenueArbitrage, "fast") => {
                    cross_venue_arbitrage::build_arb_agent_decision(
                        agent,
                        &related,
                        request.core_ir,
                        request.portfolio,
                        request.now_ms,
                        request.trace_id,
                    )
                    .into_iter()
                    .collect()
                }
                _ => Vec::new(),
            };

            for decision in agent_decisions {
                events.push(RuntimeEvent {
                    event_id: format!("evt-agent-{}-{}", decision.decision_id, request.now_ms),
                    event_type: RuntimeEventType::AgentDecisionProduced,
                    trace_id: request.trace_id.to_string(),
                    source_id: decision.agent_id.clone(),
                    ts_ms: request.now_ms,
                    payload: json!({
                        "provider_key": self.provider_key(),
                        "net_side": format!("{:?}", decision.net_side),
                        "net_strength": decision.net_strength,
                        "actions": decision.proposed_actions.len(),
                        "portfolio_targets": decision
                            .portfolio_target_decision
                            .as_ref()
                            .map(|item| item.target.target_weights.len())
                            .unwrap_or(0),
                    }),
                });
                decisions.push(decision);
            }
        }

        AgentEvaluationOutput {
            decisions,
            events,
            evaluated_rebalance_agent_ids,
        }
    }
}

#[allow(dead_code)]
fn signal_kind_for_intent(core_ir: &CoreStrategyIr, intent_id: &str) -> Option<SignalKind> {
    // v2.4.0 P2-J3: 楂橀璋冪敤璺緞, 璋冪敤鏂瑰簲棰勫厛鏋勫缓 HashMap 绱㈠紩
    // 鍗曟璋冪敤 O(N_rules) 鍙帴鍙? 浣嗗惊鐜腑閲嶅璋冪敤搴斾负 O(1)
    core_ir
        .signal_rules
        .iter()
        .find(|rule| rule.indicator_id == intent_id)
        .map(|rule| rule.signal_kind)
}

fn build_signal_kind_index(
    core_ir: &CoreStrategyIr,
) -> std::collections::HashMap<String, SignalKind> {
    core_ir
        .signal_rules
        .iter()
        .map(|rule| (rule.indicator_id.clone(), rule.signal_kind))
        .collect()
}

fn signal_score(signal: &IntentSignal) -> f64 {
    let magnitude = signal.strength.abs();
    match signal.side {
        SignalSide::Long => magnitude,
        SignalSide::Short => -magnitude,
        SignalSide::Neutral => 0.0,
    }
}

fn available_position_ratio(
    portfolio: &PortfolioState,
    exchange: &Exchange,
    symbol: &Symbol,
    reference_price: f64,
) -> f64 {
    if !reference_price.is_finite() || reference_price <= 0.0 {
        return 0.0;
    }
    let equity = portfolio_equity(portfolio).abs().max(1.0);
    let available_qty = portfolio
        .positions
        .iter()
        .find(|position| &position.exchange == exchange && &position.symbol == symbol)
        .map(|position| (position.net_qty.max(0.0) - position.frozen_qty).max(0.0))
        .unwrap_or(0.0);
    (available_qty * reference_price / equity).max(0.0)
}

fn current_position_ratio(
    portfolio: &PortfolioState,
    exchange: &Exchange,
    symbol: &Symbol,
    reference_price: f64,
) -> f64 {
    if !reference_price.is_finite() || reference_price <= 0.0 {
        return 0.0;
    }
    let equity = portfolio_equity(portfolio).abs().max(1.0);
    let current_qty = portfolio
        .positions
        .iter()
        .find(|position| &position.exchange == exchange && &position.symbol == symbol)
        .map(|position| position.net_qty.max(0.0))
        .unwrap_or(0.0);
    (current_qty * reference_price / equity).max(0.0)
}

fn portfolio_equity(portfolio: &PortfolioState) -> f64 {
    portfolio.cash_balance + portfolio.total_net_notional
}

#[cfg(test)]
mod test_harness;
