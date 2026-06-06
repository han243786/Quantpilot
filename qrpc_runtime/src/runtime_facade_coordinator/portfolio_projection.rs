use super::RuntimeCoordinator;
use qrpc_core::{
    Exchange, ExchangeExposure, NormalizedMarketData, PortfolioState, RuntimeEvent,
    RuntimeEventType, Symbol,
};
use serde_json::json;
use std::collections::BTreeMap;

impl RuntimeCoordinator {
    pub fn portfolio_update_event(
        &self,
        source_id: &str,
        trace_id: &str,
        now_ms: u64,
    ) -> RuntimeEvent {
        let equity_estimate = portfolio_equity_estimate(&self.state.portfolio);
        let open_orders = self
            .state
            .portfolio
            .open_orders
            .iter()
            .map(|order| {
                json!({
                    "order_id": order.order_id,
                    "side": format!("{:?}", order.side),
                    "remaining_qty": order.remaining_qty,
                    "limit_price": order.limit_price,
                    "reserved_cash": order.reserved_cash,
                    "reserved_qty": order.reserved_qty,
                })
            })
            .collect::<Vec<_>>();
        RuntimeEvent {
            event_id: format!("evt-portfolio-{source_id}-{now_ms}"),
            event_type: RuntimeEventType::PortfolioUpdated,
            trace_id: trace_id.to_string(),
            source_id: source_id.to_string(),
            ts_ms: now_ms,
            payload: json!({
                "cash_balance": self.state.portfolio.cash_balance,
                "available_cash_balance": self.state.portfolio.available_cash_balance,
                "frozen_cash_balance": self.state.portfolio.frozen_cash_balance,
                "total_gross_notional": self.state.portfolio.total_gross_notional,
                "total_net_notional": self.state.portfolio.total_net_notional,
                "total_leverage": self.state.portfolio.total_leverage,
                "equity_estimate": equity_estimate,
                "positions": self.state.portfolio.positions.len(),
                "open_order_count": self.state.portfolio.open_orders.len(),
                "open_orders": open_orders,
            }),
        }
    }

    pub(super) fn refresh_portfolio_state(
        &mut self,
        normalized_data: &[NormalizedMarketData],
        now_ms: u64,
    ) {
        let quotes = quote_price_map(normalized_data);
        let mut exposures: BTreeMap<Exchange, (f64, f64)> = BTreeMap::new();

        for position in &mut self.state.portfolio.positions {
            position.mark_price = quotes
                .get(&(position.exchange.clone(), position.symbol.clone()))
                .copied()
                .unwrap_or(position.mark_price);
            if position.mark_price <= 0.0 {
                position.mark_price = position.avg_entry_price.max(0.01);
            }
            position.unrealized_pnl =
                (position.mark_price - position.avg_entry_price) * position.net_qty;
            let gross = position.net_qty.abs() * position.mark_price;
            let net = position.net_qty * position.mark_price;
            let entry = exposures.entry(position.exchange.clone()).or_default();
            entry.0 += gross;
            entry.1 += net;
        }

        self.state.portfolio.exchange_exposures = exposures
            .into_iter()
            .map(
                |(exchange, (gross_notional, net_notional))| ExchangeExposure {
                    leverage: 0.0,
                    exchange,
                    gross_notional,
                    net_notional,
                },
            )
            .collect();

        self.state.portfolio.total_gross_notional = self
            .state
            .portfolio
            .exchange_exposures
            .iter()
            .map(|item| item.gross_notional)
            .sum();
        self.state.portfolio.total_net_notional = self
            .state
            .portfolio
            .exchange_exposures
            .iter()
            .map(|item| item.net_notional)
            .sum();
        let equity_estimate = portfolio_equity_estimate(&self.state.portfolio);
        let initial_equity = 100_000.0;
        let equity_floor = 1_000.0_f64.max(initial_equity * 0.001);
        self.state.portfolio.total_leverage = if equity_estimate.abs() > f64::EPSILON {
            self.state.portfolio.total_gross_notional / equity_estimate.abs().max(equity_floor)
        } else {
            0.0
        };
        for exposure in &mut self.state.portfolio.exchange_exposures {
            exposure.leverage = if equity_estimate.abs() > f64::EPSILON {
                exposure.gross_notional / equity_estimate.abs().max(equity_floor)
            } else {
                0.0
            };
        }
        self.state.portfolio.updated_at_ms = now_ms;
    }
}

pub(super) fn portfolio_equity_estimate(portfolio: &PortfolioState) -> f64 {
    portfolio.cash_balance + portfolio.total_net_notional
}

fn quote_price_map(normalized_data: &[NormalizedMarketData]) -> BTreeMap<(Exchange, Symbol), f64> {
    normalized_data
        .iter()
        .filter_map(|item| match item {
            NormalizedMarketData::Quote(quote) => Some((
                (quote.exchange.clone(), quote.symbol.clone()),
                quote.mid_price,
            )),
            _ => None,
        })
        .collect()
}
