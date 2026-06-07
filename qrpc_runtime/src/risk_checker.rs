#![allow(clippy::too_many_arguments)]
use anyhow::Result;
use qrpc_core::{
    AgentDecision, CoreStrategyIr, DecisionStatus, Exchange, OrderSide, PortfolioState,
    RiskDecision, RiskDecisionMode, RiskReasonCode, RuntimeEvent, RuntimeEventType, Symbol,
};
use qrpc_core_ir::RiskPolicy;
use std::collections::{BTreeMap, BTreeSet};

mod action_clamp_helpers;
mod direction_cross_constraints;
mod event_payload_projection;
mod exposure_math_helpers;
mod portfolio_target_clamp_helpers;
use action_clamp_helpers::clamp_actions;
use direction_cross_constraints::{apply_cross_symbol_constraints, apply_direction_conflict_check};
use event_payload_projection::build_risk_event_payload;
use exposure_math_helpers::{
    available_sell_ratio, portfolio_equity, portfolio_net_exposure_ratio, symbol_net_exposure_ratio,
};
use portfolio_target_clamp_helpers::clamp_portfolio_target_limits;

#[derive(Debug, Clone)]
pub struct RiskCheckRequest<'a> {
    pub decisions: &'a [AgentDecision],
    pub core_ir: &'a CoreStrategyIr,
    pub portfolio: &'a PortfolioState,
    pub last_action_at_ms: &'a BTreeMap<String, u64>,
    pub now_ms: u64,
    pub trace_id: &'a str,
    pub mode: RiskDecisionMode,
}

#[derive(Debug, Clone)]
pub struct RiskCheckOutput {
    pub decisions: Vec<RiskDecision>,
    pub events: Vec<RuntimeEvent>,
    pub approved_agent_ids: BTreeSet<String>,
}

pub trait RiskCheckerProvider: Send + Sync {
    fn provider_key(&self) -> &'static str {
        "builtin.risk.default"
    }

    fn evaluate(&self, request: RiskCheckRequest<'_>) -> Result<RiskCheckOutput>;
}

#[derive(Debug, Clone, Default)]
pub struct RiskChecker;

impl RiskCheckerProvider for RiskChecker {
    fn evaluate(&self, request: RiskCheckRequest<'_>) -> Result<RiskCheckOutput> {
        let risks = request.core_ir.risk_policies.as_slice();
        let n = risks.len();
        let mut outputs = Vec::with_capacity(n);
        let mut events = Vec::with_capacity(n);
        let mut approved_agent_ids = BTreeSet::new();

        for risk in risks.iter().filter(|item| item.enabled) {
            for decision in request
                .decisions
                .iter()
                .filter(|item| risk.observed_agent_ids.contains(&item.agent_id))
            {
                let risk_decision = evaluate_risk_decision(
                    risk,
                    decision,
                    request.core_ir,
                    request.portfolio,
                    request.last_action_at_ms,
                    request.now_ms,
                    request.trace_id,
                    request.mode,
                );
                if matches!(
                    risk_decision.status,
                    DecisionStatus::Approve | DecisionStatus::Clamp
                ) {
                    approved_agent_ids.insert(decision.agent_id.clone());
                }
                events.push(RuntimeEvent {
                    event_id: format!(
                        "evt-risk-{}-{}",
                        risk_decision.risk_decision_id, request.now_ms
                    ),
                    event_type: RuntimeEventType::RiskDecisionProduced,
                    trace_id: request.trace_id.to_string(),
                    source_id: risk.policy_id.clone(),
                    ts_ms: request.now_ms,
                    payload: build_risk_event_payload(
                        self.provider_key(),
                        decision,
                        &risk_decision,
                        request.portfolio,
                    ),
                });
                outputs.push(risk_decision);
            }
        }

        // v2.1.0: 跨Agent方向冲突检测
        // 同一标的上有不同Agent同时提出买卖相反方向时触发
        let outputs = apply_direction_conflict_check(outputs, &mut events, request.now_ms);

        // v1.1.0: Phase 2 — 跨标的联合风控
        // 在所有标的中检查合计敞口是否超过跨标的限制
        let outputs = apply_cross_symbol_constraints(
            risks,
            outputs,
            request.portfolio,
            request.now_ms,
            self.provider_key(),
            &mut events,
        );

        Ok(RiskCheckOutput {
            decisions: outputs,
            events,
            approved_agent_ids,
        })
    }
}

fn evaluate_risk_decision(
    risk: &RiskPolicy,
    decision: &AgentDecision,
    core_ir: &CoreStrategyIr,
    portfolio: &PortfolioState,
    last_action_at_ms: &BTreeMap<String, u64>,
    now_ms: u64,
    trace_id: &str,
    mode: RiskDecisionMode,
) -> RiskDecision {
    let risk_id = risk.policy_id.clone();
    let agent_decision_id = decision.decision_id.clone();
    let symbol = decision.symbol.clone();
    let risk_decision_id = format!("risk-{}-{now_ms}", decision.decision_id);

    // 1. Mode enforcement -> return if rejected
    if let Some(rejected) = enforce_risk_mode(
        mode,
        decision,
        &risk_decision_id,
        &risk_id,
        &agent_decision_id,
        &symbol,
        now_ms,
        trace_id,
    ) {
        return rejected;
    }

    // 2. Init variables
    let mut adjusted_portfolio_target_decision = decision.portfolio_target_decision.clone();
    let mut adjusted_actions = decision.proposed_actions.clone();
    let mut reason_codes = vec![RiskReasonCode::WithinLimit];
    let mut status = DecisionStatus::Approve;
    let mut reason_text = String::from("approved");
    let equity = portfolio_equity(portfolio).abs().max(1.0);

    // 3. Action interval check -> return if too frequent
    if let Some(last_ts) = last_action_at_ms.get(&decision.agent_id) {
        if now_ms.saturating_sub(*last_ts) < risk.min_action_interval_ms {
            return RiskDecision {
                risk_decision_id: format!("risk-{}-{now_ms}", decision.decision_id),
                risk_id: risk.policy_id.clone(),
                agent_decision_id: decision.decision_id.clone(),
                symbol: decision.symbol.clone(),
                status: DecisionStatus::Reject,
                mode,
                adjusted_portfolio_target_decision: None,
                adjusted_actions: Vec::new(),
                reason_codes: vec![RiskReasonCode::ActionTooFrequent],
                reason_text: "minimum action interval not met".into(),
                produced_at_ms: now_ms,
                trace_id: trace_id.to_string(),
            };
        }
    }

    // 4. Portfolio target clamping
    if let Some(target_decision) = adjusted_portfolio_target_decision.as_mut() {
        clamp_portfolio_target_limits(
            target_decision,
            risk,
            portfolio,
            equity,
            &mut status,
            &mut reason_codes,
        );

        let execution_venue_kind = core_ir.execution.venue_kind.clone();
        reason_text = if matches!(status, DecisionStatus::Clamp) {
            format!(
                "portfolio target clamped by {:?}",
                reason_codes
                    .iter()
                    .filter(|code| !matches!(code, RiskReasonCode::WithinLimit))
                    .collect::<Vec<_>>()
            )
        } else {
            "portfolio target approved for execution diff".into()
        };
        return RiskDecision {
            risk_decision_id: format!("risk-{}-{now_ms}", decision.decision_id),
            risk_id: risk.policy_id.clone(),
            agent_decision_id: decision.decision_id.clone(),
            symbol: decision.symbol.clone(),
            status,
            mode,
            adjusted_portfolio_target_decision,
            adjusted_actions: Vec::new(),
            reason_codes,
            reason_text: format!("{reason_text} (execution_venue={execution_venue_kind})"),
            produced_at_ms: now_ms,
            trace_id: trace_id.to_string(),
        };
    }

    // 5. Action clamping
    clamp_actions(
        &mut adjusted_actions,
        risk,
        portfolio,
        equity,
        &decision.symbol,
        &mut status,
        &mut reason_codes,
        &mut reason_text,
    );

    // 6. Build result
    adjusted_actions.retain(|item| item.quantity_ratio > 0.01);
    if adjusted_actions.is_empty() {
        status = DecisionStatus::Reject;
        if !portfolio.available_cash_balance.is_finite() || portfolio.available_cash_balance <= 0.0
        {
            reason_codes = vec![RiskReasonCode::InsufficientCash];
            reason_text = "available cash exhausted".into();
        } else if reason_codes == vec![RiskReasonCode::InsufficientInventory] {
            reason_text = "spot inventory is unavailable for sell actions".into();
        } else if reason_codes != vec![RiskReasonCode::WithinLimit] {
            reason_text = "all actions resolved to zero after portfolio risk clamps".into();
        } else {
            reason_codes = vec![RiskReasonCode::InvalidAction];
            reason_text = "all actions resolved to zero after risk checks".into();
        }
    }

    let execution_venue_kind = core_ir.execution.venue_kind.clone();
    if matches!(status, DecisionStatus::Clamp) && reason_codes != vec![RiskReasonCode::WithinLimit]
    {
        reason_text = format!(
            "action list clamped by {:?}",
            reason_codes
                .iter()
                .filter(|code| !matches!(code, RiskReasonCode::WithinLimit))
                .collect::<Vec<_>>()
        );
    } else if matches!(status, DecisionStatus::Approve) {
        reason_text = "action list approved for execution".into();
    }
    RiskDecision {
        risk_decision_id: format!("risk-{}-{now_ms}", decision.decision_id),
        risk_id: risk.policy_id.clone(),
        agent_decision_id: decision.decision_id.clone(),
        symbol: decision.symbol.clone(),
        status,
        mode,
        adjusted_portfolio_target_decision: None,
        adjusted_actions,
        reason_codes,
        reason_text: format!("{reason_text} (execution_venue={execution_venue_kind})"),
        produced_at_ms: now_ms,
        trace_id: trace_id.to_string(),
    }
}

fn enforce_risk_mode(
    mode: RiskDecisionMode,
    decision: &AgentDecision,
    risk_decision_id: &str,
    risk_id: &str,
    agent_decision_id: &str,
    symbol: &Symbol,
    now_ms: u64,
    trace_id: &str,
) -> Option<RiskDecision> {
    match mode {
        RiskDecisionMode::EmergencyHalt => Some(RiskDecision {
            risk_decision_id: risk_decision_id.to_string(),
            risk_id: risk_id.to_string(),
            agent_decision_id: agent_decision_id.to_string(),
            symbol: symbol.clone(),
            status: DecisionStatus::Reject,
            mode,
            adjusted_portfolio_target_decision: None,
            adjusted_actions: Vec::new(),
            reason_codes: vec![RiskReasonCode::InvalidAction],
            reason_text: "emergency halt: all new actions rejected, open orders must be cancelled"
                .into(),
            produced_at_ms: now_ms,
            trace_id: trace_id.to_string(),
        }),
        RiskDecisionMode::FreezeOpen => {
            let has_new_open = decision
                .proposed_actions
                .iter()
                .any(|action| matches!(action.side, OrderSide::Buy | OrderSide::Sell))
                || decision
                    .portfolio_target_decision
                    .as_ref()
                    .is_some_and(|target| {
                        target
                            .target
                            .target_weights
                            .iter()
                            .any(|weight| weight.target_weight > weight.current_weight + 1e-9)
                    });
            if has_new_open {
                Some(RiskDecision {
                    risk_decision_id: risk_decision_id.to_string(),
                    risk_id: risk_id.to_string(),
                    agent_decision_id: agent_decision_id.to_string(),
                    symbol: symbol.clone(),
                    status: DecisionStatus::Reject,
                    mode,
                    adjusted_portfolio_target_decision: None,
                    adjusted_actions: Vec::new(),
                    reason_codes: vec![RiskReasonCode::ExceedNewPositionsLimit],
                    reason_text: "freeze open: new positions and additions are prohibited".into(),
                    produced_at_ms: now_ms,
                    trace_id: trace_id.to_string(),
                })
            } else {
                None
            }
        }
        RiskDecisionMode::ReduceOnly
        | RiskDecisionMode::ReconcileOnly
        | RiskDecisionMode::Normal => None,
    }
}

fn push_reason_code(reason_codes: &mut Vec<RiskReasonCode>, reason_code: RiskReasonCode) {
    if reason_codes == &[RiskReasonCode::WithinLimit] {
        reason_codes.clear();
    }
    if !reason_codes.contains(&reason_code) {
        reason_codes.push(reason_code);
    }
}

#[cfg(test)]
mod test_harness;
