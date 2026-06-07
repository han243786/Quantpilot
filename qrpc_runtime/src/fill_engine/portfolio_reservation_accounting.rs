use super::*;

pub(super) fn cash_limited_executable_qty(
    portfolio: &PortfolioState,
    order: &SimOrder,
    state: &MarketState,
    assumptions: &ExecutionAssumptions,
) -> f64 {
    let executable = executable_qty(order, state);
    if !matches!(order.side, OrderSide::Buy) || !executable.is_finite() || executable <= 0.0 {
        return executable;
    }

    let scoped_assumptions = execution_assumptions_for_order(order, assumptions);
    let extended = state.to_extended(0.02, 1440);
    let fill_price = compute_fill_price(order, &extended, &scoped_assumptions, 0.02);
    if !fill_price.is_finite() || fill_price <= 0.0 {
        return 0.0;
    }
    let fee_multiplier = 1.0 + order.fee_bps.max(0.0) / 10_000.0;
    let max_qty = portfolio.available_cash_balance.max(0.0) / (fill_price * fee_multiplier);
    executable.min(max_qty)
}

pub(super) fn reservation_for_order(
    side: OrderSide,
    quantity: f64,
    price: f64,
    fee_bps: f64,
) -> (f64, f64) {
    let fee_bps = fee_bps.max(0.0);
    match side {
        OrderSide::Buy => (quantity * price * (1.0 + fee_bps / 10_000.0), 0.0),
        OrderSide::Sell => (0.0, quantity),
    }
}

pub(super) fn available_position_qty(
    portfolio: &PortfolioState,
    exchange: &Exchange,
    symbol: &Symbol,
) -> f64 {
    portfolio
        .positions
        .iter()
        .find(|position| &position.exchange == exchange && &position.symbol == symbol)
        .map(|position| (position.net_qty.max(0.0) - position.frozen_qty).max(0.0))
        .unwrap_or(0.0)
}

pub(super) fn available_sell_qty_for_order(portfolio: &PortfolioState, order: &SimOrder) -> f64 {
    if matches!(order.side, OrderSide::Buy) {
        return f64::INFINITY;
    }
    available_position_qty(portfolio, &order.exchange, &order.symbol)
}

pub(super) fn sync_portfolio_reservations(
    portfolio: &mut PortfolioState,
    open_orders: &BTreeMap<String, OpenOrder>,
) {
    portfolio.open_orders = open_orders.values().cloned().collect();
    portfolio.frozen_cash_balance = portfolio
        .open_orders
        .iter()
        .map(|order| order.reserved_cash)
        .sum();
    portfolio.available_cash_balance =
        (portfolio.cash_balance - portfolio.frozen_cash_balance).max(0.0);

    for position in &mut portfolio.positions {
        position.frozen_qty = 0.0;
    }
    for order in &portfolio.open_orders {
        if !matches!(order.side, OrderSide::Sell) {
            continue;
        }
        if let Some(position) = portfolio
            .positions
            .iter_mut()
            .find(|position| position.exchange == order.exchange && position.symbol == order.symbol)
        {
            let max_freezable = position.net_qty.max(0.0);
            position.frozen_qty = (position.frozen_qty + order.reserved_qty).min(max_freezable);
        }
    }
}

pub(super) fn release_reservation_for_fill(
    portfolio: &mut PortfolioState,
    open_order: &OpenOrder,
    fill_qty: f64,
) {
    if !open_order.remaining_qty.is_finite() || open_order.remaining_qty <= 0.0 {
        return;
    }
    let ratio = (fill_qty / open_order.remaining_qty).clamp(0.0, 1.0);
    if open_order.reserved_cash.is_finite() && open_order.reserved_cash > 0.0 {
        portfolio.frozen_cash_balance =
            (portfolio.frozen_cash_balance - open_order.reserved_cash * ratio).max(0.0);
        portfolio.available_cash_balance =
            (portfolio.cash_balance - portfolio.frozen_cash_balance).max(0.0);
    }
    if open_order.reserved_qty.is_finite() && open_order.reserved_qty > 0.0 {
        if let Some(position) = portfolio.positions.iter_mut().find(|position| {
            position.exchange == open_order.exchange && position.symbol == open_order.symbol
        }) {
            position.frozen_qty = (position.frozen_qty - open_order.reserved_qty * ratio).max(0.0);
        }
    }
}
