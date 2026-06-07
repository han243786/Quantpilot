use super::{
    available_position_ratio, build_signal_kind_index, signal_score, DEFAULT_DECISION_THRESHOLD,
    MIN_QUANTITY_RATIO,
};
use qrpc_core::{
    AgentDecision, CoreStrategyIr, Exchange, IntentKind, IntentSignal, OrderSide, PortfolioState,
    ProposedAction, SignalSide, Symbol,
};
use qrpc_core_ir::{AgentPolicy, SignalKind};
use std::collections::BTreeMap;

pub(super) fn build_weighted_agent_decisions(
    agent: &AgentPolicy,
    signals: &[IntentSignal],
    core_ir: &CoreStrategyIr,
    portfolio: &PortfolioState,
    now_ms: u64,
    trace_id: &str,
) -> Vec<AgentDecision> {
    let signal_kind_index = build_signal_kind_index(core_ir);
    let weighted_signals: Vec<&IntentSignal> = signals
        .iter()
        .filter(|item| {
            signal_kind_index.get(&item.intent_id) != Some(&SignalKind::Observe)
                && !matches!(item.kind, IntentKind::QuoteObserve)
        })
        .collect();
    if weighted_signals.is_empty() {
        return Vec::new();
    }

    let decision_threshold = agent
        .decision_threshold
        .unwrap_or(DEFAULT_DECISION_THRESHOLD)
        .max(MIN_QUANTITY_RATIO);
    let max_quantity_ratio = agent.max_quantity_ratio.clamp(MIN_QUANTITY_RATIO, 1.0);

    let grouped = group_signals_by_symbol(&weighted_signals);

    let mut decisions = Vec::with_capacity(grouped.len());
    for ((exchange, symbol), symbol_signals) in grouped {
        let total_weight: f64 = symbol_signals.iter().map(|s| s.confidence.max(0.1)).sum();
        if total_weight <= f64::EPSILON {
            continue;
        }
        let net: f64 = symbol_signals
            .iter()
            .map(|s| signal_score(s) * s.confidence.max(0.1))
            .sum::<f64>()
            / total_weight;

        if net.abs() < decision_threshold {
            continue;
        }

        let reference_price = symbol_signals
            .iter()
            .find_map(|s| s.reference_price)
            .unwrap_or(50_000.0);

        let available_sell_ratio =
            available_position_ratio(portfolio, &exchange, &symbol, reference_price);
        let quantity_ratio = if net > 0.0 {
            net.abs().clamp(decision_threshold, max_quantity_ratio)
        } else {
            net.abs()
                .clamp(decision_threshold, max_quantity_ratio)
                .min(available_sell_ratio)
        };
        if quantity_ratio <= MIN_QUANTITY_RATIO {
            continue;
        }

        decisions.push(AgentDecision {
            decision_id: format!("decision-{}-{}-{now_ms}", agent.agent_id, symbol.as_str()),
            agent_id: agent.agent_id.clone(),
            symbol: symbol.clone(),
            exchange_targets: vec![exchange.clone()],
            net_side: if net > 0.0 {
                SignalSide::Long
            } else {
                SignalSide::Short
            },
            net_strength: net,
            portfolio_target_decision: None,
            proposed_actions: vec![ProposedAction {
                exchange: exchange.clone(),
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
                "net_score {:.4}, signals={}, threshold={:.4}",
                net,
                symbol_signals.len(),
                decision_threshold
            ),
            produced_at_ms: now_ms,
            trace_id: trace_id.to_string(),
        });
    }

    decisions
}

fn group_signals_by_symbol<'a>(
    signals: &[&'a IntentSignal],
) -> BTreeMap<(Exchange, Symbol), Vec<&'a IntentSignal>> {
    let mut map: BTreeMap<(Exchange, Symbol), Vec<&'a IntentSignal>> = BTreeMap::new();
    for signal in signals {
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
        map.entry((exchange, symbol)).or_default().push(*signal);
    }
    map
}
