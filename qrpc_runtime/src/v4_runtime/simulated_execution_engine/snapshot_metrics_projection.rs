use super::*;

impl V4SimulatedExecutionRuntimeState {
    pub(super) fn record_asset_point(&mut self, ts_ms: u64) {
        let point = self.asset_point(ts_ms);
        self.asset_curve.push(point);
        if self.asset_curve.len() > 256 {
            let overflow = self.asset_curve.len() - 256;
            self.asset_curve.drain(0..overflow);
        }
    }

    fn asset_point(&self, ts_ms: u64) -> V4SimulatedAssetPoint {
        let position_market_value = self
            .positions
            .values()
            .map(|position| position.market_value)
            .sum::<f64>();
        V4SimulatedAssetPoint {
            ts_ms,
            cash_balance: round_money(self.cash_balance),
            position_market_value: round_money(position_market_value),
            portfolio_value: round_money(self.cash_balance + position_market_value),
        }
    }

    pub(in crate::v4_runtime) fn snapshot(&self) -> V4SimulatedExecutionSnapshot {
        let point = self.asset_point(
            self.asset_curve
                .last()
                .map(|item| item.ts_ms)
                .unwrap_or_default(),
        );
        V4SimulatedExecutionSnapshot {
            enabled: true,
            quote_asset: self.config.quote_asset.clone(),
            cash_balance: point.cash_balance,
            realized_fees: round_money(self.realized_fees),
            position_market_value: point.position_market_value,
            portfolio_value: point.portfolio_value,
            order_count: self.orders.len() as u64,
            open_order_count: self
                .orders
                .iter()
                .filter(|order| order.status == V4SimulatedOrderStatus::Accepted)
                .count() as u64,
            rejected_order_count: self.rejected_order_count,
            fill_count: self.fills.len() as u64,
            positions: self.positions.values().cloned().collect(),
            asset_curve: self.asset_curve.clone(),
            last_order: self.orders.last().cloned(),
            last_fill: self.fills.last().cloned(),
        }
    }

    pub(in crate::v4_runtime) fn microstructure_metrics(
        &self,
    ) -> qrpc_core_ir::v4::V4BacktestMicrostructureMetrics {
        let orders = self
            .orders
            .iter()
            .map(|order| MicrostructureOrderSample {
                requested_quantity: order.requested_quantity,
                filled_quantity: order.filled_quantity,
                reference_price: order.reference_price,
                is_open: order.status == V4SimulatedOrderStatus::Accepted,
            })
            .collect::<Vec<_>>();
        let fills = self
            .fills
            .iter()
            .filter_map(|fill| {
                let reference_price = self
                    .orders
                    .iter()
                    .find(|order| order.order_id == fill.order_id)
                    .map(|order| order.reference_price)
                    .unwrap_or(fill.price);
                (reference_price > 0.0).then_some(MicrostructureFillSample {
                    quantity: fill.quantity,
                    price: fill.price,
                    reference_price,
                })
            })
            .collect::<Vec<_>>();
        compute_microstructure_metrics(&orders, &fills)
    }
}

fn round_money(value: f64) -> f64 {
    (value * 100_000_000.0).round() / 100_000_000.0
}
