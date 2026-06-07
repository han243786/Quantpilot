use super::*;

pub(super) fn execution_status_for(
    fills: &[FillReport],
    pending_orders: &[OpenOrder],
) -> ExecutionStatus {
    if !fills.is_empty() && pending_orders.is_empty() {
        ExecutionStatus::Filled
    } else if fills.is_empty() && !pending_orders.is_empty() {
        ExecutionStatus::Open
    } else if !fills.is_empty() && !pending_orders.is_empty() {
        ExecutionStatus::PartiallyFilled
    } else {
        ExecutionStatus::Rejected
    }
}

pub(super) fn market_state_for(
    market_state: &BTreeMap<(Exchange, Symbol), MarketState>,
    exchange: Exchange,
    symbol: Symbol,
    fallback_price: f64,
) -> MarketState {
    market_state
        .get(&(exchange, symbol))
        .cloned()
        .unwrap_or_else(|| MarketState::from_mid_price(fallback_price))
}

pub(super) fn executable_qty(order: &SimOrder, state: &MarketState) -> f64 {
    // Limit orders use ask/bid for marketability and fall back to mid price.
    let market_ref = match order.side {
        OrderSide::Buy => state.ask_price.unwrap_or(state.price),
        OrderSide::Sell => state.bid_price.unwrap_or(state.price),
    };
    if !is_marketable(order, market_ref) {
        return 0.0;
    }
    let liquidity = match order.side {
        OrderSide::Buy => state.buy_liquidity,
        OrderSide::Sell => state.sell_liquidity,
    };
    if !liquidity.is_finite() || liquidity <= 0.0 {
        if matches!(order.order_type, OrderType::Market) {
            return order.quantity;
        }
        return 0.0;
    }
    order.quantity.min(liquidity)
}

pub(super) fn execution_assumptions_for_order(
    order: &SimOrder,
    assumptions: &ExecutionAssumptions,
) -> ExecutionAssumptions {
    let mut scoped = assumptions.clone();
    if order.slippage_bps.is_finite() && order.slippage_bps >= 0.0 {
        scoped.slippage = SlippageModel::FixedBps {
            bps: order.slippage_bps,
        };
    }
    if order.fee_bps.is_finite() && order.fee_bps >= 0.0 {
        scoped.taker_fee_bps = order.fee_bps;
    }
    scoped
}

pub(super) fn sim_order_from_open(order: &OpenOrder, default_fee_bps: f64) -> SimOrder {
    SimOrder {
        order_id: order.order_id.clone(),
        exchange: order.exchange.clone(),
        symbol: order.symbol.clone(),
        side: order.side.clone(),
        order_type: order.order_type.clone(),
        quantity: order.remaining_qty,
        limit_price: order.limit_price,
        time_in_force: order.time_in_force.clone(),
        allow_partial: true,
        reference_price: order.reference_price,
        slippage_bps: order.slippage_bps,
        fee_bps: if order.fee_bps.is_finite() && order.fee_bps >= 0.0 {
            order.fee_bps
        } else {
            default_fee_bps
        },
        strategy_tag: "resting".into(),
    }
}

pub(super) fn is_marketable(order: &SimOrder, market_ref: f64) -> bool {
    match (&order.order_type, &order.side, order.limit_price) {
        (OrderType::Market, _, _) => true,
        (OrderType::Limit, OrderSide::Buy, Some(limit)) => market_ref <= limit,
        (OrderType::Limit, OrderSide::Sell, Some(limit)) => market_ref >= limit,
        (OrderType::StopLoss | OrderType::StopLossLimit, OrderSide::Buy, Some(limit)) => {
            market_ref >= limit
        }
        (OrderType::StopLoss | OrderType::StopLossLimit, OrderSide::Sell, Some(limit)) => {
            market_ref <= limit
        }
        (OrderType::TakeProfit | OrderType::TakeProfitLimit, OrderSide::Buy, Some(limit)) => {
            market_ref <= limit
        }
        (OrderType::TakeProfit | OrderType::TakeProfitLimit, OrderSide::Sell, Some(limit)) => {
            market_ref >= limit
        }
        _ => false,
    }
}

pub(super) fn can_rest(order: &SimOrder) -> bool {
    matches!(
        order.order_type,
        OrderType::Limit | OrderType::StopLossLimit | OrderType::TakeProfitLimit
    ) && matches!(order.time_in_force, TimeInForce::Gtc)
}

pub(super) fn build_fill_report(
    plan: &ExecutionPlan,
    order: &SimOrder,
    fill_qty: f64,
    market: &MarketState,
    assumptions: &ExecutionAssumptions,
    now_ms: u64,
    trace_id: &str,
) -> FillReport {
    let volatility = 0.02;
    let extended = market.to_extended(volatility, 1440);
    let scoped_assumptions = execution_assumptions_for_order(order, assumptions);
    let fill_price = compute_fill_price(order, &extended, &scoped_assumptions, volatility);
    let fee_paid = fill_qty * fill_price * order.fee_bps.max(0.0) / 10_000.0;

    let latency_seed = now_ms.wrapping_add(
        order
            .order_id
            .bytes()
            .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64)),
    );
    let latency_ms = scoped_assumptions
        .latency
        .delay_ms(&order.exchange, latency_seed);
    let filled_at = now_ms.saturating_add(latency_ms);

    FillReport {
        fill_id: format!("fill-{}-{}-{}", plan.plan_id, order.order_id, filled_at),
        plan_id: plan.plan_id.clone(),
        exchange: order.exchange.clone(),
        symbol: order.symbol.clone(),
        side: order.side.clone(),
        filled_qty: fill_qty,
        filled_price: fill_price,
        fee_paid,
        filled_at_ms: filled_at,
        status: if fill_qty + 1e-9 < order.quantity {
            ExecutionStatus::PartiallyFilled
        } else {
            ExecutionStatus::Filled
        },
        trace_id: trace_id.to_string(),
    }
}

pub(super) fn build_fill_report_from_open(
    order: &OpenOrder,
    fill_qty: f64,
    market: &MarketState,
    assumptions: &ExecutionAssumptions,
    now_ms: u64,
    trace_id: &str,
) -> FillReport {
    // Resting fills prefer limit price and fall back to reference price.
    let fill_price = if let Some(limit) = order.limit_price {
        let volatility = 0.02;
        let extended = market.to_extended(volatility, 1440);
        let temp_order = SimOrder {
            order_id: order.order_id.clone(),
            exchange: order.exchange.clone(),
            symbol: order.symbol.clone(),
            side: order.side.clone(),
            order_type: OrderType::Limit,
            quantity: fill_qty,
            limit_price: order.limit_price,
            time_in_force: TimeInForce::Gtc,
            allow_partial: true,
            reference_price: order.reference_price,
            slippage_bps: order.slippage_bps,
            fee_bps: order.fee_bps,
            strategy_tag: "resting".into(),
        };
        let scoped_assumptions = execution_assumptions_for_order(&temp_order, assumptions);
        let computed = compute_fill_price(&temp_order, &extended, &scoped_assumptions, volatility);
        match order.side {
            OrderSide::Buy => computed.min(limit),
            OrderSide::Sell => computed.max(limit),
        }
    } else {
        order.reference_price
    };
    let latency_seed = now_ms.wrapping_add(
        order
            .order_id
            .bytes()
            .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64)),
    );
    let scoped_assumptions = execution_assumptions_for_order(
        &sim_order_from_open(order, assumptions.taker_fee_bps),
        assumptions,
    );
    let latency_ms = scoped_assumptions
        .latency
        .delay_ms(&order.exchange, latency_seed);
    let filled_at = now_ms.saturating_add(latency_ms);

    FillReport {
        fill_id: format!("fill-{}-{}-{}", order.plan_id, order.order_id, filled_at),
        plan_id: order.plan_id.clone(),
        exchange: order.exchange.clone(),
        symbol: order.symbol.clone(),
        side: order.side.clone(),
        filled_qty: fill_qty,
        filled_price: fill_price,
        fee_paid: fill_qty * fill_price * order.fee_bps.max(0.0) / 10_000.0,
        filled_at_ms: filled_at,
        status: if fill_qty + 1e-9 < order.remaining_qty {
            ExecutionStatus::PartiallyFilled
        } else {
            ExecutionStatus::Filled
        },
        trace_id: trace_id.to_string(),
    }
}
