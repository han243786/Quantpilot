use qrpc_core::{
    AgentDecision, CoreStrategyIr, Exchange, IntentKind, IntentSignal, OrderSide, PortfolioState,
    PortfolioTarget, PortfolioTargetDecision, ProposedAction, RuntimeEvent, RuntimeEventType,
    SignalSide, Symbol, TargetWeight,
};
use qrpc_core_ir::{AgentPolicy, AgentPolicyKind, RebalanceSchedule, SignalKind};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

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
        let mut decisions = Vec::new();
        let mut events = Vec::new();
        let mut evaluated_rebalance_agent_ids = BTreeSet::new();
        let agent_policies = &request.core_ir.agent_policies;

        for agent in agent_policies.iter().filter(|item| item.enabled) {
            let related = request
                .signals
                .iter()
                .filter(|signal| agent.input_signal_ids.contains(&signal.intent_id))
                .cloned()
                .collect::<Vec<_>>();

            let agent_decisions = match (&agent.kind, request.cycle_name) {
                (AgentPolicyKind::WeightedSignals, "slow") => build_weighted_agent_decision(
                    agent,
                    &related,
                    request.core_ir,
                    request.portfolio,
                    request.now_ms,
                    request.trace_id,
                )
                .into_iter()
                .collect(),
                (AgentPolicyKind::PortfolioRebalance, "slow") => {
                    if !portfolio_rebalance_due(agent, request.last_rebalance_at_ms, request.now_ms)
                    {
                        Vec::new()
                    } else {
                        evaluated_rebalance_agent_ids.insert(agent.agent_id.clone());
                        build_portfolio_rebalance_decision(
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
                (AgentPolicyKind::CrossVenueArbitrage, "fast") => build_arb_agent_decision(
                    agent,
                    &related,
                    request.core_ir,
                    request.portfolio,
                    request.now_ms,
                    request.trace_id,
                )
                .into_iter()
                .collect(),
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

fn portfolio_rebalance_due(
    agent: &AgentPolicy,
    last_rebalance_at_ms: &BTreeMap<String, u64>,
    now_ms: u64,
) -> bool {
    match agent.rebalance_schedule.as_ref() {
        Some(RebalanceSchedule::Every1d) => last_rebalance_at_ms
            .get(&agent.agent_id)
            .map(|last| now_ms.saturating_sub(*last) >= 86_400_000)
            .unwrap_or(true),
        Some(RebalanceSchedule::Weekly) => last_rebalance_at_ms
            .get(&agent.agent_id)
            .map(|last| now_ms.saturating_sub(*last) >= 604_800_000)
            .unwrap_or(true),
        Some(RebalanceSchedule::EverySlow) | None => true,
    }
}

fn build_weighted_agent_decision(
    agent: &AgentPolicy,
    signals: &[IntentSignal],
    core_ir: &CoreStrategyIr,
    portfolio: &PortfolioState,
    now_ms: u64,
    trace_id: &str,
) -> Option<AgentDecision> {
    let weighted_signals = signals
        .iter()
        .filter(|item| {
            signal_kind_for_intent(core_ir, &item.intent_id) != Some(SignalKind::Observe)
                && !matches!(item.kind, IntentKind::QuoteObserve)
        })
        .collect::<Vec<_>>();
    if weighted_signals.is_empty() {
        return None;
    }

    let total_weight = weighted_signals
        .iter()
        .map(|item| item.confidence.max(0.1))
        .sum::<f64>();
    if total_weight <= f64::EPSILON {
        return None;
    }
    let net = weighted_signals
        .iter()
        .map(|item| signal_score(item) * item.confidence.max(0.1))
        .sum::<f64>()
        / total_weight;
    let decision_threshold = agent.decision_threshold.unwrap_or(0.05);
    if net.abs() < decision_threshold {
        return None;
    }

    let reference_price = signals
        .iter()
        .find_map(|item| item.reference_price)
        .unwrap_or(50_000.0);
    let target_exchange = signals
        .iter()
        .find_map(|item| item.exchange_scope.first().cloned())
        .unwrap_or(Exchange::Binance);
    let target_symbol = signals
        .iter()
        .find_map(|item| item.symbol_scope.first().cloned())
        .unwrap_or(Symbol::BtcUsdt);
    let max_quantity_ratio = agent.max_quantity_ratio.clamp(0.01, 1.0);
    let available_sell_ratio =
        available_position_ratio(portfolio, &target_exchange, &target_symbol, reference_price);
    let quantity_ratio = if net > 0.0 {
        net.abs()
            .clamp(decision_threshold.max(0.01), max_quantity_ratio)
    } else {
        net.abs()
            .clamp(decision_threshold.max(0.01), max_quantity_ratio)
            .min(available_sell_ratio)
    };
    if quantity_ratio <= 0.01 {
        return None;
    }
    Some(AgentDecision {
        decision_id: format!("decision-{}-{now_ms}", agent.agent_id),
        agent_id: agent.agent_id.clone(),
        symbol: target_symbol,
        exchange_targets: vec![target_exchange.clone()],
        net_side: if net > 0.0 {
            SignalSide::Long
        } else {
            SignalSide::Short
        },
        net_strength: net,
        portfolio_target_decision: None,
        proposed_actions: vec![ProposedAction {
            exchange: target_exchange,
            side: if net > 0.0 {
                OrderSide::Buy
            } else {
                OrderSide::Sell
            },
            quantity_ratio,
            reference_price,
            strategy_tag: "long_term".into(),
        }],
        reason: format!(
            "net_score {:.4}, signals {}, threshold {:.4}",
            net,
            weighted_signals.len(),
            decision_threshold
        ),
        produced_at_ms: now_ms,
        trace_id: trace_id.to_string(),
    })
}

#[derive(Debug, Clone)]
struct RebalanceSymbolPlan {
    symbol: Symbol,
    exchange: Exchange,
    reference_price: f64,
    score: f64,
    target_weight: f64,
    current_weight: f64,
}

fn build_portfolio_rebalance_decision(
    agent: &AgentPolicy,
    signals: &[IntentSignal],
    core_ir: &CoreStrategyIr,
    portfolio: &PortfolioState,
    now_ms: u64,
    trace_id: &str,
) -> Option<AgentDecision> {
    let weighted_signals = signals
        .iter()
        .filter(|item| {
            signal_kind_for_intent(core_ir, &item.intent_id) != Some(SignalKind::Observe)
                && !matches!(item.kind, IntentKind::QuoteObserve)
        })
        .collect::<Vec<_>>();
    if weighted_signals.is_empty() {
        return None;
    }

    let decision_threshold = agent.decision_threshold.unwrap_or(0.05).max(0.0);
    let mut aggregated = BTreeMap::<(Exchange, Symbol), (f64, f64)>::new();
    for signal in weighted_signals {
        let exchange = signal
            .exchange_scope
            .first()
            .cloned()
            .unwrap_or(Exchange::Binance);
        let symbol = signal
            .symbol_scope
            .first()
            .cloned()
            .unwrap_or(Symbol::BtcUsdt);
        let reference_price = signal.reference_price.unwrap_or(50_000.0);
        let entry = aggregated
            .entry((exchange, symbol))
            .or_insert((0.0_f64, reference_price));
        entry.0 += signal_score(signal) * signal.confidence.max(0.1);
        if entry.1 <= 0.0 && reference_price > 0.0 {
            entry.1 = reference_price;
        }
    }

    let explicit_universe = agent
        .rebalance_symbols
        .iter()
        .map(|symbol| Symbol::parse(symbol))
        .collect::<BTreeSet<_>>();
    let universe_symbols = if explicit_universe.is_empty() {
        aggregated
            .keys()
            .map(|(_, symbol)| symbol.clone())
            .collect::<BTreeSet<_>>()
    } else {
        explicit_universe.clone()
    };
    if universe_symbols.is_empty() {
        return None;
    }

    let selected_symbols = aggregated
        .iter()
        .filter_map(|((_, symbol), (score, _))| {
            (*score > decision_threshold).then_some(symbol.clone())
        })
        .collect::<BTreeSet<_>>();

    let mut target_weights = universe_symbols
        .into_iter()
        .map(|symbol| {
            let signal_match = aggregated
                .iter()
                .find(|((_, candidate_symbol), _)| *candidate_symbol == symbol);
            let exchange = signal_match
                .map(|((exchange, _), _)| exchange.clone())
                .or_else(|| {
                    portfolio
                        .positions
                        .iter()
                        .find(|position| position.symbol == symbol)
                        .map(|position| position.exchange.clone())
                })
                .unwrap_or(Exchange::Binance);
            let score = signal_match.map(|(_, (score, _))| *score).unwrap_or(0.0);
            let reference_price = signal_match
                .map(|(_, (_, reference_price))| *reference_price)
                .or_else(|| {
                    portfolio
                        .positions
                        .iter()
                        .find(|position| position.symbol == symbol && position.mark_price > 0.0)
                        .map(|position| position.mark_price)
                })
                .or_else(|| {
                    portfolio
                        .positions
                        .iter()
                        .find(|position| {
                            position.symbol == symbol && position.avg_entry_price > 0.0
                        })
                        .map(|position| position.avg_entry_price)
                })
                .unwrap_or(50_000.0);
            let current_weight =
                current_position_ratio(portfolio, &exchange, &symbol, reference_price.max(0.0));
            RebalanceSymbolPlan {
                symbol,
                exchange,
                reference_price,
                score,
                target_weight: 0.0,
                current_weight,
            }
        })
        .collect::<Vec<_>>();
    assign_target_weights(agent, &selected_symbols, &mut target_weights);
    let net_delta = target_weights
        .iter()
        .map(|plan| (plan.target_weight - plan.current_weight).abs())
        .sum::<f64>();
    if net_delta <= 0.01 {
        return None;
    }

    let target_symbol = target_weights
        .first()
        .map(|item| item.symbol.clone())
        .unwrap_or(Symbol::BtcUsdt);
    let exchange_targets = target_weights
        .iter()
        .map(|item| item.exchange.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let buy_delta = target_weights
        .iter()
        .map(|plan| (plan.target_weight - plan.current_weight).max(0.0))
        .sum::<f64>();
    let sell_delta = target_weights
        .iter()
        .map(|plan| (plan.current_weight - plan.target_weight).max(0.0))
        .sum::<f64>();
    let net_side = if buy_delta > sell_delta {
        SignalSide::Long
    } else if sell_delta > buy_delta {
        SignalSide::Short
    } else {
        SignalSide::Neutral
    };
    let allocation_kind = agent
        .rebalance_allocation_kind
        .clone()
        .unwrap_or_else(|| "equal_weight".into());
    let reason = format!(
        "portfolio rebalance target: allocation {}, selected {} of {} symbols",
        allocation_kind,
        selected_symbols.len(),
        target_weights.len()
    );

    Some(AgentDecision {
        decision_id: format!("decision-{}-{now_ms}", agent.agent_id),
        agent_id: agent.agent_id.clone(),
        symbol: target_symbol,
        exchange_targets,
        net_side,
        net_strength: buy_delta - sell_delta,
        portfolio_target_decision: Some(PortfolioTargetDecision {
            target_id: format!("target-{}-{now_ms}", agent.agent_id),
            target: PortfolioTarget {
                allocation_kind,
                target_weights: target_weights
                    .into_iter()
                    .map(|plan| TargetWeight {
                        exchange: plan.exchange,
                        symbol: plan.symbol,
                        target_weight: plan.target_weight,
                        current_weight: plan.current_weight,
                        reference_price: plan.reference_price,
                        signal_score: Some(plan.score),
                    })
                    .collect(),
            },
            reason: reason.clone(),
        }),
        proposed_actions: Vec::new(),
        reason,
        produced_at_ms: now_ms,
        trace_id: trace_id.to_string(),
    })
}

fn assign_target_weights(
    agent: &AgentPolicy,
    selected_symbols: &BTreeSet<Symbol>,
    plans: &mut [RebalanceSymbolPlan],
) {
    let allocation_kind = agent
        .rebalance_allocation_kind
        .as_deref()
        .unwrap_or("equal_weight");
    let max_quantity_ratio = agent.max_quantity_ratio.clamp(0.01, 1.0);
    match allocation_kind {
        "fixed_weights" => {
            for (plan, configured_weight) in
                plans.iter_mut().zip(agent.rebalance_target_weights.iter())
            {
                plan.target_weight = configured_weight.clamp(0.0, max_quantity_ratio);
            }
        }
        "rank_weight" => {
            let method = agent.rebalance_rank_method.as_deref().unwrap_or("linear");
            let mut selected = plans
                .iter_mut()
                .filter(|plan| selected_symbols.contains(&plan.symbol))
                .collect::<Vec<_>>();
            selected.sort_by(|left, right| {
                right
                    .score
                    .partial_cmp(&left.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| left.symbol.as_str().cmp(right.symbol.as_str()))
            });
            let raw_weights = selected
                .iter()
                .enumerate()
                .map(|(index, _)| match method {
                    "inverse_rank" => 1.0 / (index + 1) as f64,
                    _ => (selected.len().saturating_sub(index)) as f64,
                })
                .collect::<Vec<_>>();
            let total = raw_weights.iter().sum::<f64>();
            if total > f64::EPSILON {
                for (plan, raw) in selected.into_iter().zip(raw_weights.into_iter()) {
                    plan.target_weight = (raw / total).min(max_quantity_ratio);
                }
            }
        }
        "score_weight" => {
            let selected = plans
                .iter_mut()
                .filter(|plan| selected_symbols.contains(&plan.symbol) && plan.score > 0.0)
                .collect::<Vec<_>>();
            let total = selected.iter().map(|plan| plan.score).sum::<f64>();
            if total > f64::EPSILON {
                for plan in selected {
                    plan.target_weight = (plan.score / total).min(max_quantity_ratio);
                }
            }
        }
        _ => {
            let active_count = selected_symbols.len();
            let equal_target_weight = if active_count == 0 {
                0.0
            } else {
                (1.0 / active_count as f64).min(max_quantity_ratio)
            };
            for plan in plans.iter_mut() {
                if selected_symbols.contains(&plan.symbol) {
                    plan.target_weight = equal_target_weight;
                }
            }
        }
    }
}

fn signal_kind_for_intent(core_ir: &CoreStrategyIr, intent_id: &str) -> Option<SignalKind> {
    core_ir
        .signal_rules
        .iter()
        .find(|rule| rule.indicator_id == intent_id)
        .map(|rule| rule.signal_kind)
}

fn build_arb_agent_decision(
    agent: &AgentPolicy,
    signals: &[IntentSignal],
    core_ir: &CoreStrategyIr,
    portfolio: &PortfolioState,
    now_ms: u64,
    trace_id: &str,
) -> Option<AgentDecision> {
    if let Some(decision) =
        build_arb_decision_from_spread_signal(agent, signals, core_ir, portfolio, now_ms, trace_id)
    {
        return Some(decision);
    }

    let signal_by_exchange = signals
        .iter()
        .filter_map(|item| {
            item.exchange_scope
                .first()
                .cloned()
                .map(|exchange| (exchange, item))
        })
        .collect::<BTreeMap<_, _>>();
    let binance = signal_by_exchange.get(&Exchange::Binance).copied()?;
    let okx = signal_by_exchange.get(&Exchange::Okx).copied()?;
    let binance_mid = binance.reference_price?;
    let okx_mid = okx.reference_price?;
    let spread = (binance_mid - okx_mid).abs() / binance_mid.min(okx_mid);
    let total_cost_buffer = total_cost_buffer_ratio(core_ir);
    let spread_trigger =
        (agent.spread_trigger_bps.unwrap_or(50.0) / 10_000.0).max(total_cost_buffer);
    if spread <= spread_trigger {
        return None;
    }

    let (buy_exchange, sell_exchange, buy_price, sell_price) = if binance_mid < okx_mid {
        (Exchange::Binance, Exchange::Okx, binance_mid, okx_mid)
    } else {
        (Exchange::Okx, Exchange::Binance, okx_mid, binance_mid)
    };
    let target_symbol = signals
        .iter()
        .find_map(|item| item.symbol_scope.first().cloned())
        .unwrap_or(Symbol::BtcUsdt);
    let max_quantity_ratio = agent.max_quantity_ratio.clamp(0.01, 1.0);
    let available_sell_ratio =
        available_position_ratio(portfolio, &sell_exchange, &target_symbol, sell_price);
    let quantity_ratio = (spread * 20.0)
        .clamp(0.1, max_quantity_ratio)
        .min(available_sell_ratio);
    if quantity_ratio <= 0.01 {
        return None;
    }

    Some(AgentDecision {
        decision_id: format!("decision-{}-{now_ms}", agent.agent_id),
        agent_id: agent.agent_id.clone(),
        symbol: target_symbol,
        exchange_targets: vec![buy_exchange.clone(), sell_exchange.clone()],
        net_side: SignalSide::Long,
        net_strength: quantity_ratio,
        portfolio_target_decision: None,
        proposed_actions: vec![
            ProposedAction {
                exchange: buy_exchange,
                side: OrderSide::Buy,
                quantity_ratio,
                reference_price: buy_price,
                strategy_tag: "arb_buy_leg".into(),
            },
            ProposedAction {
                exchange: sell_exchange,
                side: OrderSide::Sell,
                quantity_ratio,
                reference_price: sell_price,
                strategy_tag: "arb_sell_leg".into(),
            },
        ],
        reason: format!(
            "spread {:.4}% exceeds trigger {:.4}%",
            spread * 100.0,
            spread_trigger * 100.0
        ),
        produced_at_ms: now_ms,
        trace_id: trace_id.to_string(),
    })
}

fn build_arb_decision_from_spread_signal(
    agent: &AgentPolicy,
    signals: &[IntentSignal],
    core_ir: &CoreStrategyIr,
    portfolio: &PortfolioState,
    now_ms: u64,
    trace_id: &str,
) -> Option<AgentDecision> {
    let signal = signals
        .iter()
        .find(|item| item.derived_metrics.contains_key("spread_ratio"))?;
    let spread = *signal.derived_metrics.get("spread_ratio")?;
    let buy_price = *signal.derived_metrics.get("buy_mid_price")?;
    let sell_price = *signal.derived_metrics.get("sell_mid_price")?;
    let buy_exchange = signal.exchange_scope.first().cloned()?;
    let sell_exchange = signal.exchange_scope.get(1).cloned()?;
    let target_symbol = signal
        .symbol_scope
        .first()
        .cloned()
        .unwrap_or(Symbol::BtcUsdt);

    let total_cost_buffer = total_cost_buffer_ratio(core_ir);
    let spread_trigger =
        (agent.spread_trigger_bps.unwrap_or(50.0) / 10_000.0).max(total_cost_buffer);
    if spread <= spread_trigger {
        return None;
    }

    let max_quantity_ratio = agent.max_quantity_ratio.clamp(0.01, 1.0);
    let available_sell_ratio =
        available_position_ratio(portfolio, &sell_exchange, &target_symbol, sell_price);
    let quantity_ratio = (spread * 20.0)
        .clamp(0.1, max_quantity_ratio)
        .min(available_sell_ratio);
    if quantity_ratio <= 0.01 {
        return None;
    }

    Some(AgentDecision {
        decision_id: format!("decision-{}-{now_ms}", agent.agent_id),
        agent_id: agent.agent_id.clone(),
        symbol: target_symbol,
        exchange_targets: vec![buy_exchange.clone(), sell_exchange.clone()],
        net_side: SignalSide::Long,
        net_strength: spread,
        portfolio_target_decision: None,
        proposed_actions: vec![
            ProposedAction {
                exchange: buy_exchange,
                side: OrderSide::Buy,
                quantity_ratio,
                reference_price: buy_price,
                strategy_tag: "arb_buy_leg".into(),
            },
            ProposedAction {
                exchange: sell_exchange,
                side: OrderSide::Sell,
                quantity_ratio,
                reference_price: sell_price,
                strategy_tag: "arb_sell_leg".into(),
            },
        ],
        reason: format!(
            "{}; spread {:.4}% exceeds trigger {:.4}%",
            signal.reason,
            spread * 100.0,
            spread_trigger * 100.0
        ),
        produced_at_ms: now_ms,
        trace_id: trace_id.to_string(),
    })
}

fn total_cost_buffer_ratio(core_ir: &CoreStrategyIr) -> f64 {
    (core_ir.execution.taker_fee_bps * 2.0
        + core_ir.execution.slippage_bps * 2.0
        + core_ir.execution.total_cost_buffer_bps)
        / 10_000.0
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
    if reference_price <= 0.0 {
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
    if reference_price <= 0.0 {
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
mod tests {
    use super::*;
    use qrpc_core::{IntentSignal, PortfolioState, Position};
    use qrpc_core_ir::{
        AgentPolicy, AgentPolicyKind, CoreMetadata, CoreSourceKind, CoreStrategyIr,
        CoreTimeInForce, ExecutionRule, ExecutionSizingKind, SignalKind, SignalRule,
    };
    use std::collections::BTreeMap;

    fn sample_execution_rule() -> ExecutionRule {
        ExecutionRule {
            execution_id: "exec".into(),
            venue_kind: "paper".into(),
            sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
            slippage_bps: 5.0,
            taker_fee_bps: 10.0,
            total_cost_buffer_bps: 20.0,
            time_in_force: CoreTimeInForce::Gtc,
            params: BTreeMap::new(),
        }
    }

    fn sample_core_ir_with_agent_policy(
        agent_id: &str,
        kind: AgentPolicyKind,
        input_signal_ids: Vec<&str>,
        signal_rules: Vec<SignalRule>,
        decision_threshold: Option<f64>,
        max_quantity_ratio: f64,
        spread_trigger_bps: Option<f64>,
    ) -> CoreStrategyIr {
        CoreStrategyIr {
            ir_version: qrpc_core::CORE_IR_V1_VERSION.to_string(),
            metadata: CoreMetadata {
                strategy_id: "agent_test".into(),
                name: "Agent Test".into(),
                source_kind: CoreSourceKind::RuntimeProtocol,
            },
            data_bindings: vec![],
            indicators: vec![],
            signal_rules,
            agent_policies: vec![AgentPolicy {
                agent_id: agent_id.into(),
                name: agent_id.into(),
                kind,
                input_signal_ids: input_signal_ids.into_iter().map(str::to_string).collect(),
                rebalance_symbols: vec![],
                rebalance_schedule: None,
                rebalance_allocation_kind: None,
                rebalance_rank_method: None,
                rebalance_score_normalize: None,
                rebalance_target_weights: vec![],
                decision_threshold,
                max_quantity_ratio,
                spread_trigger_bps,
                enabled: true,
            }],
            risk_policies: vec![],
            execution: sample_execution_rule(),
            edges: vec![],
        }
    }

    fn sample_portfolio_with_symbol_position(
        exchange: Exchange,
        symbol: Symbol,
        qty: f64,
        mark_price: f64,
    ) -> PortfolioState {
        let mut portfolio = PortfolioState::new(100_000.0, 0);
        portfolio.positions.push(Position {
            exchange,
            symbol,
            net_qty: qty,
            frozen_qty: 0.0,
            avg_entry_price: mark_price,
            mark_price,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        });
        portfolio.total_net_notional = qty * mark_price;
        portfolio.total_gross_notional = qty.abs() * mark_price;
        portfolio.total_leverage = portfolio.total_gross_notional
            / (portfolio.cash_balance + portfolio.total_net_notional);
        portfolio
    }

    fn sample_portfolio_with_position(
        exchange: Exchange,
        qty: f64,
        mark_price: f64,
    ) -> PortfolioState {
        sample_portfolio_with_symbol_position(exchange, Symbol::BtcUsdt, qty, mark_price)
    }

    #[test]
    fn builtin_agent_module_emits_decision_for_fast_cycle_arb() {
        let module = BuiltinAgentModule;
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_arb",
            AgentPolicyKind::CrossVenueArbitrage,
            vec!["intent_binance_quote", "intent_okx_quote"],
            vec![
                SignalRule {
                    signal_id: "binance_observe".into(),
                    indicator_id: "intent_binance_quote".into(),
                    signal_kind: SignalKind::Observe,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "observe".into(),
                    },
                },
                SignalRule {
                    signal_id: "okx_observe".into(),
                    indicator_id: "intent_okx_quote".into(),
                    signal_kind: SignalKind::Observe,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "observe".into(),
                    },
                },
            ],
            None,
            0.4,
            Some(30.0),
        );
        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "fast",
            signals: &[
                IntentSignal {
                    signal_id: "s1".into(),
                    intent_id: "intent_binance_quote".into(),
                    kind: IntentKind::QuoteObserve,
                    exchange_scope: vec![Exchange::Binance],
                    symbol_scope: vec![Symbol::BtcUsdt],
                    side: SignalSide::Neutral,
                    strength: 0.0,
                    confidence: 1.0,
                    reference_price: Some(50_000.0),
                    derived_metrics: BTreeMap::new(),
                    reason: "binance".into(),
                    triggered_at_ms: 10,
                    ttl_ms: 1000,
                    trace_id: "trace".into(),
                },
                IntentSignal {
                    signal_id: "s2".into(),
                    intent_id: "intent_okx_quote".into(),
                    kind: IntentKind::QuoteObserve,
                    exchange_scope: vec![Exchange::Okx],
                    symbol_scope: vec![Symbol::BtcUsdt],
                    side: SignalSide::Neutral,
                    strength: 0.0,
                    confidence: 1.0,
                    reference_price: Some(50_350.0),
                    derived_metrics: BTreeMap::new(),
                    reason: "okx".into(),
                    triggered_at_ms: 10,
                    ttl_ms: 1000,
                    trace_id: "trace".into(),
                },
            ],
            core_ir: &core_ir,
            portfolio: &sample_portfolio_with_position(Exchange::Okx, 1.0, 50_350.0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.decisions.len(), 1);
        assert_eq!(output.events.len(), 1);
        assert_eq!(
            output.events[0].payload["provider_key"],
            "builtin.agent.default"
        );
    }

    #[test]
    fn builtin_agent_module_emits_decision_from_spread_signal() {
        let module = BuiltinAgentModule;
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_arb",
            AgentPolicyKind::CrossVenueArbitrage,
            vec!["intent_spread"],
            vec![SignalRule {
                signal_id: "spread_observe".into(),
                indicator_id: "intent_spread".into(),
                signal_kind: SignalKind::Observe,
                condition: qrpc_core_ir::ScalarExpr::RawText {
                    source: "spread".into(),
                },
            }],
            None,
            0.4,
            Some(30.0),
        );
        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "fast",
            signals: &[IntentSignal {
                signal_id: "s1".into(),
                intent_id: "intent_spread".into(),
                kind: IntentKind::QuoteObserve,
                exchange_scope: vec![Exchange::Binance, Exchange::Okx],
                symbol_scope: vec![Symbol::BtcUsdt],
                side: SignalSide::Neutral,
                strength: 0.007,
                confidence: 0.95,
                reference_price: Some(50_000.0),
                derived_metrics: BTreeMap::from([
                    ("buy_mid_price".into(), 50_000.0),
                    ("sell_mid_price".into(), 50_350.0),
                    ("spread_ratio".into(), 0.007),
                ]),
                reason: "spread observe Binance->Okx 70bps".into(),
                triggered_at_ms: 10,
                ttl_ms: 1000,
                trace_id: "trace".into(),
            }],
            core_ir: &core_ir,
            portfolio: &sample_portfolio_with_position(Exchange::Okx, 1.0, 50_350.0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.decisions.len(), 1);
        assert_eq!(
            output.decisions[0].exchange_targets,
            vec![Exchange::Binance, Exchange::Okx]
        );
        assert_eq!(output.events.len(), 1);
    }

    #[test]
    fn long_term_agent_inherits_exchange_from_signal_scope() {
        let module = BuiltinAgentModule;
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_long_term",
            AgentPolicyKind::WeightedSignals,
            vec!["intent_long_buy", "intent_long_sell"],
            vec![
                SignalRule {
                    signal_id: "long_buy".into(),
                    indicator_id: "intent_long_buy".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
                SignalRule {
                    signal_id: "long_sell".into(),
                    indicator_id: "intent_long_sell".into(),
                    signal_kind: SignalKind::Short,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "short".into(),
                    },
                },
            ],
            Some(0.05),
            0.5,
            None,
        );
        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[IntentSignal {
                signal_id: "s1".into(),
                intent_id: "intent_long_buy".into(),
                kind: IntentKind::LongTermBuy,
                exchange_scope: vec![Exchange::Okx],
                symbol_scope: vec![Symbol::BtcUsdt],
                side: SignalSide::Long,
                strength: 0.8,
                confidence: 1.0,
                reference_price: Some(70_000.0),
                derived_metrics: BTreeMap::new(),
                reason: "trend".into(),
                triggered_at_ms: 10,
                ttl_ms: 1000,
                trace_id: "trace".into(),
            }],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.decisions.len(), 1);
        assert_eq!(output.decisions[0].exchange_targets, vec![Exchange::Okx]);
        assert_eq!(
            output.decisions[0].proposed_actions[0].exchange,
            Exchange::Okx
        );
    }

    #[test]
    fn long_term_agent_inherits_symbol_from_signal_scope() {
        let module = BuiltinAgentModule;
        let eth = Symbol::parse("ETHUSDT");
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_long_term",
            AgentPolicyKind::WeightedSignals,
            vec!["intent_long_buy"],
            vec![SignalRule {
                signal_id: "long_buy".into(),
                indicator_id: "intent_long_buy".into(),
                signal_kind: SignalKind::Long,
                condition: qrpc_core_ir::ScalarExpr::RawText {
                    source: "long".into(),
                },
            }],
            Some(0.05),
            0.5,
            None,
        );
        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[IntentSignal {
                signal_id: "s1".into(),
                intent_id: "intent_long_buy".into(),
                kind: IntentKind::LongTermBuy,
                exchange_scope: vec![Exchange::Binance],
                symbol_scope: vec![eth.clone()],
                side: SignalSide::Long,
                strength: 0.8,
                confidence: 1.0,
                reference_price: Some(4_000.0),
                derived_metrics: BTreeMap::new(),
                reason: "trend".into(),
                triggered_at_ms: 10,
                ttl_ms: 1000,
                trace_id: "trace".into(),
            }],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.decisions.len(), 1);
        assert_eq!(output.decisions[0].symbol, eth);
    }

    #[test]
    fn portfolio_rebalance_agent_emits_equal_weight_portfolio_target() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc", "intent_eth"],
            vec![
                SignalRule {
                    signal_id: "btc_signal".into(),
                    indicator_id: "intent_btc".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
                SignalRule {
                    signal_id: "eth_signal".into(),
                    indicator_id: "intent_eth".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
            ],
            Some(0.05),
            0.8,
            None,
        );
        let mut portfolio =
            sample_portfolio_with_symbol_position(Exchange::Binance, btc.clone(), 1.4, 50_000.0);
        portfolio.cash_balance = 30_000.0;
        portfolio.available_cash_balance = 30_000.0;
        portfolio.total_net_notional = 70_000.0;
        portfolio.total_gross_notional = 70_000.0;
        portfolio.total_leverage = 70_000.0 / 100_000.0;

        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[
                IntentSignal {
                    signal_id: "s_btc".into(),
                    intent_id: "intent_btc".into(),
                    kind: IntentKind::LongTermBuy,
                    exchange_scope: vec![Exchange::Binance],
                    symbol_scope: vec![btc.clone()],
                    side: SignalSide::Long,
                    strength: 0.9,
                    confidence: 1.0,
                    reference_price: Some(50_000.0),
                    derived_metrics: BTreeMap::new(),
                    reason: "btc selected".into(),
                    triggered_at_ms: 10,
                    ttl_ms: 1000,
                    trace_id: "trace".into(),
                },
                IntentSignal {
                    signal_id: "s_eth".into(),
                    intent_id: "intent_eth".into(),
                    kind: IntentKind::LongTermBuy,
                    exchange_scope: vec![Exchange::Binance],
                    symbol_scope: vec![eth.clone()],
                    side: SignalSide::Long,
                    strength: 0.8,
                    confidence: 1.0,
                    reference_price: Some(4_000.0),
                    derived_metrics: BTreeMap::new(),
                    reason: "eth selected".into(),
                    triggered_at_ms: 10,
                    ttl_ms: 1000,
                    trace_id: "trace".into(),
                },
            ],
            core_ir: &core_ir,
            portfolio: &portfolio,
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.decisions.len(), 1);
        let target = output.decisions[0]
            .portfolio_target_decision
            .as_ref()
            .expect("portfolio target decision");
        assert!(output.decisions[0].proposed_actions.is_empty());
        assert_eq!(target.target.target_weights.len(), 2);
        let btc_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == btc)
            .expect("btc target");
        let eth_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == eth)
            .expect("eth target");
        assert!((btc_weight.current_weight - 0.7).abs() < 0.001);
        assert!((btc_weight.target_weight - 0.5).abs() < 0.001);
        assert!((eth_weight.current_weight - 0.0).abs() < 0.001);
        assert!((eth_weight.target_weight - 0.5).abs() < 0.001);
    }

    #[test]
    fn portfolio_rebalance_agent_emits_fixed_weight_portfolio_target() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc", "intent_eth"],
            vec![
                SignalRule {
                    signal_id: "btc_signal".into(),
                    indicator_id: "intent_btc".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
                SignalRule {
                    signal_id: "eth_signal".into(),
                    indicator_id: "intent_eth".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
            ],
            Some(0.05),
            1.0,
            None,
        );
        let mut core_ir = core_ir;
        core_ir.agent_policies[0].rebalance_symbols =
            vec![btc.as_str().to_string(), eth.as_str().to_string()];
        core_ir.agent_policies[0].rebalance_allocation_kind = Some("fixed_weights".into());
        core_ir.agent_policies[0].rebalance_target_weights = vec![0.7, 0.3];

        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[
                IntentSignal {
                    signal_id: "s_btc".into(),
                    intent_id: "intent_btc".into(),
                    kind: IntentKind::LongTermBuy,
                    exchange_scope: vec![Exchange::Binance],
                    symbol_scope: vec![btc.clone()],
                    side: SignalSide::Long,
                    strength: 0.9,
                    confidence: 1.0,
                    reference_price: Some(50_000.0),
                    derived_metrics: BTreeMap::new(),
                    reason: "btc selected".into(),
                    triggered_at_ms: 10,
                    ttl_ms: 1000,
                    trace_id: "trace".into(),
                },
                IntentSignal {
                    signal_id: "s_eth".into(),
                    intent_id: "intent_eth".into(),
                    kind: IntentKind::LongTermBuy,
                    exchange_scope: vec![Exchange::Binance],
                    symbol_scope: vec![eth.clone()],
                    side: SignalSide::Long,
                    strength: 0.8,
                    confidence: 1.0,
                    reference_price: Some(4_000.0),
                    derived_metrics: BTreeMap::new(),
                    reason: "eth selected".into(),
                    triggered_at_ms: 10,
                    ttl_ms: 1000,
                    trace_id: "trace".into(),
                },
            ],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        let target = output.decisions[0]
            .portfolio_target_decision
            .as_ref()
            .expect("portfolio target");
        let btc_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == btc)
            .expect("btc target");
        let eth_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == eth)
            .expect("eth target");
        assert!((btc_weight.target_weight - 0.7).abs() < 1e-9);
        assert!((eth_weight.target_weight - 0.3).abs() < 1e-9);
    }

    #[test]
    fn portfolio_rebalance_agent_emits_rank_weight_portfolio_target() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let sol = Symbol::parse("SOLUSDT");
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc", "intent_eth", "intent_sol"],
            vec![
                SignalRule {
                    signal_id: "btc_signal".into(),
                    indicator_id: "intent_btc".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
                SignalRule {
                    signal_id: "eth_signal".into(),
                    indicator_id: "intent_eth".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
                SignalRule {
                    signal_id: "sol_signal".into(),
                    indicator_id: "intent_sol".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
            ],
            Some(0.05),
            1.0,
            None,
        );
        let mut core_ir = core_ir;
        core_ir.agent_policies[0].rebalance_symbols = vec![
            btc.as_str().to_string(),
            eth.as_str().to_string(),
            sol.as_str().to_string(),
        ];
        core_ir.agent_policies[0].rebalance_allocation_kind = Some("rank_weight".into());
        core_ir.agent_policies[0].rebalance_rank_method = Some("inverse_rank".into());

        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[
                sample_long_signal("intent_btc", btc.clone(), 50_000.0, 0.9),
                sample_long_signal("intent_eth", eth.clone(), 4_000.0, 0.6),
                sample_long_signal("intent_sol", sol.clone(), 150.0, 0.3),
            ],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        let target = output.decisions[0]
            .portfolio_target_decision
            .as_ref()
            .expect("portfolio target");
        let btc_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == btc)
            .expect("btc");
        let eth_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == eth)
            .expect("eth");
        let sol_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == sol)
            .expect("sol");
        assert!((btc_weight.target_weight - (1.0 / 1.8333333333333333)).abs() < 0.001);
        assert!((eth_weight.target_weight - ((1.0 / 2.0) / 1.8333333333333333)).abs() < 0.001);
        assert!((sol_weight.target_weight - ((1.0 / 3.0) / 1.8333333333333333)).abs() < 0.001);
    }

    #[test]
    fn portfolio_rebalance_agent_emits_score_weight_portfolio_target() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc", "intent_eth"],
            vec![
                SignalRule {
                    signal_id: "btc_signal".into(),
                    indicator_id: "intent_btc".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
                SignalRule {
                    signal_id: "eth_signal".into(),
                    indicator_id: "intent_eth".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
            ],
            Some(0.05),
            1.0,
            None,
        );
        let mut core_ir = core_ir;
        core_ir.agent_policies[0].rebalance_symbols =
            vec![btc.as_str().to_string(), eth.as_str().to_string()];
        core_ir.agent_policies[0].rebalance_allocation_kind = Some("score_weight".into());
        core_ir.agent_policies[0].rebalance_score_normalize = Some("sum".into());

        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[
                sample_long_signal("intent_btc", btc.clone(), 50_000.0, 0.9),
                sample_long_signal("intent_eth", eth.clone(), 4_000.0, 0.3),
            ],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        let target = output.decisions[0]
            .portfolio_target_decision
            .as_ref()
            .expect("portfolio target");
        let btc_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == btc)
            .expect("btc");
        let eth_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == eth)
            .expect("eth");
        assert!((btc_weight.target_weight - 0.75).abs() < 0.001);
        assert!((eth_weight.target_weight - 0.25).abs() < 0.001);
    }

    #[test]
    fn portfolio_rebalance_agent_sells_universe_member_without_current_signal() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc"],
            vec![SignalRule {
                signal_id: "btc_signal".into(),
                indicator_id: "intent_btc".into(),
                signal_kind: SignalKind::Long,
                condition: qrpc_core_ir::ScalarExpr::RawText {
                    source: "long".into(),
                },
            }],
            Some(0.05),
            1.0,
            None,
        );
        let mut core_ir = core_ir;
        core_ir.agent_policies[0].rebalance_symbols =
            vec![btc.as_str().to_string(), eth.as_str().to_string()];

        let mut portfolio =
            sample_portfolio_with_symbol_position(Exchange::Binance, eth.clone(), 5.0, 4_000.0);
        portfolio.cash_balance = 80_000.0;
        portfolio.available_cash_balance = 80_000.0;
        portfolio.total_net_notional = 20_000.0;
        portfolio.total_gross_notional = 20_000.0;
        portfolio.total_leverage = 0.2;

        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[IntentSignal {
                signal_id: "s_btc".into(),
                intent_id: "intent_btc".into(),
                kind: IntentKind::LongTermBuy,
                exchange_scope: vec![Exchange::Binance],
                symbol_scope: vec![btc],
                side: SignalSide::Long,
                strength: 0.9,
                confidence: 1.0,
                reference_price: Some(50_000.0),
                derived_metrics: BTreeMap::new(),
                reason: "btc selected".into(),
                triggered_at_ms: 10,
                ttl_ms: 1000,
                trace_id: "trace".into(),
            }],
            core_ir: &core_ir,
            portfolio: &portfolio,
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.decisions.len(), 1);
        let target = output.decisions[0]
            .portfolio_target_decision
            .as_ref()
            .expect("portfolio target decision");
        let eth_target = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == eth)
            .expect("eth rebalance exit target");
        assert!((eth_target.current_weight - 0.2).abs() < 0.001);
        assert!((eth_target.target_weight - 0.0).abs() < 0.001);
    }

    #[test]
    fn portfolio_rebalance_agent_respects_daily_cadence() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc"],
            vec![SignalRule {
                signal_id: "btc_signal".into(),
                indicator_id: "intent_btc".into(),
                signal_kind: SignalKind::Long,
                condition: qrpc_core_ir::ScalarExpr::RawText {
                    source: "long".into(),
                },
            }],
            Some(0.05),
            1.0,
            None,
        );
        let mut core_ir = core_ir;
        core_ir.agent_policies[0].rebalance_symbols = vec![btc.as_str().to_string()];
        core_ir.agent_policies[0].rebalance_schedule =
            Some(qrpc_core_ir::RebalanceSchedule::Every1d);

        let signals = vec![IntentSignal {
            signal_id: "s_btc".into(),
            intent_id: "intent_btc".into(),
            kind: IntentKind::LongTermBuy,
            exchange_scope: vec![Exchange::Binance],
            symbol_scope: vec![btc],
            side: SignalSide::Long,
            strength: 0.9,
            confidence: 1.0,
            reference_price: Some(50_000.0),
            derived_metrics: BTreeMap::new(),
            reason: "btc selected".into(),
            triggered_at_ms: 10,
            ttl_ms: 1000,
            trace_id: "trace".into(),
        }];
        let last_rebalance_at_ms = BTreeMap::from([("agent_rebalance".to_string(), 1_000_u64)]);

        let skipped = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &signals,
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &last_rebalance_at_ms,
            now_ms: 1_000 + 3_600_000,
            trace_id: "trace",
        });
        assert!(skipped.decisions.is_empty());
        assert!(skipped.evaluated_rebalance_agent_ids.is_empty());

        let due = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &signals,
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &last_rebalance_at_ms,
            now_ms: 1_000 + 86_400_000,
            trace_id: "trace",
        });
        assert_eq!(due.decisions.len(), 1);
        assert!(due.decisions[0].portfolio_target_decision.is_some());
        assert!(due
            .evaluated_rebalance_agent_ids
            .contains("agent_rebalance"));
    }

    #[test]
    fn portfolio_rebalance_agent_respects_weekly_cadence() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc"],
            vec![SignalRule {
                signal_id: "btc_signal".into(),
                indicator_id: "intent_btc".into(),
                signal_kind: SignalKind::Long,
                condition: qrpc_core_ir::ScalarExpr::RawText {
                    source: "long".into(),
                },
            }],
            Some(0.05),
            1.0,
            None,
        );
        let mut core_ir = core_ir;
        core_ir.agent_policies[0].rebalance_symbols = vec![btc.as_str().to_string()];
        core_ir.agent_policies[0].rebalance_schedule = Some(RebalanceSchedule::Weekly);

        let signals = vec![IntentSignal {
            signal_id: "s_btc".into(),
            intent_id: "intent_btc".into(),
            kind: IntentKind::LongTermBuy,
            exchange_scope: vec![Exchange::Binance],
            symbol_scope: vec![btc],
            side: SignalSide::Long,
            strength: 0.9,
            confidence: 1.0,
            reference_price: Some(50_000.0),
            derived_metrics: BTreeMap::new(),
            reason: "btc selected".into(),
            triggered_at_ms: 10,
            ttl_ms: 1000,
            trace_id: "trace".into(),
        }];
        let last_rebalance_at_ms = BTreeMap::from([("agent_rebalance".to_string(), 1_000_u64)]);

        let skipped = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &signals,
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &last_rebalance_at_ms,
            now_ms: 1_000 + 6 * 86_400_000,
            trace_id: "trace",
        });
        assert!(skipped.decisions.is_empty());

        let due = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &signals,
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &last_rebalance_at_ms,
            now_ms: 1_000 + 7 * 86_400_000,
            trace_id: "trace",
        });
        assert_eq!(due.decisions.len(), 1);
        assert!(due
            .evaluated_rebalance_agent_ids
            .contains("agent_rebalance"));
    }

    fn sample_long_signal(
        intent_id: &str,
        symbol: Symbol,
        reference_price: f64,
        strength: f64,
    ) -> IntentSignal {
        IntentSignal {
            signal_id: format!("signal_{intent_id}"),
            intent_id: intent_id.into(),
            kind: IntentKind::LongTermBuy,
            exchange_scope: vec![Exchange::Binance],
            symbol_scope: vec![symbol],
            side: SignalSide::Long,
            strength,
            confidence: 1.0,
            reference_price: Some(reference_price),
            derived_metrics: BTreeMap::new(),
            reason: "selected".into(),
            triggered_at_ms: 10,
            ttl_ms: 1000,
            trace_id: "trace".into(),
        }
    }
}
