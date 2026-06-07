use super::{
    build_signal_kind_index, current_position_ratio, signal_score, DEFAULT_DECISION_THRESHOLD,
    MIN_QUANTITY_RATIO,
};
use qrpc_core::{
    AgentDecision, CoreStrategyIr, Exchange, IntentKind, IntentSignal, PortfolioState,
    PortfolioTarget, PortfolioTargetDecision, SignalSide, Symbol, TargetWeight,
};
use qrpc_core_ir::{AgentPolicy, RebalanceSchedule, SignalKind};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn portfolio_rebalance_due(
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

#[derive(Debug, Clone)]
struct RebalanceSymbolPlan {
    symbol: Symbol,
    exchange: Exchange,
    reference_price: f64,
    score: f64,
    target_weight: f64,
    current_weight: f64,
}

pub(super) fn build_portfolio_rebalance_decision(
    agent: &AgentPolicy,
    signals: &[IntentSignal],
    core_ir: &CoreStrategyIr,
    portfolio: &PortfolioState,
    now_ms: u64,
    trace_id: &str,
) -> Option<AgentDecision> {
    let signal_kind_index = build_signal_kind_index(core_ir);
    let weighted_signals = signals
        .iter()
        .filter(|item| {
            signal_kind_index.get(&item.intent_id) != Some(&SignalKind::Observe)
                && !matches!(item.kind, IntentKind::QuoteObserve)
        })
        .collect::<Vec<_>>();
    if weighted_signals.is_empty() {
        return None;
    }

    let decision_threshold = agent
        .decision_threshold
        .unwrap_or(DEFAULT_DECISION_THRESHOLD)
        .max(0.0);
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
        if entry.1 <= 0.0 && reference_price.is_finite() && reference_price > 0.0 {
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
                        .find(|position| {
                            position.symbol == symbol
                                && position.mark_price.is_finite()
                                && position.mark_price > 0.0
                        })
                        .map(|position| position.mark_price)
                })
                .or_else(|| {
                    portfolio
                        .positions
                        .iter()
                        .find(|position| {
                            position.symbol == symbol
                                && position.avg_entry_price.is_finite()
                                && position.avg_entry_price > 0.0
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
    if net_delta <= MIN_QUANTITY_RATIO {
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
            } else {
                let count = selected.len() as f64;
                if count > 0.0 {
                    let eq = (1.0 / count).min(max_quantity_ratio);
                    for plan in selected {
                        plan.target_weight = eq;
                    }
                }
            }
        }
        "score_weight" => {
            let selected = plans
                .iter_mut()
                .filter(|plan| {
                    selected_symbols.contains(&plan.symbol)
                        && plan.score.is_finite()
                        && plan.score > 0.0
                })
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
