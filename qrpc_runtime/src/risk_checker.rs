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
use action_clamp_helpers::clamp_actions;
use direction_cross_constraints::{apply_cross_symbol_constraints, apply_direction_conflict_check};
use event_payload_projection::build_risk_event_payload;

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

fn clamp_portfolio_target_limits(
    target_decision: &mut qrpc_core::PortfolioTargetDecision,
    risk: &RiskPolicy,
    portfolio: &PortfolioState,
    equity: f64,
    status: &mut DecisionStatus,
    reason_codes: &mut Vec<RiskReasonCode>,
) {
    let single_weight_limit = risk
        .max_single_weight
        .unwrap_or(risk.max_position_ratio)
        .min(risk.max_position_ratio)
        .clamp(0.0, 1.0);
    let total_buy_headroom = (risk.max_total_leverage - portfolio.total_leverage)
        .max(0.0)
        .min((portfolio.available_cash_balance / equity).max(0.0));
    let exchange_headroom = portfolio
        .exchange_exposures
        .iter()
        .map(|item| {
            (
                item.exchange.clone(),
                (risk.max_exchange_leverage - item.leverage).max(0.0),
            )
        })
        .collect::<BTreeMap<_, _>>();

    refresh_portfolio_target_current_weights(&mut target_decision.target.target_weights, portfolio);
    if let Some(limit) = risk.max_new_positions_per_rebalance {
        if clamp_portfolio_target_new_positions(
            &mut target_decision.target.target_weights,
            limit as usize,
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedNewPositionsLimit);
        }
    }
    if clamp_portfolio_target_single_weight(
        &mut target_decision.target.target_weights,
        single_weight_limit,
    ) {
        *status = DecisionStatus::Clamp;
        push_reason_code(reason_codes, RiskReasonCode::ExceedSingleWeight);
    }
    if let Some(max_concentration_ratio) = risk.max_concentration_ratio {
        if clamp_portfolio_target_single_weight(
            &mut target_decision.target.target_weights,
            max_concentration_ratio.clamp(0.0, 1.0),
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedConcentration);
        }
    }
    if let Some(max_symbol_net_exposure_ratio) = risk.max_symbol_net_exposure_ratio {
        if clamp_portfolio_target_single_weight(
            &mut target_decision.target.target_weights,
            max_symbol_net_exposure_ratio.clamp(0.0, 1.0),
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedSymbolNetExposure);
        }
    }
    if let Some(max_turnover) = risk.max_turnover {
        if clamp_portfolio_target_turnover(
            &mut target_decision.target.target_weights,
            max_turnover.max(0.0),
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedTurnover);
        }
    }
    if let Some(max_portfolio_net_exposure_ratio) = risk.max_portfolio_net_exposure_ratio {
        if clamp_portfolio_target_portfolio_net_exposure(
            &mut target_decision.target.target_weights,
            max_portfolio_net_exposure_ratio.max(0.0),
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedPortfolioNetExposure);
        }
    }
    if clamp_portfolio_target_total_buy_headroom(
        &mut target_decision.target.target_weights,
        total_buy_headroom,
    ) {
        *status = DecisionStatus::Clamp;
        push_reason_code(reason_codes, RiskReasonCode::ExceedTotalLeverage);
    }
    if clamp_portfolio_target_exchange_headroom(
        &mut target_decision.target.target_weights,
        &exchange_headroom,
    ) {
        *status = DecisionStatus::Clamp;
        push_reason_code(reason_codes, RiskReasonCode::ExceedExchangeLeverage);
    }
    if let Some(min_trade_weight) = risk.min_trade_weight {
        if clamp_portfolio_target_min_trade_weight(
            &mut target_decision.target.target_weights,
            min_trade_weight.max(0.0),
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::TradeBelowMinimum);
        }
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

fn refresh_portfolio_target_current_weights(
    target_weights: &mut [qrpc_core::TargetWeight],
    portfolio: &PortfolioState,
) {
    for item in target_weights {
        item.current_weight = available_sell_ratio(
            portfolio,
            &item.exchange,
            &item.symbol,
            item.reference_price,
            portfolio_equity(portfolio).abs().max(1.0),
        );
    }
}

fn clamp_portfolio_target_new_positions(
    target_weights: &mut [qrpc_core::TargetWeight],
    max_new_positions: usize,
) -> bool {
    let mut candidates = target_weights
        .iter()
        .enumerate()
        .filter(|(_, item)| item.current_weight <= 1e-9 && item.target_weight > 1e-9)
        .map(|(index, item)| {
            (
                index,
                item.signal_score.unwrap_or_default(),
                item.target_weight,
                item.symbol.as_str().to_string(),
            )
        })
        .collect::<Vec<_>>();
    if candidates.len() <= max_new_positions {
        return false;
    }
    candidates.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                right
                    .2
                    .partial_cmp(&left.2)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| left.3.cmp(&right.3))
    });
    let kept = candidates
        .into_iter()
        .take(max_new_positions)
        .map(|(index, _, _, _)| index)
        .collect::<BTreeSet<_>>();
    let mut clamped = false;
    for (index, item) in target_weights.iter_mut().enumerate() {
        if item.current_weight <= 1e-9 && item.target_weight > 1e-9 && !kept.contains(&index) {
            item.target_weight = 0.0;
            clamped = true;
        }
    }
    clamped
}

fn clamp_portfolio_target_single_weight(
    target_weights: &mut [qrpc_core::TargetWeight],
    max_single_weight: f64,
) -> bool {
    let mut clamped = false;
    for item in target_weights {
        let bounded = item.target_weight.clamp(0.0, max_single_weight);
        if (bounded - item.target_weight).abs() > 1e-9 {
            item.target_weight = bounded;
            clamped = true;
        }
    }
    clamped
}

fn clamp_portfolio_target_turnover(
    target_weights: &mut [qrpc_core::TargetWeight],
    max_turnover: f64,
) -> bool {
    let total_turnover = target_weights
        .iter()
        .map(|item| (item.target_weight - item.current_weight).abs())
        .sum::<f64>();
    if total_turnover <= max_turnover + 1e-9 {
        return false;
    }
    let scale = if !max_turnover.is_finite() || max_turnover <= 0.0 || total_turnover <= 0.0 {
        0.0
    } else {
        (max_turnover / total_turnover).min(1.0) // v2.3.0: clamp防NaN
    };
    for item in target_weights {
        let delta = item.target_weight - item.current_weight;
        item.target_weight = (item.current_weight + delta * scale).max(0.0);
    }
    true
}

fn clamp_portfolio_target_total_buy_headroom(
    target_weights: &mut [qrpc_core::TargetWeight],
    total_buy_headroom: f64,
) -> bool {
    let total_buy = target_weights
        .iter()
        .map(|item| (item.target_weight - item.current_weight).max(0.0))
        .sum::<f64>();
    if total_buy <= total_buy_headroom + 1e-9 {
        return false;
    }
    let scale = if !total_buy_headroom.is_finite() || total_buy_headroom <= 0.0 {
        0.0
    } else {
        total_buy_headroom / total_buy
    };
    for item in target_weights {
        let buy_delta = (item.target_weight - item.current_weight).max(0.0);
        if buy_delta.is_finite() && buy_delta > 0.0 {
            item.target_weight = item.current_weight + buy_delta * scale;
        }
    }
    true
}

fn clamp_portfolio_target_exchange_headroom(
    target_weights: &mut [qrpc_core::TargetWeight],
    exchange_headroom: &BTreeMap<Exchange, f64>,
) -> bool {
    let mut total_buy_by_exchange = BTreeMap::<Exchange, f64>::new();
    for item in target_weights.iter() {
        *total_buy_by_exchange
            .entry(item.exchange.clone())
            .or_default() += (item.target_weight - item.current_weight).max(0.0);
    }
    let mut scale_by_exchange = BTreeMap::<Exchange, f64>::new();
    let mut clamped = false;
    for (exchange, total_buy) in total_buy_by_exchange {
        let headroom = exchange_headroom
            .get(&exchange)
            .copied()
            .unwrap_or(f64::INFINITY);
        if total_buy > headroom + 1e-9 {
            scale_by_exchange.insert(
                exchange,
                if !headroom.is_finite() || headroom <= 0.0 {
                    0.0
                } else {
                    headroom / total_buy
                },
            );
            clamped = true;
        }
    }
    if !clamped {
        return false;
    }
    for item in target_weights {
        if let Some(scale) = scale_by_exchange.get(&item.exchange) {
            let buy_delta = (item.target_weight - item.current_weight).max(0.0);
            if buy_delta.is_finite() && buy_delta > 0.0 {
                item.target_weight = item.current_weight + buy_delta * *scale;
            }
        }
    }
    true
}

fn clamp_portfolio_target_min_trade_weight(
    target_weights: &mut [qrpc_core::TargetWeight],
    min_trade_weight: f64,
) -> bool {
    let mut clamped = false;
    for item in target_weights {
        let delta = (item.target_weight - item.current_weight).abs();
        if delta < min_trade_weight && delta > 1e-9 {
            item.target_weight = item.current_weight;
            clamped = true;
        }
    }
    clamped
}

fn clamp_portfolio_target_portfolio_net_exposure(
    target_weights: &mut [qrpc_core::TargetWeight],
    max_portfolio_net_exposure_ratio: f64,
) -> bool {
    // v2.4.0 P1-C1: 净敞口不使用 abs(), 分别计算多头和空头后取净值
    // abs() 求和会将空头转为正敞口, 导致含空头仓位的策略被不当限制
    let (long_sum, short_sum): (f64, f64) = target_weights
        .iter()
        .map(|item| item.target_weight)
        .fold((0.0, 0.0), |(long, short), w| {
            if w > 0.0 {
                (long + w, short)
            } else {
                (long, short + w.abs())
            }
        });
    let total_target_ratio = (long_sum - short_sum).abs();
    if total_target_ratio <= max_portfolio_net_exposure_ratio + 1e-9 {
        return false;
    }
    let scale = if !max_portfolio_net_exposure_ratio.is_finite()
        || max_portfolio_net_exposure_ratio <= 0.0
    {
        0.0
    } else {
        max_portfolio_net_exposure_ratio / total_target_ratio
    };
    for item in target_weights {
        item.target_weight *= scale;
    }
    true
}

fn available_sell_ratio(
    portfolio: &PortfolioState,
    exchange: &Exchange,
    symbol: &Symbol,
    reference_price: f64,
    equity: f64,
) -> f64 {
    if !reference_price.is_finite() || reference_price <= 0.0 {
        return 0.0;
    }
    let available_qty = portfolio
        .positions
        .iter()
        .find(|position| &position.exchange == exchange && &position.symbol == symbol)
        .map(|position| (position.net_qty.max(0.0) - position.frozen_qty).max(0.0))
        .unwrap_or(0.0);
    (available_qty * reference_price / equity).max(0.0)
}

fn portfolio_equity(portfolio: &PortfolioState) -> f64 {
    portfolio.cash_balance + portfolio.total_net_notional
}

fn symbol_net_exposure_ratio(portfolio: &PortfolioState, symbol: &Symbol, equity: f64) -> f64 {
    portfolio
        .positions
        .iter()
        .filter(|position| &position.symbol == symbol)
        .map(|position| position.net_qty * position.mark_price)
        .sum::<f64>()
        .abs()
        / equity
}

fn portfolio_net_exposure_ratio(portfolio: &PortfolioState, equity: f64) -> f64 {
    (portfolio.total_net_notional / equity).abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::{
        Exchange, PortfolioTarget, PortfolioTargetDecision, Position, ProposedAction, SignalSide,
        Symbol, TargetWeight,
    };
    use qrpc_core_ir::{
        CoreMetadata, CoreSourceKind, CoreStrategyIr, CoreTimeInForce, ExecutionRule,
        ExecutionSizingKind, RiskPolicy,
    };
    use std::collections::BTreeMap;

    fn sample_core_ir_with_risk(
        observed_agent_ids: Vec<&str>,
        min_action_interval_ms: u64,
    ) -> CoreStrategyIr {
        CoreStrategyIr {
            ir_version: qrpc_core::CORE_IR_V1_VERSION.to_string(),
            metadata: CoreMetadata {
                strategy_id: "risk_test".into(),
                name: "Risk Test".into(),
                source_kind: CoreSourceKind::RuntimeProtocol,
            },
            data_bindings: vec![],
            indicators: vec![],
            signal_rules: vec![],
            agent_policies: vec![],
            risk_policies: vec![RiskPolicy {
                policy_id: "risk_global".into(),
                name: "Global Risk".into(),
                observed_agent_ids: observed_agent_ids.into_iter().map(str::to_string).collect(),
                max_position_ratio: 0.3,
                max_single_weight: None,
                max_concentration_ratio: None,
                max_symbol_net_exposure_ratio: None,
                max_portfolio_net_exposure_ratio: None,
                max_turnover: None,
                min_trade_weight: None,
                max_new_positions_per_rebalance: None,
                max_total_leverage: 3.0,
                max_exchange_leverage: 3.0,
                min_action_interval_ms,
                enabled: true,
                max_cross_symbol_leverage: None,
            }],
            edges: vec![],
            execution: ExecutionRule {
                execution_id: "exec".into(),
                venue_kind: "paper".into(),
                sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
                slippage_bps: 5.0,
                taker_fee_bps: 10.0,
                total_cost_buffer_bps: 20.0,
                time_in_force: CoreTimeInForce::Gtc,
                params: BTreeMap::new(),
            },
        }
    }

    #[test]
    fn checker_rejects_actions_when_min_interval_not_met() {
        let checker = RiskChecker;
        let core_ir = sample_core_ir_with_risk(vec!["agent_1"], 1_000);
        let decision = AgentDecision {
            decision_id: "decision_1".into(),
            agent_id: "agent_1".into(),
            symbol: Symbol::BtcUsdt,
            exchange_targets: vec![Exchange::Binance],
            net_side: SignalSide::Long,
            net_strength: 0.5,
            portfolio_target_decision: None,
            proposed_actions: vec![ProposedAction {
                exchange: Exchange::Binance,
                side: qrpc_core::OrderSide::Buy,
                quantity_ratio: 0.3,
                reference_price: 50_000.0,
                strategy_tag: "test".into(),
            }],
            reason: "test".into(),
            produced_at_ms: 100,
            trace_id: "trace".into(),
        };
        let mut last_action = BTreeMap::new();
        last_action.insert("agent_1".to_string(), 900);

        let output = checker
            .evaluate(RiskCheckRequest {
                decisions: &[decision],
                core_ir: &core_ir,
                portfolio: &PortfolioState::new(100_000.0, 0),
                last_action_at_ms: &last_action,
                now_ms: 1_000,
                mode: RiskDecisionMode::Normal,
                trace_id: "trace",
            })
            .unwrap();

        assert!(matches!(output.decisions[0].status, DecisionStatus::Reject));
        assert_eq!(output.events.len(), 1);
        assert_eq!(
            output.events[0].payload["provider_key"],
            "builtin.risk.default"
        );
    }

    #[test]
    fn checker_rejects_spot_sell_without_inventory() {
        let checker = RiskChecker;
        let core_ir = sample_core_ir_with_risk(vec!["agent_1"], 0);
        let decision = AgentDecision {
            decision_id: "decision_1".into(),
            agent_id: "agent_1".into(),
            symbol: Symbol::BtcUsdt,
            exchange_targets: vec![Exchange::Binance],
            net_side: SignalSide::Short,
            net_strength: -0.5,
            portfolio_target_decision: None,
            proposed_actions: vec![ProposedAction {
                exchange: Exchange::Binance,
                side: qrpc_core::OrderSide::Sell,
                quantity_ratio: 0.2,
                reference_price: 50_000.0,
                strategy_tag: "test".into(),
            }],
            reason: "test".into(),
            produced_at_ms: 100,
            trace_id: "trace".into(),
        };
        let mut portfolio = PortfolioState::new(100_000.0, 0);
        portfolio.positions.push(Position {
            exchange: Exchange::Binance,
            symbol: Symbol::BtcUsdt,
            net_qty: 0.0,
            frozen_qty: 0.0,
            avg_entry_price: 0.0,
            mark_price: 50_000.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        });

        let output = checker
            .evaluate(RiskCheckRequest {
                decisions: &[decision],
                core_ir: &core_ir,
                portfolio: &portfolio,
                last_action_at_ms: &BTreeMap::new(),
                now_ms: 1_000,
                mode: RiskDecisionMode::Normal,
                trace_id: "trace",
            })
            .unwrap();

        assert!(matches!(output.decisions[0].status, DecisionStatus::Reject));
        assert_eq!(
            output.decisions[0].reason_codes,
            vec![RiskReasonCode::InsufficientInventory]
        );
    }

    #[test]
    fn checker_uses_agent_decision_symbol_for_inventory_checks() {
        let checker = RiskChecker;
        let core_ir = sample_core_ir_with_risk(vec!["agent_1"], 0);
        let eth = Symbol::parse("ETHUSDT");
        let decision = AgentDecision {
            decision_id: "decision_1".into(),
            agent_id: "agent_1".into(),
            symbol: eth.clone(),
            exchange_targets: vec![Exchange::Binance],
            net_side: SignalSide::Short,
            net_strength: -0.5,
            portfolio_target_decision: None,
            proposed_actions: vec![ProposedAction {
                exchange: Exchange::Binance,
                side: qrpc_core::OrderSide::Sell,
                quantity_ratio: 0.2,
                reference_price: 4_000.0,
                strategy_tag: "test".into(),
            }],
            reason: "test".into(),
            produced_at_ms: 100,
            trace_id: "trace".into(),
        };
        let mut portfolio = PortfolioState::new(100_000.0, 0);
        portfolio.positions.push(Position {
            exchange: Exchange::Binance,
            symbol: eth.clone(),
            net_qty: 10.0,
            frozen_qty: 0.0,
            avg_entry_price: 4_000.0,
            mark_price: 4_000.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        });

        let output = checker
            .evaluate(RiskCheckRequest {
                decisions: &[decision],
                core_ir: &core_ir,
                portfolio: &portfolio,
                last_action_at_ms: &BTreeMap::new(),
                now_ms: 1_000,
                mode: RiskDecisionMode::Normal,
                trace_id: "trace",
            })
            .unwrap();

        assert!(matches!(
            output.decisions[0].status,
            DecisionStatus::Approve
        ));
        assert_eq!(output.decisions[0].symbol, eth);
    }

    #[test]
    fn checker_preserves_portfolio_target_decisions_for_execution_diff() {
        let checker = RiskChecker;
        let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
        core_ir.risk_policies[0].max_position_ratio = 1.0;
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let decision = AgentDecision {
            decision_id: "decision_rebalance".into(),
            agent_id: "agent_rebalance".into(),
            symbol: btc.clone(),
            exchange_targets: vec![Exchange::Binance],
            net_side: SignalSide::Neutral,
            net_strength: 0.0,
            portfolio_target_decision: Some(PortfolioTargetDecision {
                target_id: "target_rebalance".into(),
                target: PortfolioTarget {
                    allocation_kind: "equal_weight".into(),
                    target_weights: vec![
                        TargetWeight {
                            exchange: Exchange::Binance,
                            symbol: btc.clone(),
                            target_weight: 0.5,
                            current_weight: 0.7,
                            reference_price: 50_000.0,
                            signal_score: Some(0.9),
                        },
                        TargetWeight {
                            exchange: Exchange::Binance,
                            symbol: eth.clone(),
                            target_weight: 0.5,
                            current_weight: 0.0,
                            reference_price: 4_000.0,
                            signal_score: Some(0.8),
                        },
                    ],
                },
                reason: "equal weight".into(),
            }),
            proposed_actions: Vec::new(),
            reason: "rebalance".into(),
            produced_at_ms: 100,
            trace_id: "trace".into(),
        };

        let output = checker
            .evaluate(RiskCheckRequest {
                decisions: &[decision],
                core_ir: &core_ir,
                portfolio: &PortfolioState::new(100_000.0, 0),
                last_action_at_ms: &BTreeMap::new(),
                now_ms: 1_000,
                mode: RiskDecisionMode::Normal,
                trace_id: "trace",
            })
            .unwrap();

        assert!(matches!(
            output.decisions[0].status,
            DecisionStatus::Approve
        ));
        assert!(output.decisions[0].adjusted_actions.is_empty());
        let target = output.decisions[0]
            .adjusted_portfolio_target_decision
            .as_ref()
            .expect("adjusted portfolio target");
        assert_eq!(target.target.target_weights.len(), 2);
        assert_eq!(target.target.target_weights[0].symbol, btc);
        assert_eq!(target.target.target_weights[1].symbol, eth);
    }

    #[test]
    fn checker_clamps_portfolio_target_to_max_single_weight() {
        let checker = RiskChecker;
        let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
        core_ir.risk_policies[0].max_single_weight = Some(0.3);
        let btc = Symbol::BtcUsdt;
        let decision = AgentDecision {
            decision_id: "decision_rebalance".into(),
            agent_id: "agent_rebalance".into(),
            symbol: btc.clone(),
            exchange_targets: vec![Exchange::Binance],
            net_side: SignalSide::Neutral,
            net_strength: 0.0,
            portfolio_target_decision: Some(PortfolioTargetDecision {
                target_id: "target_rebalance".into(),
                target: PortfolioTarget {
                    allocation_kind: "fixed".into(),
                    target_weights: vec![TargetWeight {
                        exchange: Exchange::Binance,
                        symbol: btc.clone(),
                        target_weight: 0.8,
                        current_weight: 0.0,
                        reference_price: 50_000.0,
                        signal_score: Some(1.0),
                    }],
                },
                reason: "fixed weight".into(),
            }),
            proposed_actions: Vec::new(),
            reason: "rebalance".into(),
            produced_at_ms: 100,
            trace_id: "trace".into(),
        };

        let output = checker
            .evaluate(RiskCheckRequest {
                decisions: &[decision],
                core_ir: &core_ir,
                portfolio: &PortfolioState::new(100_000.0, 0),
                last_action_at_ms: &BTreeMap::new(),
                now_ms: 1_000,
                mode: RiskDecisionMode::Normal,
                trace_id: "trace",
            })
            .unwrap();

        assert!(matches!(output.decisions[0].status, DecisionStatus::Clamp));
        assert_eq!(
            output.decisions[0].reason_codes,
            vec![RiskReasonCode::ExceedSingleWeight]
        );
        let target = output.decisions[0]
            .adjusted_portfolio_target_decision
            .as_ref()
            .expect("adjusted portfolio target");
        assert!((target.target.target_weights[0].target_weight - 0.3).abs() < 1e-9);
    }

    #[test]
    fn checker_scales_portfolio_target_to_max_turnover() {
        let checker = RiskChecker;
        let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
        core_ir.risk_policies[0].max_position_ratio = 1.0;
        core_ir.risk_policies[0].max_turnover = Some(0.35);
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let mut portfolio = PortfolioState::new(100_000.0, 0);
        portfolio.positions.push(Position {
            exchange: Exchange::Binance,
            symbol: btc.clone(),
            net_qty: 1.4,
            frozen_qty: 0.0,
            avg_entry_price: 50_000.0,
            mark_price: 50_000.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        });
        portfolio.cash_balance = 30_000.0;
        portfolio.available_cash_balance = 30_000.0;
        portfolio.total_net_notional = 70_000.0;
        portfolio.total_gross_notional = 70_000.0;
        portfolio.total_leverage = 0.7;

        let decision = AgentDecision {
            decision_id: "decision_rebalance".into(),
            agent_id: "agent_rebalance".into(),
            symbol: btc.clone(),
            exchange_targets: vec![Exchange::Binance],
            net_side: SignalSide::Neutral,
            net_strength: 0.0,
            portfolio_target_decision: Some(PortfolioTargetDecision {
                target_id: "target_rebalance".into(),
                target: PortfolioTarget {
                    allocation_kind: "equal_weight".into(),
                    target_weights: vec![
                        TargetWeight {
                            exchange: Exchange::Binance,
                            symbol: btc.clone(),
                            target_weight: 0.5,
                            current_weight: 0.7,
                            reference_price: 50_000.0,
                            signal_score: Some(0.9),
                        },
                        TargetWeight {
                            exchange: Exchange::Binance,
                            symbol: eth.clone(),
                            target_weight: 0.5,
                            current_weight: 0.0,
                            reference_price: 4_000.0,
                            signal_score: Some(0.8),
                        },
                    ],
                },
                reason: "equal weight".into(),
            }),
            proposed_actions: Vec::new(),
            reason: "rebalance".into(),
            produced_at_ms: 100,
            trace_id: "trace".into(),
        };

        let output = checker
            .evaluate(RiskCheckRequest {
                decisions: &[decision],
                core_ir: &core_ir,
                portfolio: &portfolio,
                last_action_at_ms: &BTreeMap::new(),
                now_ms: 1_000,
                mode: RiskDecisionMode::Normal,
                trace_id: "trace",
            })
            .unwrap();

        assert!(matches!(output.decisions[0].status, DecisionStatus::Clamp));
        assert_eq!(
            output.decisions[0].reason_codes,
            vec![RiskReasonCode::ExceedTurnover]
        );
        let target = output.decisions[0]
            .adjusted_portfolio_target_decision
            .as_ref()
            .expect("adjusted portfolio target");
        let btc_target = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == btc)
            .expect("btc");
        let eth_target = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == eth)
            .expect("eth");
        assert!((btc_target.target_weight - 0.6).abs() < 1e-9);
        assert!((eth_target.target_weight - 0.25).abs() < 1e-9);
    }

    #[test]
    fn checker_removes_small_portfolio_target_trades() {
        let checker = RiskChecker;
        let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
        core_ir.risk_policies[0].max_position_ratio = 1.0;
        core_ir.risk_policies[0].min_trade_weight = Some(0.02);
        let btc = Symbol::BtcUsdt;
        let decision = AgentDecision {
            decision_id: "decision_rebalance".into(),
            agent_id: "agent_rebalance".into(),
            symbol: btc.clone(),
            exchange_targets: vec![Exchange::Binance],
            net_side: SignalSide::Neutral,
            net_strength: 0.0,
            portfolio_target_decision: Some(PortfolioTargetDecision {
                target_id: "target_rebalance".into(),
                target: PortfolioTarget {
                    allocation_kind: "fixed".into(),
                    target_weights: vec![TargetWeight {
                        exchange: Exchange::Binance,
                        symbol: btc.clone(),
                        target_weight: 0.515,
                        current_weight: 0.5,
                        reference_price: 50_000.0,
                        signal_score: Some(1.0),
                    }],
                },
                reason: "small rebalance".into(),
            }),
            proposed_actions: Vec::new(),
            reason: "rebalance".into(),
            produced_at_ms: 100,
            trace_id: "trace".into(),
        };
        let mut portfolio = PortfolioState::new(100_000.0, 0);
        portfolio.positions.push(Position {
            exchange: Exchange::Binance,
            symbol: btc.clone(),
            net_qty: 1.0,
            frozen_qty: 0.0,
            avg_entry_price: 50_000.0,
            mark_price: 50_000.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        });
        portfolio.cash_balance = 50_000.0;
        portfolio.available_cash_balance = 50_000.0;
        portfolio.total_net_notional = 50_000.0;
        portfolio.total_gross_notional = 50_000.0;
        portfolio.total_leverage = 0.5;

        let output = checker
            .evaluate(RiskCheckRequest {
                decisions: &[decision],
                core_ir: &core_ir,
                portfolio: &portfolio,
                last_action_at_ms: &BTreeMap::new(),
                now_ms: 1_000,
                mode: RiskDecisionMode::Normal,
                trace_id: "trace",
            })
            .unwrap();

        assert!(matches!(output.decisions[0].status, DecisionStatus::Clamp));
        assert_eq!(
            output.decisions[0].reason_codes,
            vec![RiskReasonCode::TradeBelowMinimum]
        );
        let target = output.decisions[0]
            .adjusted_portfolio_target_decision
            .as_ref()
            .expect("adjusted portfolio target");
        assert!((target.target.target_weights[0].target_weight - 0.5).abs() < 1e-9);
    }

    #[test]
    fn checker_limits_new_positions_per_rebalance() {
        let checker = RiskChecker;
        let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
        core_ir.risk_policies[0].max_position_ratio = 1.0;
        core_ir.risk_policies[0].max_new_positions_per_rebalance = Some(1);
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let sol = Symbol::parse("SOLUSDT");
        let decision = AgentDecision {
            decision_id: "decision_rebalance".into(),
            agent_id: "agent_rebalance".into(),
            symbol: btc.clone(),
            exchange_targets: vec![Exchange::Binance],
            net_side: SignalSide::Neutral,
            net_strength: 0.0,
            portfolio_target_decision: Some(PortfolioTargetDecision {
                target_id: "target_rebalance".into(),
                target: PortfolioTarget {
                    allocation_kind: "rank_weight".into(),
                    target_weights: vec![
                        TargetWeight {
                            exchange: Exchange::Binance,
                            symbol: btc.clone(),
                            target_weight: 0.3,
                            current_weight: 0.0,
                            reference_price: 50_000.0,
                            signal_score: Some(0.95),
                        },
                        TargetWeight {
                            exchange: Exchange::Binance,
                            symbol: eth.clone(),
                            target_weight: 0.2,
                            current_weight: 0.0,
                            reference_price: 4_000.0,
                            signal_score: Some(0.7),
                        },
                        TargetWeight {
                            exchange: Exchange::Binance,
                            symbol: sol.clone(),
                            target_weight: 0.1,
                            current_weight: 0.0,
                            reference_price: 150.0,
                            signal_score: Some(0.6),
                        },
                    ],
                },
                reason: "limit new names".into(),
            }),
            proposed_actions: Vec::new(),
            reason: "rebalance".into(),
            produced_at_ms: 100,
            trace_id: "trace".into(),
        };

        let output = checker
            .evaluate(RiskCheckRequest {
                decisions: &[decision],
                core_ir: &core_ir,
                portfolio: &PortfolioState::new(100_000.0, 0),
                last_action_at_ms: &BTreeMap::new(),
                now_ms: 1_000,
                mode: RiskDecisionMode::Normal,
                trace_id: "trace",
            })
            .unwrap();

        assert!(matches!(output.decisions[0].status, DecisionStatus::Clamp));
        assert_eq!(
            output.decisions[0].reason_codes,
            vec![RiskReasonCode::ExceedNewPositionsLimit]
        );
        let target = output.decisions[0]
            .adjusted_portfolio_target_decision
            .as_ref()
            .expect("adjusted portfolio target");
        let btc_target = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == btc)
            .expect("btc");
        let eth_target = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == eth)
            .expect("eth");
        let sol_target = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == sol)
            .expect("sol");
        assert!((btc_target.target_weight - 0.3).abs() < 1e-9);
        assert!((eth_target.target_weight - 0.0).abs() < 1e-9);
        assert!((sol_target.target_weight - 0.0).abs() < 1e-9);
    }

    #[test]
    fn checker_limits_portfolio_target_to_portfolio_net_exposure() {
        let checker = RiskChecker;
        let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
        core_ir.risk_policies[0].max_position_ratio = 1.0;
        core_ir.risk_policies[0].max_portfolio_net_exposure_ratio = Some(0.6);
        let decision = AgentDecision {
            decision_id: "decision_rebalance".into(),
            agent_id: "agent_rebalance".into(),
            symbol: Symbol::BtcUsdt,
            exchange_targets: vec![Exchange::Binance],
            net_side: SignalSide::Neutral,
            net_strength: 0.0,
            portfolio_target_decision: Some(PortfolioTargetDecision {
                target_id: "target_rebalance".into(),
                target: PortfolioTarget {
                    allocation_kind: "fixed_weights".into(),
                    target_weights: vec![
                        TargetWeight {
                            exchange: Exchange::Binance,
                            symbol: Symbol::BtcUsdt,
                            target_weight: 0.4,
                            current_weight: 0.0,
                            reference_price: 50_000.0,
                            signal_score: Some(0.8),
                        },
                        TargetWeight {
                            exchange: Exchange::Binance,
                            symbol: Symbol::parse("ETHUSDT"),
                            target_weight: 0.3,
                            current_weight: 0.0,
                            reference_price: 3_000.0,
                            signal_score: Some(0.6),
                        },
                    ],
                },
                reason: "rebalance".into(),
            }),
            proposed_actions: vec![],
            reason: "rebalance".into(),
            produced_at_ms: 0,
            trace_id: "trace".into(),
        };

        let output = checker
            .evaluate(RiskCheckRequest {
                decisions: &[decision],
                core_ir: &core_ir,
                portfolio: &PortfolioState::new(100_000.0, 0),
                last_action_at_ms: &BTreeMap::new(),
                now_ms: 1_000,
                mode: RiskDecisionMode::Normal,
                trace_id: "trace",
            })
            .expect("risk evaluation");

        assert_eq!(output.decisions[0].status, DecisionStatus::Clamp);
        assert_eq!(
            output.decisions[0].reason_codes,
            vec![RiskReasonCode::ExceedPortfolioNetExposure]
        );
        let target = output.decisions[0]
            .adjusted_portfolio_target_decision
            .as_ref()
            .expect("adjusted portfolio target");
        assert!((target.target.target_weights[0].target_weight - 0.342857142857).abs() < 1e-6);
        assert!((target.target.target_weights[1].target_weight - 0.257142857142).abs() < 1e-6);
    }

    #[test]
    fn checker_limits_action_list_to_symbol_net_exposure() {
        let checker = RiskChecker;
        let mut core_ir = sample_core_ir_with_risk(vec!["agent_1"], 0);
        core_ir.risk_policies[0].max_position_ratio = 1.0;
        core_ir.risk_policies[0].max_symbol_net_exposure_ratio = Some(0.25);

        let mut portfolio = PortfolioState::new(100_000.0, 0);
        portfolio.positions.push(Position {
            exchange: Exchange::Binance,
            symbol: Symbol::BtcUsdt,
            net_qty: 0.4,
            frozen_qty: 0.0,
            avg_entry_price: 50_000.0,
            mark_price: 50_000.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        });
        portfolio.total_net_notional = 20_000.0;

        let decision = AgentDecision {
            decision_id: "decision_1".into(),
            agent_id: "agent_1".into(),
            symbol: Symbol::BtcUsdt,
            exchange_targets: vec![Exchange::Binance, Exchange::Okx],
            net_side: SignalSide::Long,
            net_strength: 0.5,
            portfolio_target_decision: None,
            proposed_actions: vec![
                ProposedAction {
                    exchange: Exchange::Binance,
                    side: qrpc_core::OrderSide::Buy,
                    quantity_ratio: 0.2,
                    reference_price: 50_000.0,
                    strategy_tag: "test".into(),
                },
                ProposedAction {
                    exchange: Exchange::Okx,
                    side: qrpc_core::OrderSide::Buy,
                    quantity_ratio: 0.2,
                    reference_price: 50_000.0,
                    strategy_tag: "test".into(),
                },
            ],
            reason: "test".into(),
            produced_at_ms: 100,
            trace_id: "trace".into(),
        };

        let output = checker
            .evaluate(RiskCheckRequest {
                decisions: &[decision],
                core_ir: &core_ir,
                portfolio: &portfolio,
                last_action_at_ms: &BTreeMap::new(),
                now_ms: 1_000,
                mode: RiskDecisionMode::Normal,
                trace_id: "trace",
            })
            .expect("risk evaluation");

        assert_eq!(output.decisions[0].status, DecisionStatus::Clamp);
        assert!(output.decisions[0]
            .reason_codes
            .contains(&RiskReasonCode::ExceedSymbolNetExposure));
        let total_buy = output.decisions[0]
            .adjusted_actions
            .iter()
            .map(|action| action.quantity_ratio)
            .sum::<f64>();
        assert!((total_buy - (0.25 - 20_000.0 / 120_000.0)).abs() < 1e-9);
    }

    #[test]
    fn risk_events_include_explanation_and_pre_post_sizing() {
        let checker = RiskChecker;
        let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
        core_ir.risk_policies[0].max_single_weight = Some(0.3);
        let btc = Symbol::BtcUsdt;
        let decision = AgentDecision {
            decision_id: "decision_rebalance".into(),
            agent_id: "agent_rebalance".into(),
            symbol: btc.clone(),
            exchange_targets: vec![Exchange::Binance],
            net_side: SignalSide::Neutral,
            net_strength: 0.0,
            portfolio_target_decision: Some(PortfolioTargetDecision {
                target_id: "target_rebalance".into(),
                target: PortfolioTarget {
                    allocation_kind: "fixed".into(),
                    target_weights: vec![TargetWeight {
                        exchange: Exchange::Binance,
                        symbol: btc,
                        target_weight: 0.8,
                        current_weight: 0.0,
                        reference_price: 50_000.0,
                        signal_score: Some(1.0),
                    }],
                },
                reason: "fixed weight".into(),
            }),
            proposed_actions: Vec::new(),
            reason: "rebalance".into(),
            produced_at_ms: 100,
            trace_id: "trace".into(),
        };

        let output = checker
            .evaluate(RiskCheckRequest {
                decisions: &[decision],
                core_ir: &core_ir,
                portfolio: &PortfolioState::new(100_000.0, 0),
                last_action_at_ms: &BTreeMap::new(),
                now_ms: 1_000,
                mode: RiskDecisionMode::Normal,
                trace_id: "trace",
            })
            .unwrap();

        let payload = &output.events[0].payload;
        assert_eq!(
            payload["reason_text"],
            "portfolio target clamped by [ExceedSingleWeight] (execution_venue=paper)"
        );
        assert_eq!(payload["limit_triggered"], "max_single_weight");
        assert_eq!(payload["sizing_mode"], "portfolio_target");
        assert_eq!(payload["pre_risk"]["max_target_weight"], 0.8);
        assert_eq!(payload["post_risk"]["max_target_weight"], 0.3);
        assert_eq!(
            payload["explanation_summary"],
            "Risk clamped sizing after triggering max_single_weight."
        );
    }
}
