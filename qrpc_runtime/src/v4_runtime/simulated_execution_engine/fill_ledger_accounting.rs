use super::*;

impl V4SimulatedExecutionRuntimeState {
    pub(super) fn fill_existing_order(
        &mut self,
        index: usize,
        request: V4SimulatedOrderRequest,
        trigger_reason: &'static str,
        ts_ms: u64,
    ) -> Vec<(&'static str, Value)> {
        if let Err(reason) = validate_simulated_order_request(&request) {
            return self.reject_existing_order(index, reason, ts_ms);
        }
        if let Err(reason) = self.validate_position_action(&request) {
            return self.reject_existing_order(index, reason, ts_ms);
        }
        let side = request.action.side();
        if let Some(reason) = self.pre_execution_rejection_reason(&request, side) {
            return self.reject_existing_order(index, reason, ts_ms);
        }

        let Some(mut order) = self.orders.get(index).cloned() else {
            return Vec::new();
        };
        let fill_quantity = request.quantity;
        if fill_quantity <= 0.0 {
            return self.reject_existing_order(index, "本地模拟流动性为 0".to_string(), ts_ms);
        }

        let fill_price = compute_simulated_fill_price(&request, side);
        let notional = fill_quantity * fill_price;
        let fee = notional * request.fee_bps.max(0.0) / 10_000.0;
        order.filled_quantity += fill_quantity;
        order.remaining_quantity = (order.remaining_quantity - fill_quantity).max(0.0);
        order.fill_price = Some(fill_price);
        order.status = if order.remaining_quantity > 1e-9 {
            V4SimulatedOrderStatus::PartiallyFilled
        } else {
            V4SimulatedOrderStatus::Filled
        };
        order.reference_price = request.reference_price;
        order.ts_ms = ts_ms;

        let fill = V4SimulatedFill {
            fill_id: format!("fill-{}-{}", order.order_id, self.fills.len() + 1),
            order_id: order.order_id.clone(),
            venue_id: order.venue_id.clone(),
            symbol: order.symbol.clone(),
            side,
            action: order.action,
            quantity: fill_quantity,
            price: fill_price,
            notional,
            fee,
            fee_asset: self.config.quote_asset.clone(),
            ts_ms,
        };

        self.apply_fill_to_ledger(&fill);
        self.orders[index] = order.clone();
        self.fills.push(fill.clone());
        self.trim_history();
        self.record_asset_point(ts_ms);

        let mut events = Vec::new();
        if order.status == V4SimulatedOrderStatus::PartiallyFilled {
            events.push((
                EVENT_EXECUTION_ORDER_PARTIALLY_FILLED,
                json!({ "order": order.clone(), "fill": fill.clone(), "trigger_reason": trigger_reason }),
            ));
        } else {
            events.push((
                EVENT_EXECUTION_ORDER_FILLED,
                json!({ "order": order.clone(), "fill": fill.clone(), "trigger_reason": trigger_reason }),
            ));
        }
        events.push((
            EVENT_EXECUTION_FEE_CHARGED,
            json!({ "order_id": fill.order_id, "fee": fill.fee, "fee_asset": fill.fee_asset }),
        ));
        if order.status == V4SimulatedOrderStatus::Filled {
            events.extend(self.cancel_oco_siblings(&order, ts_ms));
        }
        events.push((
            EVENT_EXECUTION_PORTFOLIO_CHANGED,
            json!({ "snapshot": self.snapshot() }),
        ));
        events
    }

    pub(super) fn apply_fill_to_ledger(&mut self, fill: &V4SimulatedFill) {
        match fill.side {
            V4SimulatedOrderSide::Buy => {
                self.cash_balance -= fill.notional + fill.fee;
            }
            V4SimulatedOrderSide::Sell => {
                self.cash_balance += fill.notional - fill.fee;
            }
        }
        self.realized_fees += fill.fee;

        let key = (fill.venue_id.clone(), fill.symbol.clone());
        let position = self
            .positions
            .entry(key)
            .or_insert_with(|| V4SimulatedPosition {
                venue_id: fill.venue_id.clone(),
                symbol: fill.symbol.clone(),
                net_quantity: 0.0,
                average_price: 0.0,
                market_price: fill.price,
                market_value: 0.0,
            });
        let old_qty = position.net_quantity;
        let signed_qty = match fill.action {
            V4SimulatedPositionAction::Buy
            | V4SimulatedPositionAction::OpenLong
            | V4SimulatedPositionAction::CloseShort => fill.quantity,
            V4SimulatedPositionAction::Sell
            | V4SimulatedPositionAction::OpenShort
            | V4SimulatedPositionAction::CloseLong => -fill.quantity,
        };
        let new_qty = old_qty + signed_qty;
        if old_qty.signum() == signed_qty.signum() || old_qty.abs() <= f64::EPSILON {
            let old_notional = old_qty.abs() * position.average_price;
            let added_notional = fill.quantity * fill.price;
            let total_qty = old_qty.abs() + fill.quantity;
            position.average_price = if total_qty > 0.0 {
                (old_notional + added_notional) / total_qty
            } else {
                0.0
            };
        } else if new_qty.abs() <= f64::EPSILON {
            position.average_price = 0.0;
        }
        position.net_quantity = if new_qty.abs() <= 1e-9 { 0.0 } else { new_qty };
        position.market_price = fill.price;
        position.market_value = position.net_quantity * fill.price;
    }
}
