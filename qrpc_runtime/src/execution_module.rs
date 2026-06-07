use crate::fill_engine::{FillEngine, MarketState};
use qrpc_core::{
    CoreStrategyIr, DecisionStatus, ExecutionPlan, FillResult, NormalizedMarketData, OrderSide,
    OrderType, PortfolioState, RiskDecision, RuntimeEvent, RuntimeEventType, SimOrder, Symbol,
    TimeInForce,
};
use qrpc_core_ir::{CoreTimeInForce, ExecutionSizingKind};
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ExecutionPlanningRequest<'a> {
    pub risk_decisions: &'a [RiskDecision],
    pub core_ir: &'a CoreStrategyIr,
    pub normalized_data: &'a [NormalizedMarketData],
    pub portfolio: &'a PortfolioState,
    pub now_ms: u64,
    pub trace_id: &'a str,
}

#[derive(Debug, Clone)]
pub struct ExecutionPlanningOutput {
    pub plans: Vec<ExecutionPlan>,
    pub events: Vec<RuntimeEvent>,
}

/// v2.3.2: execution planning is read-only and stateless.
pub trait ExecutionPlanner: Send {
    fn provider_key(&self) -> &'static str {
        "builtin.execution.paper"
    }
    fn plan_execution(&self, request: ExecutionPlanningRequest<'_>) -> ExecutionPlanningOutput;
}

/// v2.3.2: execution submission mutates portfolio/fill state.
pub trait ExecutionSubmitter: Send {
    fn submit_plan(
        &mut self,
        plan: &ExecutionPlan,
        normalized_data: &[NormalizedMarketData],
        portfolio: &mut PortfolioState,
        now_ms: u64,
        trace_id: &str,
    ) -> FillResult;
    fn on_market_update(
        &mut self,
        normalized_data: &[NormalizedMarketData],
        portfolio: &mut PortfolioState,
        now_ms: u64,
        trace_id: &str,
    ) -> FillResult;
    fn set_execution_assumptions(&mut self, _assumptions: crate::slippage::ExecutionAssumptions) {}
}

/// Compatibility alias for the pre-v2.3.2 combined provider trait.
pub trait ExecutionModuleProvider: ExecutionPlanner + ExecutionSubmitter {}
impl<T: ExecutionPlanner + ExecutionSubmitter> ExecutionModuleProvider for T {}

#[derive(Debug, Clone, Default)]
pub struct BuiltinExecutionModule {
    fill_engine: FillEngine,
}

impl ExecutionPlanner for BuiltinExecutionModule {
    fn plan_execution(&self, request: ExecutionPlanningRequest<'_>) -> ExecutionPlanningOutput {
        let quote_map = quote_price_map(request.normalized_data);
        let n = request.risk_decisions.len();
        let mut plans = Vec::with_capacity(n);
        let mut events = Vec::with_capacity(n);
        let equity = portfolio_equity(request.portfolio).max(0.0);
        let execution_semantics = resolve_execution_semantics(request.core_ir);

        for decision in request
            .risk_decisions
            .iter()
            .filter(|item| !matches!(item.status, DecisionStatus::Reject))
        {
            let orders = if let Some(target_decision) = &decision.adjusted_portfolio_target_decision
            {
                build_portfolio_target_orders(
                    decision,
                    target_decision,
                    &quote_map,
                    request.portfolio,
                    equity,
                    &execution_semantics,
                )
            } else {
                build_action_orders(decision, &quote_map, equity, &execution_semantics)
            };

            if orders.is_empty() {
                continue;
            }

            let plan = ExecutionPlan {
                plan_id: format!("plan-{}", decision.risk_decision_id),
                source_risk_decision_id: decision.risk_decision_id.clone(),
                orders,
                created_at_ms: request.now_ms,
                trace_id: request.trace_id.to_string(),
            };
            events.push(RuntimeEvent {
                event_id: format!("evt-plan-{}-{}", plan.plan_id, request.now_ms),
                event_type: RuntimeEventType::ExecutionPlanned,
                trace_id: request.trace_id.to_string(),
                source_id: decision.risk_decision_id.clone(),
                ts_ms: request.now_ms,
                payload: build_execution_plan_payload(
                    self.provider_key(),
                    decision,
                    &plan,
                    equity,
                    &execution_semantics,
                ),
            });
            plans.push(plan);
        }

        ExecutionPlanningOutput { plans, events }
    }
}

impl ExecutionSubmitter for BuiltinExecutionModule {
    fn submit_plan(
        &mut self,
        plan: &ExecutionPlan,
        normalized_data: &[NormalizedMarketData],
        portfolio: &mut PortfolioState,
        now_ms: u64,
        trace_id: &str,
    ) -> FillResult {
        let market_prices = quote_market_state_map(normalized_data);
        self.fill_engine
            .submit_plan(plan, &market_prices, portfolio, now_ms, trace_id)
    }

    fn on_market_update(
        &mut self,
        normalized_data: &[NormalizedMarketData],
        portfolio: &mut PortfolioState,
        now_ms: u64,
        trace_id: &str,
    ) -> FillResult {
        let market_prices = quote_market_state_map(normalized_data);
        self.fill_engine
            .on_market_update(&market_prices, portfolio, now_ms, trace_id)
    }

    fn set_execution_assumptions(&mut self, assumptions: crate::slippage::ExecutionAssumptions) {
        self.fill_engine.set_assumptions(assumptions);
    }
}

fn build_action_orders(
    decision: &RiskDecision,
    quote_map: &BTreeMap<(qrpc_core::Exchange, Symbol), f64>,
    equity: f64,
    execution_semantics: &ExecutionSemantics,
) -> Vec<SimOrder> {
    decision
        .adjusted_actions
        .iter()
        .filter_map(|action| {
            let quote_price = quote_map
                .get(&(action.exchange.clone(), decision.symbol.clone()))
                .copied()
                .unwrap_or(action.reference_price);
            let notional_budget = equity * action.quantity_ratio.max(0.0);
            let quantity = if quote_price > 0.0 {
                notional_budget / quote_price
            } else {
                0.0
            };
            // v2.1.0: warn on NaN/Inf quantity instead of silently dropping it.
            if quantity.is_nan() || quantity.is_infinite() {
                eprintln!(
                    "[execution] 璀﹀憡: 璁＄畻鍑虹殑鏁伴噺鏃犳晥 (NaN/Inf), symbol={:?}, action={:?}",
                    decision.symbol, action
                );
            }
            (quantity.is_finite() && quantity > 0.0).then(|| {
                let should_rest = quote_map.is_empty() && matches!(action.side, OrderSide::Buy);
                let order_type = if should_rest {
                    OrderType::Limit
                } else {
                    OrderType::Market
                };
                let limit_price = if should_rest {
                    Some(quote_price * 0.98)
                } else {
                    None
                };
                SimOrder {
                    order_id: format!("ord-{}-{:?}", decision.risk_decision_id, action.exchange),
                    exchange: action.exchange.clone(),
                    symbol: decision.symbol.clone(),
                    side: action.side.clone(),
                    order_type,
                    quantity,
                    limit_price,
                    time_in_force: execution_semantics.time_in_force.clone(),
                    allow_partial: false,
                    reference_price: quote_price,
                    slippage_bps: execution_semantics.slippage_bps,
                    fee_bps: execution_semantics.fee_bps,
                    strategy_tag: action.strategy_tag.clone(),
                }
            })
        })
        .collect()
}

fn build_portfolio_target_orders(
    decision: &RiskDecision,
    target_decision: &qrpc_core::PortfolioTargetDecision,
    quote_map: &BTreeMap<(qrpc_core::Exchange, Symbol), f64>,
    portfolio: &PortfolioState,
    equity: f64,
    execution_semantics: &ExecutionSemantics,
) -> Vec<SimOrder> {
    target_decision
        .target
        .target_weights
        .iter()
        .filter_map(|item| {
            let quote_price = quote_map
                .get(&(item.exchange.clone(), item.symbol.clone()))
                .copied()
                .unwrap_or(item.reference_price);
            let current_weight =
                current_position_ratio(portfolio, &item.exchange, &item.symbol, quote_price);
            let delta_weight = item.target_weight - current_weight;
            if delta_weight.abs() <= 0.01 || !quote_price.is_finite() || quote_price <= 0.0 {
                return None;
            }
            let notional_budget = equity * delta_weight.abs();
            let quantity = notional_budget / quote_price;
            (quantity.is_finite() && quantity > 0.0).then(|| SimOrder {
                order_id: format!(
                    "ord-{}-{}-{:?}",
                    decision.risk_decision_id,
                    item.symbol.as_str(),
                    item.exchange
                ),
                exchange: item.exchange.clone(),
                symbol: item.symbol.clone(),
                side: if delta_weight > 0.0 {
                    OrderSide::Buy
                } else {
                    OrderSide::Sell
                },
                order_type: OrderType::Market,
                quantity,
                limit_price: None,
                time_in_force: execution_semantics.time_in_force.clone(),
                allow_partial: false,
                reference_price: quote_price,
                slippage_bps: execution_semantics.slippage_bps,
                fee_bps: execution_semantics.fee_bps,
                strategy_tag: format!(
                    "portfolio_target:{}:{}",
                    target_decision.target.allocation_kind, target_decision.target_id
                ),
            })
        })
        .collect()
}

#[derive(Debug, Clone)]
struct ExecutionSemantics {
    execution_id: String,
    venue_kind: String,
    sizing_semantics: String,
    slippage_bps: f64,
    fee_bps: f64,
    time_in_force: TimeInForce,
}

fn build_execution_plan_payload(
    provider_key: &str,
    decision: &RiskDecision,
    plan: &ExecutionPlan,
    equity: f64,
    execution_semantics: &ExecutionSemantics,
) -> serde_json::Value {
    let sizing_source = if decision.adjusted_portfolio_target_decision.is_some() {
        "portfolio_target_diff"
    } else {
        "risk_adjusted_actions"
    };
    let order_previews = plan
        .orders
        .iter()
        .map(|order| {
            let order_type_decision_reason = match order.order_type {
                OrderType::Market => {
                    if decision.adjusted_portfolio_target_decision.is_some() {
                        "rebalance_diff_executes_at_market"
                    } else {
                        "live_quote_available_or_direct_market_execution"
                    }
                }
                OrderType::Limit => "fallback_to_resting_limit_without_live_quote",
                OrderType::StopLoss
                | OrderType::StopLossLimit
                | OrderType::TakeProfit
                | OrderType::TakeProfitLimit => "plugin_execution_algorithm",
            };
            json!({
                "order_id": order.order_id,
                "side": format!("{:?}", order.side),
                "qty": order.quantity,
                "order_type": format!("{:?}", order.order_type),
                "limit_price": order.limit_price,
                "strategy_tag": order.strategy_tag,
                "sizing_source": sizing_source,
                "order_type_decision_reason": order_type_decision_reason,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "provider_key": provider_key,
        "orders": plan.orders.len(),
        "equity_base": equity,
        "sizing_semantics": execution_semantics.sizing_semantics,
        "execution_id": execution_semantics.execution_id,
        "execution_venue_kind": execution_semantics.venue_kind,
        "execution_slippage_bps": execution_semantics.slippage_bps,
        "time_in_force": format!("{:?}", execution_semantics.time_in_force),
        "sizing_source": sizing_source,
        "order_type_decision_reason": if plan.orders.iter().any(|order| matches!(order.order_type, OrderType::Limit)) {
            "plan_contains_resting_limit_orders"
        } else {
            "plan_executes_immediately_when_submitted"
        },
        "explanation_summary": format!(
            "Execution planned {} order(s) from {} using {} sizing.",
            plan.orders.len(),
            sizing_source,
            execution_semantics.sizing_semantics
        ),
        "order_previews": order_previews,
    })
}

fn resolve_execution_semantics(core_ir: &CoreStrategyIr) -> ExecutionSemantics {
    let execution = &core_ir.execution;
    let sizing_semantics = match execution.sizing_kind {
        ExecutionSizingKind::EquityNotionalRatio => "equity_notional_ratio",
    };

    ExecutionSemantics {
        execution_id: execution.execution_id.clone(),
        venue_kind: execution.venue_kind.clone(),
        sizing_semantics: sizing_semantics.to_string(),
        slippage_bps: execution.slippage_bps,
        fee_bps: execution.taker_fee_bps,
        time_in_force: parse_time_in_force(&execution.time_in_force),
    }
}

fn parse_time_in_force(value: &CoreTimeInForce) -> TimeInForce {
    match value {
        CoreTimeInForce::Ioc => TimeInForce::Ioc,
        CoreTimeInForce::Fok => TimeInForce::Fok,
        CoreTimeInForce::Gtc => TimeInForce::Gtc,
    }
}

fn portfolio_equity(portfolio: &PortfolioState) -> f64 {
    portfolio.cash_balance + portfolio.total_net_notional
}

fn current_position_ratio(
    portfolio: &PortfolioState,
    exchange: &qrpc_core::Exchange,
    symbol: &Symbol,
    reference_price: f64,
) -> f64 {
    let equity = portfolio_equity(portfolio).abs().max(1.0);
    if !reference_price.is_finite() || reference_price <= 0.0 {
        return 0.0;
    }
    let notional = portfolio
        .positions
        .iter()
        .find(|position| &position.exchange == exchange && &position.symbol == symbol)
        .map(|position| position.net_qty.max(0.0) * reference_price)
        .unwrap_or(0.0);
    (notional / equity).max(0.0)
}

fn quote_price_map(
    normalized_data: &[NormalizedMarketData],
) -> BTreeMap<(qrpc_core::Exchange, Symbol), f64> {
    let mut map = BTreeMap::new();
    for item in normalized_data {
        match item {
            NormalizedMarketData::Quote(quote) => {
                map.insert(
                    (quote.exchange.clone(), quote.symbol.clone()),
                    quote.mid_price,
                );
            }
            NormalizedMarketData::KlineSeries(series) => {
                // Use last bar's close as fallback mid-price so orders become MARKET
                // in mock/backtest environments where Quote data is unavailable
                if let Some(last_bar) = series.bars.last() {
                    map.entry((series.exchange.clone(), series.symbol.clone()))
                        .or_insert(last_bar.close);
                }
            }
        }
    }
    map
}

fn quote_market_state_map(
    normalized_data: &[NormalizedMarketData],
) -> BTreeMap<(qrpc_core::Exchange, Symbol), MarketState> {
    normalized_data
        .iter()
        .filter_map(|item| match item {
            NormalizedMarketData::Quote(quote) => Some((
                (quote.exchange.clone(), quote.symbol.clone()),
                MarketState {
                    price: quote.mid_price,
                    bid_price: Some(quote.best_bid),
                    ask_price: Some(quote.best_ask),
                    buy_liquidity: quote.ask_size,
                    sell_liquidity: quote.bid_size,
                },
            )),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod test_harness;
