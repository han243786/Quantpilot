use super::*;

pub(super) fn available_sell_ratio(
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

pub(super) fn portfolio_equity(portfolio: &PortfolioState) -> f64 {
    portfolio.cash_balance + portfolio.total_net_notional
}

pub(super) fn symbol_net_exposure_ratio(
    portfolio: &PortfolioState,
    symbol: &Symbol,
    equity: f64,
) -> f64 {
    portfolio
        .positions
        .iter()
        .filter(|position| &position.symbol == symbol)
        .map(|position| position.net_qty * position.mark_price)
        .sum::<f64>()
        .abs()
        / equity
}

pub(super) fn portfolio_net_exposure_ratio(portfolio: &PortfolioState, equity: f64) -> f64 {
    (portfolio.total_net_notional / equity).abs()
}
