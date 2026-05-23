#![allow(clippy::too_many_arguments)]
use anyhow::Result;
use qrpc_core::{
    AgentDecision, CoreStrategyIr, DecisionStatus, Exchange, OrderSide, PortfolioState,
    RiskDecision, RiskDecisionMode, RiskReasonCode, RuntimeEvent, RuntimeEventType, Symbol,
};
use qrpc_core_ir::RiskPolicy;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

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

/// v2.1.0: 跨Agent方向冲突检测
/// 同一标的有多个Agent同时发出买卖相反方向→DirectionConflict拒绝
fn apply_direction_conflict_check(
    mut decisions: Vec<RiskDecision>,
    events: &mut Vec<RuntimeEvent>,
    now_ms: u64,
) -> Vec<RiskDecision> {
    use std::collections::HashMap;
    // 按标的收集所有已批准的决策
    let mut by_symbol: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, d) in decisions.iter().enumerate() {
        if matches!(d.status, DecisionStatus::Approve | DecisionStatus::Clamp) {
            by_symbol
                .entry(format!("{:?}", d.symbol))
                .or_default()
                .push(i);
        }
    }
    // 对每个标的有≥2个不同Agent的决策，检查方向冲突
    for indices in by_symbol.values() {
        if indices.len() < 2 {
            continue;
        }
        let has_buy = indices.iter().any(|&i| {
            decisions[i]
                .adjusted_actions
                .iter()
                .any(|a| matches!(a.side, OrderSide::Buy))
        });
        let has_sell = indices.iter().any(|&i| {
            decisions[i]
                .adjusted_actions
                .iter()
                .any(|a| matches!(a.side, OrderSide::Sell))
        });
        if has_buy && has_sell {
            for &i in indices {
                decisions[i].status = DecisionStatus::Reject;
                decisions[i].reason_codes = vec![RiskReasonCode::DirectionConflict];
                decisions[i].reason_text =
                    "方向冲突: 同一标的上存在多个Agent的相反方向操作，已全部拒绝".to_string();
                decisions[i].adjusted_actions.clear();
                decisions[i].adjusted_portfolio_target_decision = None;
                events.push(RuntimeEvent {
                    event_id: format!(
                        "evt-risk-direction-conflict-{}-{}",
                        decisions[i].risk_decision_id, now_ms
                    ),
                    event_type: RuntimeEventType::RiskDecisionProduced,
                    trace_id: decisions[i].trace_id.clone(),
                    source_id: decisions[i].risk_id.clone(),
                    ts_ms: now_ms,
                    payload: serde_json::json!({
                        "risk_decision_id": decisions[i].risk_decision_id,
                        "symbol": decisions[i].symbol,
                        "reason": "direction_conflict",
                        "explanation": "同一标的多Agent方向冲突，为避免相互踩踏全部拒绝"
                    }),
                });
            }
        }
    }
    decisions
}

/// v1.1.0: Phase 2 跨标的联合约束检查
fn apply_cross_symbol_constraints(
    risks: &[qrpc_core_ir::RiskPolicy],
    mut decisions: Vec<RiskDecision>,
    portfolio: &PortfolioState,
    now_ms: u64,
    provider_key: &str,
    events: &mut Vec<RuntimeEvent>,
) -> Vec<RiskDecision> {
    for risk in risks.iter().filter(|r| r.enabled) {
        let cross_leverage_limit = risk.max_cross_symbol_leverage.unwrap_or(f64::MAX);

        if cross_leverage_limit >= f64::MAX {
            continue; // 无跨标的约束配置
        }

        let equity = portfolio_equity(portfolio).max(1.0);

        // 计算所有已批准/clamped 决策的合计敞口
        // v2.0.1: quantity_ratio 已经是 equity 的比例 (如 0.2 = 20%)，
        // 所以名义本金 = equity * quantity_ratio，无需再乘价格。
        let total_notional: f64 = decisions
            .iter()
            .filter(|d| !matches!(d.status, DecisionStatus::Reject))
            .map(|d| {
                d.adjusted_actions
                    .iter()
                    .map(|a| a.quantity_ratio * equity)
                    .sum::<f64>()
            })
            .sum();

        let cross_symbol_leverage = total_notional / equity;

        if cross_symbol_leverage > cross_leverage_limit && cross_leverage_limit.is_finite() {
            // 按比例缩减所有非 Reject 的 decision
            let scale = cross_leverage_limit / cross_symbol_leverage;
            for d in decisions
                .iter_mut()
                .filter(|d| !matches!(d.status, DecisionStatus::Reject))
            {
                for action in &mut d.adjusted_actions {
                    action.quantity_ratio *= scale;
                }
                if let Some(ref mut target) = d.adjusted_portfolio_target_decision {
                    for tw in &mut target.target.target_weights {
                        tw.target_weight =
                            tw.current_weight + (tw.target_weight - tw.current_weight) * scale;
                    }
                }
                // 标记为 Clamp 如果不是 Approve
                if matches!(d.status, DecisionStatus::Approve) {
                    d.status = DecisionStatus::Clamp;
                }
                d.reason_codes
                    .push(RiskReasonCode::ExceedPortfolioNetExposure);
                d.reason_text.push_str(&format!(
                    " 跨标的杠杆 {:.2} 超过限制 {:.2}, 缩减至 {:.0}%",
                    cross_symbol_leverage,
                    cross_leverage_limit,
                    scale * 100.0,
                ));
            }
            events.push(RuntimeEvent {
                event_id: format!("evt-risk-cross-symbol-{now_ms}"),
                event_type: RuntimeEventType::RiskDecisionProduced,
                trace_id: "cross_symbol".to_string(),
                source_id: risk.policy_id.clone(),
                ts_ms: now_ms,
                payload: serde_json::json!({
                    "provider_key": provider_key,
                    "status": "clamped_cross_symbol",
                    "cross_symbol_leverage": cross_symbol_leverage,
                    "limit": cross_leverage_limit,
                    "scale": scale,
                    "reason_text": format!("跨标的合计杠杆 {:.2} 超过限制 {:.2}", cross_symbol_leverage, cross_leverage_limit),
                }),
            });
        }
    }
    decisions
}

fn build_risk_event_payload(
    provider_key: &str,
    decision: &AgentDecision,
    risk_decision: &RiskDecision,
    portfolio: &PortfolioState,
) -> Value {
    let reason_codes = risk_decision
        .reason_codes
        .iter()
        .map(reason_code_name)
        .collect::<Vec<_>>();
    let limit_triggered = risk_decision
        .reason_codes
        .iter()
        .find(|code| !matches!(code, RiskReasonCode::WithinLimit))
        .map(reason_code_name);

    let payload = if let Some(original_target) = &decision.portfolio_target_decision {
        let adjusted_target = risk_decision
            .adjusted_portfolio_target_decision
            .as_ref()
            .map(|item| &item.target.target_weights[..])
            .unwrap_or(&[]);
        json!({
            "sizing_mode": "portfolio_target",
            "pre_risk": target_weight_stats(&original_target.target.target_weights),
            "post_risk": target_weight_stats(adjusted_target),
        })
    } else {
        let equity = portfolio_equity(portfolio).abs().max(1.0);
        json!({
            "sizing_mode": "action_list",
            "pre_risk": action_stats(&decision.proposed_actions, portfolio, &decision.symbol, equity),
            "post_risk": action_stats(&risk_decision.adjusted_actions, portfolio, &decision.symbol, equity),
        })
    };

    let mut object = json!({
        "provider_key": provider_key,
        "status": format!("{:?}", risk_decision.status),
        "reasons": reason_codes,
        "reason_text": risk_decision.reason_text,
        "limit_triggered": limit_triggered,
        "explanation_summary": risk_explanation_summary(risk_decision),
        "agent_decision_id": decision.decision_id,
        "risk_id": risk_decision.risk_id,
    });

    if let (Value::Object(object_map), Value::Object(payload_map)) = (&mut object, payload) {
        object_map.extend(payload_map);
    }

    object
}

fn risk_explanation_summary(risk_decision: &RiskDecision) -> String {
    let limit = risk_decision
        .reason_codes
        .iter()
        .find(|code| !matches!(code, RiskReasonCode::WithinLimit))
        .map(reason_code_name);
    match (&risk_decision.status, limit) {
        (DecisionStatus::Approve, _) => "Risk approved without clamp.".to_string(),
        (DecisionStatus::Clamp, Some(limit)) => {
            format!("Risk clamped sizing after triggering {limit}.")
        }
        (DecisionStatus::Reject, Some(limit)) => {
            format!("Risk rejected the decision because {limit} triggered.")
        }
        (DecisionStatus::Clamp, None) => "Risk clamped sizing before execution.".to_string(),
        (DecisionStatus::Reject, None) => "Risk rejected the decision.".to_string(),
    }
}

fn reason_code_name(reason_code: &RiskReasonCode) -> &'static str {
    match reason_code {
        RiskReasonCode::WithinLimit => "within_limit",
        RiskReasonCode::ExceedTotalLeverage => "max_total_leverage",
        RiskReasonCode::ExceedExchangeLeverage => "max_exchange_leverage",
        RiskReasonCode::ExceedSingleWeight => "max_single_weight",
        RiskReasonCode::ExceedConcentration => "max_concentration_ratio",
        RiskReasonCode::ExceedSymbolNetExposure => "max_symbol_net_exposure_ratio",
        RiskReasonCode::ExceedPortfolioNetExposure => "max_portfolio_net_exposure_ratio",
        RiskReasonCode::ExceedTurnover => "max_turnover",
        RiskReasonCode::TradeBelowMinimum => "min_trade_weight",
        RiskReasonCode::ExceedNewPositionsLimit => "max_new_positions_per_rebalance",
        RiskReasonCode::ActionTooFrequent => "min_action_interval_ms",
        RiskReasonCode::DirectionConflict => "direction_conflict",
        RiskReasonCode::InsufficientCash => "available_cash_balance",
        RiskReasonCode::InsufficientInventory => "available_inventory",
        RiskReasonCode::CostNotCovered => "cost_not_covered",
        RiskReasonCode::InvalidAction => "invalid_action",
        RiskReasonCode::ExceedDailyLoss => "max_daily_loss",
        RiskReasonCode::ExceedDrawdown => "max_drawdown",
    }
}

fn action_stats(
    actions: &[qrpc_core::ProposedAction],
    portfolio: &PortfolioState,
    symbol: &Symbol,
    equity: f64,
) -> Value {
    let total_quantity_ratio = actions.iter().map(|item| item.quantity_ratio).sum::<f64>();
    let buy_quantity_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Buy))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    let sell_quantity_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Sell))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    let portfolio_net_exposure_ratio =
        portfolio_net_exposure_ratio(portfolio, equity) + buy_quantity_ratio - sell_quantity_ratio;
    let symbol_net_exposure_ratio = symbol_net_exposure_ratio(portfolio, symbol, equity)
        + buy_quantity_ratio
        - sell_quantity_ratio;

    json!({
        "action_count": actions.len(),
        "total_quantity_ratio": total_quantity_ratio,
        "buy_quantity_ratio": buy_quantity_ratio,
        "sell_quantity_ratio": sell_quantity_ratio,
        "portfolio_net_exposure_ratio": portfolio_net_exposure_ratio.max(0.0),
        "symbol_net_exposure_ratio": symbol_net_exposure_ratio.max(0.0),
    })
}

fn target_weight_stats(target_weights: &[qrpc_core::TargetWeight]) -> Value {
    let turnover_ratio = target_weights
        .iter()
        .map(|item| (item.target_weight - item.current_weight).abs())
        .sum::<f64>();
    let max_target_weight = target_weights
        .iter()
        .map(|item| item.target_weight.abs())
        .fold(0.0_f64, f64::max);
    let portfolio_net_exposure_ratio = target_weights
        .iter()
        .map(|item| item.target_weight.abs())
        .sum::<f64>();
    let new_positions = target_weights
        .iter()
        .filter(|item| item.current_weight <= 1e-9 && item.target_weight > 1e-9)
        .count();

    json!({
        "basket_members": target_weights.len(),
        "turnover_ratio": turnover_ratio,
        "max_target_weight": max_target_weight,
        "concentration_ratio": max_target_weight,
        "max_symbol_net_exposure_ratio": max_target_weight,
        "portfolio_net_exposure_ratio": portfolio_net_exposure_ratio,
        "new_positions": new_positions,
    })
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

fn clamp_actions(
    actions: &mut [qrpc_core::ProposedAction],
    risk: &RiskPolicy,
    portfolio: &PortfolioState,
    equity: f64,
    symbol: &Symbol,
    status: &mut DecisionStatus,
    reason_codes: &mut Vec<RiskReasonCode>,
    reason_text: &mut String,
) {
    for action in actions.iter_mut() {
        action.quantity_ratio = action.quantity_ratio.min(risk.max_position_ratio).max(0.0);
    }
    if clamp_sell_actions_to_inventory(actions, portfolio, symbol, equity) {
        *status = DecisionStatus::Clamp;
        push_reason_code(reason_codes, RiskReasonCode::InsufficientInventory);
        *reason_text = "sell actions clamped to available spot inventory".into();
    }

    if let Some(max_symbol_net_exposure_ratio) = risk.max_symbol_net_exposure_ratio {
        if clamp_buy_actions_to_symbol_net_exposure(
            actions,
            portfolio,
            symbol,
            max_symbol_net_exposure_ratio.max(0.0),
            equity,
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedSymbolNetExposure);
        }
    }

    let total_buy_headroom = (risk.max_total_leverage - portfolio.total_leverage)
        .max(0.0)
        .min((portfolio.available_cash_balance / equity).max(0.0));
    if clamp_buy_actions_to_total_headroom(actions, total_buy_headroom) {
        *status = DecisionStatus::Clamp;
        push_reason_code(reason_codes, RiskReasonCode::ExceedTotalLeverage);
        *reason_text = "buy actions clamped to total leverage or cash headroom".into();
    }

    if let Some(max_portfolio_net_exposure_ratio) = risk.max_portfolio_net_exposure_ratio {
        if clamp_buy_actions_to_portfolio_net_exposure(
            actions,
            portfolio,
            max_portfolio_net_exposure_ratio.max(0.0),
            equity,
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedPortfolioNetExposure);
        }
    }

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
    if clamp_buy_actions_to_exchange_headroom(actions, &exchange_headroom) {
        *status = DecisionStatus::Clamp;
        push_reason_code(reason_codes, RiskReasonCode::ExceedExchangeLeverage);
        *reason_text = "buy actions clamped to exchange leverage headroom".into();
    }
}

fn clamp_sell_actions_to_inventory(
    actions: &mut [qrpc_core::ProposedAction],
    portfolio: &PortfolioState,
    symbol: &Symbol,
    equity: f64,
) -> bool {
    let mut used_ratio_by_exchange = BTreeMap::<Exchange, f64>::new();
    let mut clamped = false;

    for action in actions
        .iter_mut()
        .filter(|item| matches!(item.side, OrderSide::Sell))
    {
        let available_ratio = available_sell_ratio(
            portfolio,
            &action.exchange,
            symbol,
            action.reference_price,
            equity,
        );
        let consumed = used_ratio_by_exchange
            .get(&action.exchange)
            .copied()
            .unwrap_or_default();
        let remaining = (available_ratio - consumed).max(0.0);
        if action.quantity_ratio > remaining {
            action.quantity_ratio = remaining;
            clamped = true;
        }
        *used_ratio_by_exchange
            .entry(action.exchange.clone())
            .or_default() += action.quantity_ratio;
    }

    clamped
}

fn clamp_buy_actions_to_total_headroom(
    actions: &mut [qrpc_core::ProposedAction],
    total_headroom: f64,
) -> bool {
    let total_buy_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Buy))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    if total_buy_ratio <= total_headroom + 1e-9 {
        return false;
    }

    if !total_headroom.is_finite() || total_headroom <= 0.0 {
        for action in actions
            .iter_mut()
            .filter(|item| matches!(item.side, OrderSide::Buy))
        {
            action.quantity_ratio = 0.0;
        }
        return true;
    }

    let scale = total_headroom / total_buy_ratio;
    for action in actions
        .iter_mut()
        .filter(|item| matches!(item.side, OrderSide::Buy))
    {
        action.quantity_ratio *= scale;
    }
    true
}

fn clamp_buy_actions_to_exchange_headroom(
    actions: &mut [qrpc_core::ProposedAction],
    exchange_headroom: &BTreeMap<Exchange, f64>,
) -> bool {
    let mut total_buy_by_exchange = BTreeMap::<Exchange, f64>::new();
    for action in actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Buy))
    {
        *total_buy_by_exchange
            .entry(action.exchange.clone())
            .or_default() += action.quantity_ratio;
    }

    let mut scale_by_exchange = BTreeMap::<Exchange, f64>::new();
    let mut clamped = false;
    for (exchange, total_ratio) in total_buy_by_exchange {
        let headroom = exchange_headroom
            .get(&exchange)
            .copied()
            .unwrap_or(f64::INFINITY);
        if total_ratio > headroom + 1e-9 {
            let scale = if !headroom.is_finite() || headroom <= 0.0 {
                0.0
            } else {
                headroom / total_ratio
            };
            scale_by_exchange.insert(exchange, scale);
            clamped = true;
        }
    }

    if !clamped {
        return false;
    }
    for action in actions
        .iter_mut()
        .filter(|item| matches!(item.side, OrderSide::Buy))
    {
        if let Some(scale) = scale_by_exchange.get(&action.exchange) {
            action.quantity_ratio *= *scale;
        }
    }
    true
}

fn clamp_buy_actions_to_symbol_net_exposure(
    actions: &mut [qrpc_core::ProposedAction],
    portfolio: &PortfolioState,
    symbol: &Symbol,
    max_symbol_net_exposure_ratio: f64,
    equity: f64,
) -> bool {
    let total_buy_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Buy))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    if !total_buy_ratio.is_finite() || total_buy_ratio <= 0.0 {
        return false;
    }
    let total_sell_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Sell))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    let current_ratio = symbol_net_exposure_ratio(portfolio, symbol, equity);
    let remaining_headroom =
        (max_symbol_net_exposure_ratio - (current_ratio - total_sell_ratio).max(0.0)).max(0.0);

    if total_buy_ratio <= remaining_headroom + 1e-9 {
        return false;
    }

    let scale = if !remaining_headroom.is_finite() || remaining_headroom <= 0.0 {
        0.0
    } else {
        remaining_headroom / total_buy_ratio
    };
    for action in actions
        .iter_mut()
        .filter(|item| matches!(item.side, OrderSide::Buy))
    {
        action.quantity_ratio *= scale;
    }
    true
}

fn clamp_buy_actions_to_portfolio_net_exposure(
    actions: &mut [qrpc_core::ProposedAction],
    portfolio: &PortfolioState,
    max_portfolio_net_exposure_ratio: f64,
    equity: f64,
) -> bool {
    let total_buy_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Buy))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    if !total_buy_ratio.is_finite() || total_buy_ratio <= 0.0 {
        return false;
    }
    let total_sell_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Sell))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    let current_ratio = portfolio_net_exposure_ratio(portfolio, equity);
    let remaining_headroom =
        (max_portfolio_net_exposure_ratio - (current_ratio - total_sell_ratio).max(0.0)).max(0.0);

    if total_buy_ratio <= remaining_headroom + 1e-9 {
        return false;
    }

    let scale = if !remaining_headroom.is_finite() || remaining_headroom <= 0.0 {
        0.0
    } else {
        remaining_headroom / total_buy_ratio
    };
    for action in actions
        .iter_mut()
        .filter(|item| matches!(item.side, OrderSide::Buy))
    {
        action.quantity_ratio *= scale;
    }
    true
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
