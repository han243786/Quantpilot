use super::*;

impl V4SimulatedExecutionRuntimeState {
    pub(in crate::v4_runtime) fn update_market_price(
        &mut self,
        venue_id: &str,
        symbol: &str,
        price: f64,
        ts_ms: u64,
    ) -> V4SimulatedExecutionOutcome {
        self.market_prices
            .insert((venue_id.to_string(), symbol.to_string()), price);
        if let Some(position) = self
            .positions
            .get_mut(&(venue_id.to_string(), symbol.to_string()))
        {
            position.market_price = price;
            position.market_value = position.net_quantity * price;
        }
        self.record_asset_point(ts_ms);
        let mut events = Vec::new();
        events.extend(self.expire_orders(ts_ms).events);
        events.extend(self.check_order_triggers(venue_id, symbol, price, ts_ms));
        events.push((
            EVENT_EXECUTION_PORTFOLIO_CHANGED,
            json!({
                "market_price": {
                    "venue_id": venue_id,
                    "symbol": symbol,
                    "price": price,
                },
                "snapshot": self.snapshot(),
            }),
        ));

        V4SimulatedExecutionOutcome { events }
    }

    fn check_order_triggers(
        &mut self,
        venue_id: &str,
        symbol: &str,
        price: f64,
        ts_ms: u64,
    ) -> Vec<(&'static str, Value)> {
        let mut events = Vec::new();
        for index in 0..self.orders.len() {
            let Some(order) = self.orders.get(index).cloned() else {
                continue;
            };
            if order.status != V4SimulatedOrderStatus::Accepted
                || order.venue_id != venue_id
                || order.symbol != symbol
            {
                continue;
            }

            if order.order_type == V4SimulatedOrderType::Limit {
                let request = self.request_from_order(&order, V4SimulatedOrderType::Limit, price);
                if limit_order_is_marketable(&request, order.side) {
                    events.extend(self.fill_existing_order(
                        index,
                        request,
                        "resting_limit_marketable",
                        ts_ms,
                    ));
                }
                continue;
            }

            if order.order_type == V4SimulatedOrderType::TrailingStop {
                events.extend(self.update_trailing_stop(index, price, ts_ms));
            }
            let Some(order) = self.orders.get(index).cloned() else {
                continue;
            };

            if !is_conditional_order_type(order.order_type) {
                continue;
            }
            let Some(trigger_price) = order.trigger_price else {
                events.extend(self.reject_existing_order(
                    index,
                    "条件单缺少 trigger_price".to_string(),
                    ts_ms,
                ));
                continue;
            };
            if !conditional_order_is_triggered(&order, price, trigger_price) {
                continue;
            }

            let converted_order_type = conditional_order_execution_type(order.order_type);
            let request = self.request_from_order(&order, converted_order_type, price);
            events.push((
                EVENT_EXECUTION_CONDITIONAL_ORDER_TRIGGERED,
                json!({
                    "order_id": order.order_id,
                    "order_type": order.order_type,
                    "converted_order_type": converted_order_type,
                    "trigger_price": trigger_price,
                    "market_price": price,
                }),
            ));

            if converted_order_type == V4SimulatedOrderType::Limit
                && !limit_order_is_marketable(&request, order.side)
            {
                let mut converted = order;
                converted.order_type = V4SimulatedOrderType::Limit;
                converted.reference_price = price;
                converted.ts_ms = ts_ms;
                self.orders[index] = converted.clone();
                events.push((
                    EVENT_EXECUTION_ORDER_ACKNOWLEDGED,
                    json!({
                        "order": converted,
                        "resting_reason": "条件单已触发并转换为限价挂单",
                    }),
                ));
                continue;
            }

            events.extend(self.fill_existing_order(index, request, "conditional_trigger", ts_ms));
        }
        events
    }

    fn update_trailing_stop(
        &mut self,
        index: usize,
        price: f64,
        ts_ms: u64,
    ) -> Vec<(&'static str, Value)> {
        let Some(mut order) = self.orders.get(index).cloned() else {
            return Vec::new();
        };
        if order.status != V4SimulatedOrderStatus::Accepted
            || order.order_type != V4SimulatedOrderType::TrailingStop
        {
            return Vec::new();
        }
        let Some(offset_bps) = order.trailing_offset_bps else {
            return Vec::new();
        };
        let offset = offset_bps / 10_000.0;
        let old_trigger = order.trigger_price;
        match order.side {
            V4SimulatedOrderSide::Sell => {
                let peak = order
                    .trailing_peak_price
                    .unwrap_or(order.reference_price)
                    .max(price);
                order.trailing_peak_price = Some(peak);
                let next_trigger = peak * (1.0 - offset);
                order.trigger_price = Some(
                    order
                        .trigger_price
                        .unwrap_or(next_trigger)
                        .max(next_trigger),
                );
            }
            V4SimulatedOrderSide::Buy => {
                let trough = order
                    .trailing_trough_price
                    .unwrap_or(order.reference_price)
                    .min(price);
                order.trailing_trough_price = Some(trough);
                let next_trigger = trough * (1.0 + offset);
                order.trigger_price = Some(
                    order
                        .trigger_price
                        .unwrap_or(next_trigger)
                        .min(next_trigger),
                );
            }
        }
        order.reference_price = price;
        order.ts_ms = ts_ms;
        if old_trigger == order.trigger_price {
            self.orders[index] = order;
            return Vec::new();
        }
        self.orders[index] = order.clone();
        vec![(
            EVENT_EXECUTION_ORDER_AMENDED,
            json!({
                "order": order,
                "reason": "trailing_stop_adjusted",
                "previous_trigger_price": old_trigger,
            }),
        )]
    }

    fn request_from_order(
        &self,
        order: &V4SimulatedOrder,
        order_type: V4SimulatedOrderType,
        reference_price: f64,
    ) -> V4SimulatedOrderRequest {
        V4SimulatedOrderRequest {
            order_id: Some(order.order_id.clone()),
            client_order_id: order.client_order_id.clone(),
            venue_id: order.venue_id.clone(),
            symbol: order.symbol.clone(),
            action: order.action,
            order_type,
            quantity: order.remaining_quantity.max(0.0),
            reference_price,
            limit_price: order.limit_price,
            trigger_price: order.trigger_price,
            take_profit_price: order.take_profit_price,
            stop_loss_price: order.stop_loss_price,
            trailing_offset_bps: order.trailing_offset_bps,
            expire_at_ms: order.expire_at_ms,
            time_in_force: order.time_in_force,
            post_only: false,
            reduce_only: order.action.is_reducing(),
            close_only: matches!(
                order.action,
                V4SimulatedPositionAction::CloseLong | V4SimulatedPositionAction::CloseShort
            ),
            allow_partial_fill: true,
            fee_bps: order.fee_bps,
            slippage_bps: order.slippage_bps,
            max_fill_quantity: None,
        }
    }
}
