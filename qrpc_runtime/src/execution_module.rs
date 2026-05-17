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

/// v2.3.2: ISP拆分 — 执行计划 (只读, 无状态变更)
pub trait ExecutionPlanner: Send {
    fn provider_key(&self) -> &'static str {
        "builtin.execution.paper"
    }
    fn plan_execution(&self, request: ExecutionPlanningRequest<'_>) -> ExecutionPlanningOutput;
}

/// v2.3.2: ISP拆分 — 执行提交 (有状态变更: 提交计划/市场更新/假设设置)
pub trait ExecutionSubmitter: Send {
    fn submit_plan(
        &mut self, plan: &ExecutionPlan, normalized_data: &[NormalizedMarketData],
        portfolio: &mut PortfolioState, now_ms: u64, trace_id: &str,
    ) -> FillResult;
    fn on_market_update(
        &mut self, normalized_data: &[NormalizedMarketData],
        portfolio: &mut PortfolioState, now_ms: u64, trace_id: &str,
    ) -> FillResult;
    fn set_execution_assumptions(&mut self, _assumptions: crate::slippage::ExecutionAssumptions) {}
}

/// 兼容别名: v2.3.2 前使用的组合 trait。新代码应分别使用 ExecutionPlanner 和 ExecutionSubmitter。
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
        &mut self, plan: &ExecutionPlan, normalized_data: &[NormalizedMarketData],
        portfolio: &mut PortfolioState, now_ms: u64, trace_id: &str,
    ) -> FillResult {
        let market_prices = quote_market_state_map(normalized_data);
        self.fill_engine.submit_plan(plan, &market_prices, portfolio, now_ms, trace_id)
    }

    fn on_market_update(
        &mut self, normalized_data: &[NormalizedMarketData],
        portfolio: &mut PortfolioState, now_ms: u64, trace_id: &str,
    ) -> FillResult {
        let market_prices = quote_market_state_map(normalized_data);
        self.fill_engine.on_market_update(&market_prices, portfolio, now_ms, trace_id)
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
            // v2.1.0: NaN/Inf 数量告警，不再静默丢弃
            if quantity.is_nan() || quantity.is_infinite() {
                eprintln!("[execution] 警告: 计算出的数量无效 (NaN/Inf), symbol={:?}, action={:?}", decision.symbol, action);
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
mod tests {
    use super::*;
    use qrpc_core::{
        DataQualitySnapshot, Exchange, ExecutionStatus, MarketType, PortfolioTarget,
        PortfolioTargetDecision, QuoteSnapshot, RiskDecisionMode, RiskReasonCode,
        SourceStatus, TargetWeight,
    };
    use qrpc_core_ir::{CoreMetadata, CoreSourceKind, CoreTimeInForce, ExecutionRule};
    use std::collections::BTreeMap;

    fn sample_quote(mid_price: f64) -> NormalizedMarketData {
        NormalizedMarketData::Quote(QuoteSnapshot {
            data_id: "binance_btc_quote".into(),
            exchange: Exchange::Binance,
            symbol: Symbol::BtcUsdt,
            market_type: MarketType::Spot,
            best_bid: mid_price - 5.0,
            best_ask: mid_price + 5.0,
            bid_size: 10.0,
            ask_size: 10.0,
            mid_price,
            ts_ms: 10,
            source_latency_ms: 0,
            source_status: SourceStatus::Healthy,
            data_quality: DataQualitySnapshot::default(),
        })
    }

    fn sample_core_ir() -> CoreStrategyIr {
        CoreStrategyIr {
            ir_version: qrpc_core::CORE_IR_V1_VERSION.to_string(),
            metadata: CoreMetadata {
                strategy_id: "execution_test".into(),
                name: "Execution Test".into(),
                source_kind: CoreSourceKind::RuntimeProtocol,
            },
            data_bindings: vec![],
            indicators: vec![],
            signal_rules: vec![],
            agent_policies: vec![],
            risk_policies: vec![],
            edges: vec![],
            execution: ExecutionRule {

                execution_id: "execution.paper".into(),
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
    fn builtin_execution_module_plans_orders_from_risk_decisions() {
        let module = BuiltinExecutionModule::default();
        let core_ir = sample_core_ir();
        let decision = RiskDecision {
            risk_decision_id: "risk_1".into(),
            risk_id: "risk_global".into(),
            agent_decision_id: "decision_1".into(),
            symbol: Symbol::BtcUsdt,
            status: DecisionStatus::Approve,
            mode: RiskDecisionMode::Normal,
            adjusted_portfolio_target_decision: None,
            adjusted_actions: vec![qrpc_core::ProposedAction {
                exchange: Exchange::Binance,
                side: OrderSide::Buy,
                quantity_ratio: 0.2,
                reference_price: 50_000.0,
                strategy_tag: "test".into(),
            }],
            reason_codes: vec![RiskReasonCode::WithinLimit],
            reason_text: "approved".into(),
            produced_at_ms: 10,
            trace_id: "trace".into(),
        };

        let output = module.plan_execution(ExecutionPlanningRequest {
            risk_decisions: &[decision],
            core_ir: &core_ir,
            normalized_data: &[sample_quote(50_000.0)],
            portfolio: &PortfolioState::new(100_000.0, 0),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.plans.len(), 1);
        assert_eq!(output.events.len(), 1);
        assert_eq!(
            output.events[0].payload["explanation_summary"],
            "Execution planned 1 order(s) from risk_adjusted_actions using equity_notional_ratio sizing."
        );
        assert_eq!(
            output.events[0].payload["sizing_source"],
            "risk_adjusted_actions"
        );
        assert_eq!(
            output.events[0].payload["order_previews"][0]["order_type_decision_reason"],
            "live_quote_available_or_direct_market_execution"
        );
    }

    #[test]
    fn builtin_execution_module_submits_plan_through_fill_engine() {
        let mut module = BuiltinExecutionModule::default();
        let plan = ExecutionPlan {
            plan_id: "plan_1".into(),
            source_risk_decision_id: "risk_1".into(),
            orders: vec![SimOrder {
                order_id: "ord_1".into(),
                exchange: Exchange::Binance,
                symbol: Symbol::BtcUsdt,
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                quantity: 1.0,
                limit_price: None,
                time_in_force: TimeInForce::Gtc,
                allow_partial: false,
                reference_price: 50_000.0,
                slippage_bps: 5.0,
                fee_bps: 10.0,
                strategy_tag: "test".into(),
            }],
            created_at_ms: 10,
            trace_id: "trace".into(),
        };
        let mut portfolio = PortfolioState::new(100_000.0, 0);

        let result = module.submit_plan(
            &plan,
            &[sample_quote(50_000.0)],
            &mut portfolio,
            10,
            "trace",
        );

        assert!(matches!(result.status, ExecutionStatus::Filled));
        assert!(result.open_orders.is_empty());
    }

    #[test]
    fn builtin_execution_module_uses_risk_decision_symbol_for_quotes_and_orders() {
        let module = BuiltinExecutionModule::default();
        let core_ir = sample_core_ir();
        let eth = Symbol::parse("ETHUSDT");
        let decision = RiskDecision {
            risk_decision_id: "risk_eth".into(),
            risk_id: "risk_global".into(),
            agent_decision_id: "decision_eth".into(),
            symbol: eth.clone(),
            status: DecisionStatus::Approve,
            mode: RiskDecisionMode::Normal,
            adjusted_portfolio_target_decision: None,
            adjusted_actions: vec![qrpc_core::ProposedAction {
                exchange: Exchange::Binance,
                side: OrderSide::Buy,
                quantity_ratio: 0.2,
                reference_price: 50_000.0,
                strategy_tag: "test".into(),
            }],
            reason_codes: vec![RiskReasonCode::WithinLimit],
            reason_text: "approved".into(),
            produced_at_ms: 10,
            trace_id: "trace".into(),
        };
        let eth_quote = NormalizedMarketData::Quote(QuoteSnapshot {
            data_id: "binance_eth_quote".into(),
            exchange: Exchange::Binance,
            symbol: eth.clone(),
            market_type: MarketType::Spot,
            best_bid: 3_995.0,
            best_ask: 4_005.0,
            bid_size: 10.0,
            ask_size: 10.0,
            mid_price: 4_000.0,
            ts_ms: 10,
            source_latency_ms: 0,
            source_status: SourceStatus::Healthy,
            data_quality: DataQualitySnapshot::default(),
        });

        let output = module.plan_execution(ExecutionPlanningRequest {
            risk_decisions: &[decision],
            core_ir: &core_ir,
            normalized_data: &[eth_quote],
            portfolio: &PortfolioState::new(100_000.0, 0),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.plans.len(), 1);
        assert_eq!(output.plans[0].orders[0].symbol, eth);
        assert!((output.plans[0].orders[0].quantity - 5.0).abs() < 1e-9);
    }

    #[test]
    fn builtin_execution_module_builds_basket_from_portfolio_target_diff() {
        let module = BuiltinExecutionModule::default();
        let core_ir = sample_core_ir();
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let mut portfolio = PortfolioState::new(100_000.0, 0);
        portfolio.positions.push(qrpc_core::Position {
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

        let decision = RiskDecision {
            risk_decision_id: "risk_rebalance".into(),
            risk_id: "risk_global".into(),
            agent_decision_id: "decision_rebalance".into(),
            symbol: btc.clone(),
            status: DecisionStatus::Approve,
            mode: RiskDecisionMode::Normal,
            adjusted_portfolio_target_decision: Some(PortfolioTargetDecision {
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
                reason: "equal-weight rebalance".into(),
            }),
            adjusted_actions: Vec::new(),
            reason_codes: vec![RiskReasonCode::WithinLimit],
            reason_text: "approved".into(),
            produced_at_ms: 10,
            trace_id: "trace".into(),
        };
        let eth_quote = NormalizedMarketData::Quote(QuoteSnapshot {
            data_id: "binance_eth_quote".into(),
            exchange: Exchange::Binance,
            symbol: eth.clone(),
            market_type: MarketType::Spot,
            best_bid: 3_995.0,
            best_ask: 4_005.0,
            bid_size: 10.0,
            ask_size: 10.0,
            mid_price: 4_000.0,
            ts_ms: 10,
            source_latency_ms: 0,
            source_status: SourceStatus::Healthy,
            data_quality: DataQualitySnapshot::default(),
        });

        let output = module.plan_execution(ExecutionPlanningRequest {
            risk_decisions: &[decision],
            core_ir: &core_ir,
            normalized_data: &[sample_quote(50_000.0), eth_quote],
            portfolio: &portfolio,
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.plans.len(), 1);
        assert_eq!(output.plans[0].orders.len(), 2);
        let btc_order = output.plans[0]
            .orders
            .iter()
            .find(|order| order.symbol == btc)
            .expect("btc basket order");
        let eth_order = output.plans[0]
            .orders
            .iter()
            .find(|order| order.symbol == eth)
            .expect("eth basket order");
        assert_eq!(btc_order.side, OrderSide::Sell);
        assert_eq!(eth_order.side, OrderSide::Buy);
        assert!((btc_order.quantity - 0.4).abs() < 1e-9);
        assert!((eth_order.quantity - 12.5).abs() < 1e-9);
        assert_eq!(
            output.events[0].payload["sizing_source"],
            "portfolio_target_diff"
        );
    }
}
