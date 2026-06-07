use crate::slippage::{
    compute_fill_price, ExecutionAssumptions, ExtendedMarketState, SlippageModel,
};
use qrpc_core::{
    Exchange, ExecutionPlan, ExecutionStatus, FillReport, FillResult, OpenOrder, OrderSide,
    OrderType, PortfolioState, Position, RuntimeEvent, RuntimeEventType, SimOrder, Symbol,
    TimeInForce,
};
use serde_json::json;
use std::collections::BTreeMap;

mod event_projection_wave;
mod fill_report_execution_helpers;
mod portfolio_reservation_accounting;
use event_projection_wave::{cancel_event, fill_event, open_event, partial_event, reject_event};
use fill_report_execution_helpers::{
    build_fill_report, build_fill_report_from_open, can_rest, executable_qty,
    execution_assumptions_for_order, execution_status_for, market_state_for, sim_order_from_open,
};
use portfolio_reservation_accounting::{
    available_position_qty, available_sell_qty_for_order, cash_limited_executable_qty,
    release_reservation_for_fill, reservation_for_order, sync_portfolio_reservations,
};

#[derive(Debug, Clone, Copy)]
pub struct MarketState {
    pub price: f64,
    pub bid_price: Option<f64>,
    pub ask_price: Option<f64>,
    pub buy_liquidity: f64,
    pub sell_liquidity: f64,
}

impl Default for MarketState {
    fn default() -> Self {
        Self {
            price: 0.0,
            bid_price: None,
            ask_price: None,
            buy_liquidity: f64::MAX,
            sell_liquidity: f64::MAX,
        }
    }
}

impl MarketState {
    pub fn from_mid_price(price: f64) -> Self {
        Self {
            price,
            bid_price: None,
            ask_price: None,
            buy_liquidity: f64::MAX,
            sell_liquidity: f64::MAX,
        }
    }

    #[allow(dead_code)]
    pub fn from_quote(bid: f64, ask: f64) -> Self {
        Self {
            price: (bid + ask) / 2.0,
            bid_price: Some(bid),
            ask_price: Some(ask),
            buy_liquidity: f64::MAX,
            sell_liquidity: f64::MAX,
        }
    }

    fn to_extended(self, volatility: f64, timeframe_minutes: u64) -> ExtendedMarketState {
        match (self.bid_price, self.ask_price) {
            (Some(bid), Some(ask)) => {
                ExtendedMarketState::from_quote(bid, ask, self.buy_liquidity, self.sell_liquidity)
            }
            _ => ExtendedMarketState::from_mid_price(
                self.price,
                self.buy_liquidity,
                self.sell_liquidity,
                volatility,
                timeframe_minutes,
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FillEngine {
    processed_results: BTreeMap<String, FillResult>,
    open_orders: BTreeMap<String, OpenOrder>,
    assumptions: ExecutionAssumptions,
}

impl Default for FillEngine {
    fn default() -> Self {
        Self {
            processed_results: BTreeMap::new(),
            open_orders: BTreeMap::new(),
            assumptions: ExecutionAssumptions::v1_0_7_compat(),
        }
    }
}

impl FillEngine {
    #[allow(dead_code)]
    pub fn with_assumptions(assumptions: ExecutionAssumptions) -> Self {
        Self {
            processed_results: BTreeMap::new(),
            open_orders: BTreeMap::new(),
            assumptions,
        }
    }

    pub fn set_assumptions(&mut self, assumptions: ExecutionAssumptions) {
        self.assumptions = assumptions;
    }

    #[allow(dead_code)]
    pub fn assumptions(&self) -> &ExecutionAssumptions {
        &self.assumptions
    }
}

struct RestingOrderOpenRequest<'a> {
    plan: &'a ExecutionPlan,
    order: &'a SimOrder,
    remaining_qty: f64,
    now_ms: u64,
    trace_id: &'a str,
}

impl FillEngine {
    pub fn submit_plan(
        &mut self,
        plan: &ExecutionPlan,
        market_state: &BTreeMap<(Exchange, Symbol), MarketState>,
        portfolio: &mut PortfolioState,
        now_ms: u64,
        trace_id: &str,
    ) -> FillResult {
        if let Some(existing) = self.processed_results.get(&plan.plan_id) {
            return existing.clone();
        }
        // v2.3.0: 缓存上限 1000, 超出时清理最旧条目防止内存泄漏
        const MAX_PROCESSED_RESULTS: usize = 1000;
        if self.processed_results.len() >= MAX_PROCESSED_RESULTS {
            let oldest: Vec<String> = self.processed_results.keys().take(100).cloned().collect();
            for key in oldest {
                self.processed_results.remove(&key);
            }
        }

        let mut fills = Vec::new();
        let mut pending_orders = Vec::new();
        let mut events = vec![RuntimeEvent {
            event_id: format!("evt-plan-accepted-{}-{now_ms}", plan.plan_id),
            event_type: RuntimeEventType::ExecutionPlanned,
            trace_id: trace_id.to_string(),
            source_id: plan.plan_id.clone(),
            ts_ms: now_ms,
            payload: json!({
                "status": "Accepted",
                "lifecycle_stage": "accepted",
                "orders": plan.orders.len(),
                "explanation_summary": format!("Accepted {} order(s) for execution.", plan.orders.len()),
            }),
        }];

        for order in &plan.orders {
            let state = market_state_for(
                market_state,
                order.exchange.clone(),
                order.symbol.clone(),
                order.reference_price,
            );
            let available_sell_qty = available_sell_qty_for_order(portfolio, order);
            let executable_qty =
                cash_limited_executable_qty(portfolio, order, &state, &self.assumptions)
                    .min(available_sell_qty);

            match order.time_in_force {
                TimeInForce::Fok => {
                    if executable_qty + f64::EPSILON < order.quantity {
                        events.push(reject_event(
                            &plan.plan_id,
                            &order.order_id,
                            "FOK 流动性不足",
                            now_ms,
                            trace_id,
                        ));
                        continue;
                    }
                    let fill = build_fill_report(
                        plan,
                        order,
                        executable_qty,
                        &state,
                        &self.assumptions,
                        now_ms,
                        trace_id,
                    );
                    apply_fill_to_portfolio(portfolio, &fill);
                    events.push(fill_event(&fill, &order.order_id, now_ms, trace_id));
                    fills.push(fill);
                }
                TimeInForce::Ioc => {
                    if !executable_qty.is_finite() || executable_qty <= 0.0 {
                        events.push(cancel_event(
                            &plan.plan_id,
                            &order.order_id,
                            "IOC expired without fill",
                            now_ms,
                            trace_id,
                        ));
                        continue;
                    }
                    if executable_qty + f64::EPSILON < order.quantity && !order.allow_partial {
                        events.push(reject_event(
                            &plan.plan_id,
                            &order.order_id,
                            "IOC requires full fill when partial disabled",
                            now_ms,
                            trace_id,
                        ));
                        continue;
                    }
                    let fill = build_fill_report(
                        plan,
                        order,
                        executable_qty,
                        &state,
                        &self.assumptions,
                        now_ms,
                        trace_id,
                    );
                    apply_fill_to_portfolio(portfolio, &fill);
                    if executable_qty + f64::EPSILON < order.quantity {
                        events.push(partial_event(
                            &plan.plan_id,
                            &order.order_id,
                            order.quantity - executable_qty,
                            now_ms,
                            trace_id,
                        ));
                    }
                    events.push(fill_event(&fill, &order.order_id, now_ms, trace_id));
                    fills.push(fill);
                }
                TimeInForce::Gtc => {
                    if executable_qty > 0.0 {
                        let fill = build_fill_report(
                            plan,
                            order,
                            executable_qty,
                            &state,
                            &self.assumptions,
                            now_ms,
                            trace_id,
                        );
                        apply_fill_to_portfolio(portfolio, &fill);
                        let remaining_qty = (order.quantity - executable_qty).max(0.0);
                        if remaining_qty > 1e-9 {
                            let open_order = self.try_open_resting_order(
                                RestingOrderOpenRequest {
                                    plan,
                                    order,
                                    remaining_qty,
                                    now_ms,
                                    trace_id,
                                },
                                portfolio,
                                &mut events,
                            );
                            if let Some(open_order) = open_order {
                                pending_orders.push(open_order);
                                events.push(partial_event(
                                    &plan.plan_id,
                                    &order.order_id,
                                    remaining_qty,
                                    now_ms,
                                    trace_id,
                                ));
                            }
                        }
                        events.push(fill_event(&fill, &order.order_id, now_ms, trace_id));
                        fills.push(fill);
                    } else if let Some(open_order) = self.try_open_resting_order(
                        RestingOrderOpenRequest {
                            plan,
                            order,
                            remaining_qty: order.quantity,
                            now_ms,
                            trace_id,
                        },
                        portfolio,
                        &mut events,
                    ) {
                        pending_orders.push(open_order);
                    }
                }
            }
        }

        sync_portfolio_reservations(portfolio, &self.open_orders);
        let status = execution_status_for(&fills, &pending_orders);
        let result = FillResult {
            plan_id: plan.plan_id.clone(),
            status,
            fills,
            open_orders: pending_orders,
            events,
        };
        self.processed_results
            .insert(plan.plan_id.clone(), result.clone());
        result
    }

    pub fn on_market_update(
        &mut self,
        market_state: &BTreeMap<(Exchange, Symbol), MarketState>,
        portfolio: &mut PortfolioState,
        now_ms: u64,
        trace_id: &str,
    ) -> FillResult {
        let mut fills = Vec::new();
        let mut events = Vec::new();
        let open_ids = self.open_orders.keys().cloned().collect::<Vec<_>>();

        for order_id in open_ids {
            let Some(open_order) = self.open_orders.get(&order_id).cloned() else {
                continue;
            };
            let state = market_state_for(
                market_state,
                open_order.exchange.clone(),
                open_order.symbol.clone(),
                open_order.reference_price,
            );
            let order = sim_order_from_open(&open_order, self.assumptions.taker_fee_bps);
            let available_sell_qty = available_sell_qty_for_order(portfolio, &order);
            let executable_qty = executable_qty(&order, &state).min(available_sell_qty);
            if !executable_qty.is_finite() || executable_qty <= 0.0 {
                continue;
            }

            let remaining_after = (open_order.remaining_qty - executable_qty).max(0.0);
            let fill = build_fill_report_from_open(
                &open_order,
                executable_qty,
                &state,
                &self.assumptions,
                now_ms,
                trace_id,
            );
            release_reservation_for_fill(portfolio, &open_order, executable_qty);
            apply_fill_to_portfolio(portfolio, &fill);

            if remaining_after > 1e-9 {
                let mut updated = open_order.clone();
                updated.remaining_qty = remaining_after;
                let reserve_price = updated.limit_price.unwrap_or(updated.reference_price);
                let (reserved_cash, reserved_qty) = reservation_for_order(
                    updated.side.clone(),
                    remaining_after,
                    reserve_price,
                    updated.fee_bps,
                );
                updated.reserved_cash = reserved_cash;
                updated.reserved_qty = reserved_qty;
                updated.updated_at_ms = now_ms;
                self.open_orders.insert(order_id.clone(), updated.clone());
                events.push(partial_event(
                    &open_order.plan_id,
                    &open_order.order_id,
                    remaining_after,
                    now_ms,
                    trace_id,
                ));
                events.push(open_event(&updated, now_ms, trace_id));
            } else {
                self.open_orders.remove(&order_id);
                events.push(RuntimeEvent {
                    event_id: format!("evt-order-filled-{}-{now_ms}", open_order.order_id),
                    event_type: RuntimeEventType::ExecutionFilled, // v1.1.10: 修正事件类型
                    trace_id: trace_id.to_string(),
                    source_id: open_order.plan_id.clone(),
                    ts_ms: now_ms,
                    payload: json!({
                        "status": "Filled",
                        "lifecycle_stage": "completed",
                        "order_id": open_order.order_id,
                        "remaining_qty": 0.0,
                        "limit_price": open_order.limit_price,
                        "reserved_cash": 0.0,
                        "reserved_qty": 0.0,
                        "explanation_summary": "Resting order fully completed on market update.",
                    }),
                });
            }
            events.push(fill_event(&fill, &open_order.order_id, now_ms, trace_id));
            fills.push(fill);
        }

        sync_portfolio_reservations(portfolio, &self.open_orders);
        FillResult {
            plan_id: format!("market-update-{now_ms}"),
            status: if fills.is_empty() {
                ExecutionStatus::Open
            } else {
                ExecutionStatus::Filled
            },
            fills,
            open_orders: self.open_orders.values().cloned().collect(),
            events,
        }
    }

    fn try_open_resting_order(
        &mut self,
        request: RestingOrderOpenRequest<'_>,
        portfolio: &mut PortfolioState,
        events: &mut Vec<RuntimeEvent>,
    ) -> Option<OpenOrder> {
        let RestingOrderOpenRequest {
            plan,
            order,
            remaining_qty,
            now_ms,
            trace_id,
        } = request;

        if !can_rest(order) {
            events.push(cancel_event(
                &plan.plan_id,
                &order.order_id,
                "order cannot rest",
                now_ms,
                trace_id,
            ));
            return None;
        }

        sync_portfolio_reservations(portfolio, &self.open_orders);
        let reserve_price = order.limit_price.unwrap_or(order.reference_price);
        let (reserved_cash, reserved_qty) = reservation_for_order(
            order.side.clone(),
            remaining_qty,
            reserve_price,
            order.fee_bps,
        );
        if matches!(order.side, OrderSide::Buy)
            && portfolio.available_cash_balance + 1e-9 < reserved_cash
        {
            events.push(reject_event(
                &plan.plan_id,
                &order.order_id,
                "insufficient available cash for resting order",
                now_ms,
                trace_id,
            ));
            return None;
        }
        if matches!(order.side, OrderSide::Sell)
            && available_position_qty(portfolio, &order.exchange, &order.symbol) + 1e-9
                < remaining_qty
        {
            events.push(reject_event(
                &plan.plan_id,
                &order.order_id,
                "insufficient available position for resting sell order",
                now_ms,
                trace_id,
            ));
            return None;
        }

        let open_order = OpenOrder {
            order_id: order.order_id.clone(),
            plan_id: plan.plan_id.clone(),
            exchange: order.exchange.clone(),
            symbol: order.symbol.clone(),
            side: order.side.clone(),
            order_type: order.order_type.clone(),
            time_in_force: order.time_in_force.clone(),
            remaining_qty,
            reserved_cash,
            reserved_qty,
            limit_price: order.limit_price,
            reference_price: order.reference_price,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
            trace_id: trace_id.to_string(),
            slippage_bps: order.slippage_bps,
            fee_bps: order.fee_bps,
        };
        self.open_orders
            .insert(open_order.order_id.clone(), open_order.clone());
        sync_portfolio_reservations(portfolio, &self.open_orders);
        events.push(open_event(&open_order, now_ms, trace_id));
        Some(open_order)
    }

    #[cfg(test)]
    pub fn open_order_count(&self) -> usize {
        self.open_orders.len()
    }
}

pub fn apply_fill_to_portfolio(portfolio: &mut PortfolioState, fill: &FillReport) {
    let signed_qty = match fill.side {
        OrderSide::Buy => fill.filled_qty,
        OrderSide::Sell => -fill.filled_qty,
    };
    let cash_delta = signed_qty * fill.filled_price;
    portfolio.cash_balance -= cash_delta;
    portfolio.cash_balance -= fill.fee_paid;
    portfolio.available_cash_balance =
        (portfolio.cash_balance - portfolio.frozen_cash_balance).max(0.0);

    let mut found = false;
    for position in &mut portfolio.positions {
        if position.exchange == fill.exchange && position.symbol == fill.symbol {
            found = true;
            let prior_qty = position.net_qty;
            let next_qty = prior_qty + signed_qty;
            if prior_qty.abs() < f64::EPSILON || prior_qty.signum() == signed_qty.signum() {
                let gross_notional = prior_qty.abs() * position.avg_entry_price
                    + fill.filled_qty * fill.filled_price;
                position.net_qty = next_qty;
                position.avg_entry_price = if next_qty.abs() > f64::EPSILON {
                    gross_notional / next_qty.abs()
                } else {
                    0.0
                };
            } else {
                let closed_qty = prior_qty.abs().min(fill.filled_qty);
                position.realized_pnl += match fill.side {
                    OrderSide::Sell => (fill.filled_price - position.avg_entry_price) * closed_qty,
                    OrderSide::Buy => (position.avg_entry_price - fill.filled_price) * closed_qty,
                };
                position.net_qty = next_qty;
                if position.net_qty.abs() < f64::EPSILON {
                    position.avg_entry_price = 0.0;
                    position.frozen_qty = 0.0;
                }
            }
            // v2.1.0: 成交后更新 mark_price 和未实现盈亏
            position.mark_price = fill.filled_price;
            position.unrealized_pnl =
                position.net_qty * (position.mark_price - position.avg_entry_price);
            break;
        }
    }

    if !found {
        portfolio.positions.push(Position {
            exchange: fill.exchange.clone(),
            symbol: fill.symbol.clone(),
            net_qty: signed_qty,
            frozen_qty: 0.0,
            avg_entry_price: fill.filled_price,
            mark_price: fill.filled_price,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::ExecutionPlan;

    fn sample_order_with_side(
        side: OrderSide,
        order_type: OrderType,
        limit_price: Option<f64>,
    ) -> SimOrder {
        SimOrder {
            order_id: "ord_1".into(),
            exchange: Exchange::Binance,
            symbol: Symbol::BtcUsdt,
            side,
            order_type,
            quantity: 1.0,
            limit_price,
            time_in_force: TimeInForce::Gtc,
            allow_partial: false,
            reference_price: 50_000.0,
            slippage_bps: 5.0,
            fee_bps: 10.0,
            strategy_tag: "test".into(),
        }
    }

    fn sample_order(order_type: OrderType, limit_price: Option<f64>) -> SimOrder {
        sample_order_with_side(OrderSide::Buy, order_type, limit_price)
    }

    fn sample_plan(order: SimOrder) -> ExecutionPlan {
        ExecutionPlan {
            plan_id: "plan_1".into(),
            source_risk_decision_id: "risk_1".into(),
            orders: vec![order],
            created_at_ms: 1,
            trace_id: "trace_1".into(),
        }
    }

    fn sample_portfolio() -> PortfolioState {
        PortfolioState::new(100_000.0, 0)
    }

    fn sample_portfolio_with_long_position(qty: f64, avg_entry_price: f64) -> PortfolioState {
        let mut portfolio = PortfolioState::new(100_000.0, 0);
        portfolio.positions.push(Position {
            exchange: Exchange::Binance,
            symbol: Symbol::BtcUsdt,
            net_qty: qty,
            frozen_qty: 0.0,
            avg_entry_price,
            mark_price: avg_entry_price,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        });
        portfolio
    }

    fn sample_market(price: f64) -> BTreeMap<(Exchange, Symbol), MarketState> {
        BTreeMap::from([(
            (Exchange::Binance, Symbol::BtcUsdt),
            MarketState {
                price,
                bid_price: None,
                ask_price: None,
                buy_liquidity: 10.0,
                sell_liquidity: 10.0,
            },
        )])
    }

    #[test]
    fn market_order_fills_immediately() {
        let mut engine = FillEngine::default();
        let plan = sample_plan(sample_order(OrderType::Market, None));
        let mut portfolio = sample_portfolio();

        let result = engine.submit_plan(
            &plan,
            &sample_market(50_000.0),
            &mut portfolio,
            10,
            "trace_1",
        );

        assert!(matches!(result.status, ExecutionStatus::Filled));
        assert_eq!(result.fills.len(), 1);
        assert!(portfolio.cash_balance < 100_000.0);
    }

    #[test]
    fn order_level_slippage_changes_market_fill_price() {
        let mut low_slippage = FillEngine::default();
        let mut high_slippage = FillEngine::default();
        let mut base_order = sample_order(OrderType::Market, None);
        base_order.fee_bps = 0.0;

        let mut low_order = base_order.clone();
        low_order.slippage_bps = 0.0;
        let mut high_order = base_order;
        high_order.slippage_bps = 100.0;

        let low = low_slippage.submit_plan(
            &sample_plan(low_order),
            &sample_market(50_000.0),
            &mut sample_portfolio(),
            10,
            "trace_low",
        );
        let high = high_slippage.submit_plan(
            &sample_plan(high_order),
            &sample_market(50_000.0),
            &mut sample_portfolio(),
            10,
            "trace_high",
        );

        assert!(high.fills[0].filled_price > low.fills[0].filled_price);
    }

    #[test]
    fn gtc_buy_order_is_capped_by_available_cash() {
        let mut engine = FillEngine::default();
        let mut order = sample_order(OrderType::Market, None);
        order.quantity = 3.0;
        order.allow_partial = true;
        order.fee_bps = 10.0;
        order.slippage_bps = 10.0;
        let plan = sample_plan(order);
        let mut portfolio = PortfolioState::new(50_000.0, 0);

        let result = engine.submit_plan(
            &plan,
            &sample_market(50_000.0),
            &mut portfolio,
            10,
            "trace_1",
        );

        assert_eq!(result.fills.len(), 1);
        assert!(result.fills[0].filled_qty < 1.0);
        assert!(portfolio.cash_balance >= -1e-6);
        assert!(portfolio.available_cash_balance <= 1e-6);
    }

    #[test]
    fn resting_limit_order_stays_open_when_not_marketable() {
        let mut engine = FillEngine::default();
        let plan = sample_plan(sample_order(OrderType::Limit, Some(49_000.0)));
        let mut portfolio = sample_portfolio();

        let result = engine.submit_plan(
            &plan,
            &sample_market(50_000.0),
            &mut portfolio,
            10,
            "trace_1",
        );

        assert!(matches!(result.status, ExecutionStatus::Open));
        assert_eq!(result.open_orders.len(), 1);
        assert_eq!(engine.open_order_count(), 1);
        assert_eq!(portfolio.positions.len(), 0);
        assert!(portfolio.frozen_cash_balance > 0.0);
    }

    #[test]
    fn repeated_plan_id_is_idempotent() {
        let mut engine = FillEngine::default();
        let plan = sample_plan(sample_order(OrderType::Market, None));
        let mut portfolio = sample_portfolio();

        let first = engine.submit_plan(
            &plan,
            &sample_market(50_000.0),
            &mut portfolio,
            10,
            "trace_1",
        );
        let cash_after_first = portfolio.cash_balance;
        let second = engine.submit_plan(
            &plan,
            &sample_market(50_000.0),
            &mut portfolio,
            11,
            "trace_1",
        );

        assert_eq!(first.fills.len(), second.fills.len());
        assert_eq!(cash_after_first, portfolio.cash_balance);
    }

    #[test]
    fn open_order_fills_on_later_market_update() {
        let mut engine = FillEngine::default();
        let plan = sample_plan(sample_order(OrderType::Limit, Some(49_000.0)));
        let mut portfolio = sample_portfolio();

        let submit = engine.submit_plan(
            &plan,
            &sample_market(50_000.0),
            &mut portfolio,
            10,
            "trace_1",
        );
        assert!(matches!(submit.status, ExecutionStatus::Open));
        let update =
            engine.on_market_update(&sample_market(48_500.0), &mut portfolio, 20, "trace_2");

        assert_eq!(update.fills.len(), 1);
        assert_eq!(engine.open_order_count(), 0);
        assert_eq!(portfolio.positions.len(), 1);
        assert_eq!(portfolio.frozen_cash_balance, 0.0);
    }

    #[test]
    fn ioc_partial_fill_does_not_rest() {
        let mut engine = FillEngine::default();
        let mut order = sample_order(OrderType::Market, None);
        order.quantity = 12.0;
        order.time_in_force = TimeInForce::Ioc;
        order.allow_partial = true;
        let plan = sample_plan(order);
        let mut portfolio = PortfolioState::new(1_000_000.0, 0);
        let result = engine.submit_plan(
            &plan,
            &sample_market(50_000.0),
            &mut portfolio,
            10,
            "trace_1",
        );

        assert!(matches!(result.status, ExecutionStatus::Filled));
        assert_eq!(result.fills[0].filled_qty, 10.0);
        assert_eq!(result.open_orders.len(), 0);
    }

    #[test]
    fn fok_rejects_when_liquidity_is_insufficient() {
        let mut engine = FillEngine::default();
        let mut order = sample_order(OrderType::Market, None);
        order.quantity = 12.0;
        order.time_in_force = TimeInForce::Fok;
        let plan = sample_plan(order);
        let mut portfolio = sample_portfolio();
        let result = engine.submit_plan(
            &plan,
            &sample_market(50_000.0),
            &mut portfolio,
            10,
            "trace_1",
        );

        assert!(matches!(result.status, ExecutionStatus::Rejected));
        assert_eq!(result.fills.len(), 0);
        assert_eq!(result.events[0].payload["lifecycle_stage"], "accepted");
        assert_eq!(result.events[1].payload["lifecycle_stage"], "rejected");
        assert_eq!(
            result.events[1].payload["explanation_summary"],
            "Order rejected: FOK 流动性不足."
        );
    }
    #[test]
    fn resting_sell_order_freezes_position_and_releases_on_fill() {
        let mut engine = FillEngine::default();
        let plan = sample_plan(sample_order_with_side(
            OrderSide::Sell,
            OrderType::Limit,
            Some(51_000.0),
        ));
        let mut portfolio = sample_portfolio_with_long_position(2.0, 48_000.0);

        let submit = engine.submit_plan(
            &plan,
            &sample_market(50_000.0),
            &mut portfolio,
            10,
            "trace_1",
        );

        assert!(matches!(submit.status, ExecutionStatus::Open));
        assert_eq!(submit.open_orders[0].reserved_qty, 1.0);
        assert_eq!(portfolio.positions[0].frozen_qty, 1.0);
        assert_eq!(submit.events[1].payload["lifecycle_stage"], "open");

        let update =
            engine.on_market_update(&sample_market(51_000.0), &mut portfolio, 20, "trace_2");
        assert_eq!(update.fills.len(), 1);
        assert_eq!(engine.open_order_count(), 0);
        assert!((portfolio.positions[0].net_qty - 1.0).abs() < 1e-9);
        assert_eq!(portfolio.positions[0].frozen_qty, 0.0);
        assert_eq!(update.events[0].payload["lifecycle_stage"], "completed");
        assert_eq!(update.events[1].payload["lifecycle_stage"], "completed");
    }

    #[test]
    fn resting_sell_order_rejects_when_position_is_unavailable() {
        let mut engine = FillEngine::default();
        let mut first_order =
            sample_order_with_side(OrderSide::Sell, OrderType::Limit, Some(51_000.0));
        first_order.order_id = "ord_1".into();
        let mut second_order =
            sample_order_with_side(OrderSide::Sell, OrderType::Limit, Some(52_000.0));
        second_order.order_id = "ord_2".into();
        let mut portfolio = sample_portfolio_with_long_position(1.0, 48_000.0);

        let first = engine.submit_plan(
            &sample_plan(first_order),
            &sample_market(50_000.0),
            &mut portfolio,
            10,
            "trace_1",
        );
        assert!(matches!(first.status, ExecutionStatus::Open));

        let second_plan = ExecutionPlan {
            plan_id: "plan_2".into(),
            source_risk_decision_id: "risk_1".into(),
            orders: vec![second_order],
            created_at_ms: 2,
            trace_id: "trace_2".into(),
        };
        let second = engine.submit_plan(
            &second_plan,
            &sample_market(50_000.0),
            &mut portfolio,
            11,
            "trace_2",
        );

        assert!(matches!(second.status, ExecutionStatus::Rejected));
        assert_eq!(engine.open_order_count(), 1);
        assert_eq!(portfolio.positions[0].frozen_qty, 1.0);
    }

    #[test]
    fn market_sell_order_rejects_when_position_is_unavailable() {
        let mut engine = FillEngine::default();
        let plan = sample_plan(sample_order_with_side(
            OrderSide::Sell,
            OrderType::Market,
            None,
        ));
        let mut portfolio = sample_portfolio();

        let result = engine.submit_plan(
            &plan,
            &sample_market(50_000.0),
            &mut portfolio,
            10,
            "trace_1",
        );

        assert!(matches!(result.status, ExecutionStatus::Rejected));
        assert!(result.fills.is_empty());
        assert!(portfolio.positions.is_empty());
    }
}
