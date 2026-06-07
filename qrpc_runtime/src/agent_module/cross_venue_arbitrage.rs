use super::{available_position_ratio, SPREAD_MULTIPLIER};
use qrpc_core::{
    AgentDecision, CoreStrategyIr, Exchange, IntentSignal, OrderSide, PortfolioState,
    ProposedAction, SignalSide, Symbol,
};
use qrpc_core_ir::AgentPolicy;
use std::collections::BTreeMap;

pub(super) fn build_arb_agent_decision(
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
    const DEFAULT_SPREAD_TRIGGER_BPS: f64 = 50.0;
    let spread_trigger = (agent
        .spread_trigger_bps
        .unwrap_or(DEFAULT_SPREAD_TRIGGER_BPS)
        / 10_000.0)
        .max(total_cost_buffer);
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
    let quantity_ratio = (spread * SPREAD_MULTIPLIER)
        .clamp(0.1, max_quantity_ratio)
        .min(available_sell_ratio);
    if !quantity_ratio.is_finite() || quantity_ratio <= 0.01 {
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
    const DEFAULT_SPREAD_TRIGGER_BPS: f64 = 50.0;
    let spread_trigger = (agent
        .spread_trigger_bps
        .unwrap_or(DEFAULT_SPREAD_TRIGGER_BPS)
        / 10_000.0)
        .max(total_cost_buffer);
    if spread <= spread_trigger {
        return None;
    }

    let max_quantity_ratio = agent.max_quantity_ratio.clamp(0.01, 1.0);
    let available_sell_ratio =
        available_position_ratio(portfolio, &sell_exchange, &target_symbol, sell_price);
    let quantity_ratio = (spread * SPREAD_MULTIPLIER)
        .clamp(0.1, max_quantity_ratio)
        .min(available_sell_ratio);
    if !quantity_ratio.is_finite() || quantity_ratio <= 0.01 {
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
