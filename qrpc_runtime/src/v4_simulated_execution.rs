impl V4SimulatedExecutionRuntimeState {
    fn new(config: V4SimulatedExecutionConfig, sequence: u64) -> Self {
        Self {
            cash_balance: config.starting_cash,
            config,
            realized_fees: 0.0,
            order_sequence: sequence,
            rejected_order_count: 0,
            positions: BTreeMap::new(),
            orders: Vec::new(),
            fills: Vec::new(),
            asset_curve: Vec::new(),
            market_prices: BTreeMap::new(),
        }
    }

    fn submit_order(
        &mut self,
        request: V4SimulatedOrderRequest,
        source_event_sequence: u64,
        ts_ms: u64,
    ) -> V4SimulatedExecutionOutcome {
        if let Err(reason) = validate_simulated_order_request(&request) {
            return self.reject_order(request, source_event_sequence, ts_ms, reason);
        }

        if let Err(reason) = self.validate_position_action(&request) {
            return self.reject_order(request, source_event_sequence, ts_ms, reason);
        }

        let side = request.action.side();
        if let Some(reason) = self.pre_execution_rejection_reason(&request, side) {
            return self.reject_order(request, source_event_sequence, ts_ms, reason);
        }
        if request.order_type == V4SimulatedOrderType::OcoBracket {
            return self.submit_oco_bracket(request, source_event_sequence, ts_ms);
        }
        if let Some(reason) = self.non_executable_resting_reason(&request, side) {
            let order = self.accepted_order(&request, source_event_sequence, ts_ms);
            self.orders.push(order.clone());
            self.trim_history();
            return V4SimulatedExecutionOutcome {
                events: vec![
                    (
                        EVENT_EXECUTION_ORDER_ACKNOWLEDGED,
                        json!({ "order": order, "resting_reason": reason }),
                    ),
                    (
                        EVENT_EXECUTION_PORTFOLIO_CHANGED,
                        json!({ "snapshot": self.snapshot() }),
                    ),
                ],
            };
        }

        let requested_quantity = request.quantity;
        let max_fill_quantity = request
            .max_fill_quantity
            .unwrap_or(requested_quantity)
            .max(0.0);
        let fill_quantity = requested_quantity.min(max_fill_quantity);

        if fill_quantity <= 0.0 {
            return self.reject_order(
                request,
                source_event_sequence,
                ts_ms,
                "本地模拟流动性为 0".to_string(),
            );
        }
        if fill_quantity + f64::EPSILON < requested_quantity {
            if matches!(request.time_in_force, Some(V4SimulatedTimeInForce::Fok)) {
                return self.reject_order(
                    request,
                    source_event_sequence,
                    ts_ms,
                    "FOK 订单无法被本地模拟流动性完全成交".to_string(),
                );
            }
            if !request.allow_partial_fill {
                return self.reject_order(
                    request,
                    source_event_sequence,
                    ts_ms,
                    "当前本地模拟订单未启用部分成交".to_string(),
                );
            }
        }

        let mut order = self.accepted_order(&request, source_event_sequence, ts_ms);
        let acknowledged_order = order.clone();
        let fill_price = compute_simulated_fill_price(&request, side);
        let notional = fill_quantity * fill_price;
        let fee = notional * request.fee_bps.max(0.0) / 10_000.0;
        order.filled_quantity = fill_quantity;
        order.remaining_quantity = (requested_quantity - fill_quantity).max(0.0);
        order.fill_price = Some(fill_price);
        order.status = if order.remaining_quantity > 1e-9 {
            V4SimulatedOrderStatus::PartiallyFilled
        } else {
            V4SimulatedOrderStatus::Filled
        };

        let fill = V4SimulatedFill {
            fill_id: format!("fill-{}-{}", order.order_id, self.fills.len() + 1),
            order_id: order.order_id.clone(),
            venue_id: order.venue_id.clone(),
            symbol: order.symbol.clone(),
            side,
            action: request.action,
            quantity: fill_quantity,
            price: fill_price,
            notional,
            fee,
            fee_asset: self.config.quote_asset.clone(),
            ts_ms,
        };

        self.apply_fill_to_ledger(&fill);
        self.orders.push(order.clone());
        self.fills.push(fill.clone());
        self.trim_history();
        self.record_asset_point(ts_ms);

        let mut events = vec![(
            EVENT_EXECUTION_ORDER_ACKNOWLEDGED,
            json!({ "order": acknowledged_order }),
        )];
        if order.status == V4SimulatedOrderStatus::PartiallyFilled {
            events.push((
                EVENT_EXECUTION_ORDER_PARTIALLY_FILLED,
                json!({ "order": order.clone(), "fill": fill.clone() }),
            ));
        } else {
            events.push((
                EVENT_EXECUTION_ORDER_FILLED,
                json!({ "order": order.clone(), "fill": fill.clone() }),
            ));
        }
        events.push((
            EVENT_EXECUTION_FEE_CHARGED,
            json!({ "order_id": fill.order_id, "fee": fill.fee, "fee_asset": fill.fee_asset }),
        ));
        events.push((
            EVENT_EXECUTION_PORTFOLIO_CHANGED,
            json!({ "snapshot": self.snapshot() }),
        ));
        V4SimulatedExecutionOutcome { events }
    }

    fn submit_oco_bracket(
        &mut self,
        request: V4SimulatedOrderRequest,
        source_event_sequence: u64,
        ts_ms: u64,
    ) -> V4SimulatedExecutionOutcome {
        let (Some(take_profit_price), Some(stop_loss_price)) =
            (request.take_profit_price, request.stop_loss_price)
        else {
            return self.reject_order(
                request,
                source_event_sequence,
                ts_ms,
                "OCO bracket requires take_profit_price and stop_loss_price".to_string(),
            );
        };

        let parent = self.accepted_order(&request, source_event_sequence, ts_ms);
        let group_id = parent.order_id.clone();
        let mut take_profit = self.accepted_order(
            &V4SimulatedOrderRequest {
                order_id: Some(format!("{}-take-profit", group_id)),
                order_type: V4SimulatedOrderType::TakeProfitMarket,
                trigger_price: Some(take_profit_price),
                ..request.clone()
            },
            source_event_sequence,
            ts_ms,
        );
        take_profit.parent_order_id = Some(parent.order_id.clone());
        take_profit.oco_group_id = Some(group_id.clone());
        let mut stop_loss = self.accepted_order(
            &V4SimulatedOrderRequest {
                order_id: Some(format!("{}-stop-loss", group_id)),
                order_type: V4SimulatedOrderType::StopMarket,
                trigger_price: Some(stop_loss_price),
                ..request
            },
            source_event_sequence,
            ts_ms,
        );
        stop_loss.parent_order_id = Some(parent.order_id.clone());
        stop_loss.oco_group_id = Some(group_id);

        self.orders.push(parent.clone());
        self.orders.push(take_profit.clone());
        self.orders.push(stop_loss.clone());
        self.trim_history();
        self.record_asset_point(ts_ms);

        V4SimulatedExecutionOutcome {
            events: vec![
                (
                    EVENT_EXECUTION_ORDER_ACKNOWLEDGED,
                    json!({
                        "order": parent,
                        "oco_legs": [take_profit, stop_loss],
                        "resting_reason": "OCO bracket registered as linked take-profit and stop-loss legs",
                    }),
                ),
                (
                    EVENT_EXECUTION_PORTFOLIO_CHANGED,
                    json!({ "snapshot": self.snapshot() }),
                ),
            ],
        }
    }

    fn amend_order(
        &mut self,
        order_id: &str,
        new_reference_price: Option<f64>,
        new_limit_price: Option<f64>,
        new_trigger_price: Option<f64>,
        new_quantity: Option<f64>,
        ts_ms: u64,
    ) -> V4SimulatedExecutionOutcome {
        let Some(index) = self
            .orders
            .iter()
            .position(|order| order.order_id == order_id)
        else {
            let request = V4SimulatedOrderRequest {
                order_id: Some(order_id.to_string()),
                client_order_id: None,
                venue_id: self.config.default_venue_id.clone(),
                symbol: self.config.default_symbol.clone(),
                action: V4SimulatedPositionAction::Buy,
                order_type: V4SimulatedOrderType::Limit,
                quantity: self.config.default_quantity,
                reference_price: self.config.default_price,
                limit_price: None,
                trigger_price: None,
                take_profit_price: None,
                stop_loss_price: None,
                trailing_offset_bps: None,
                expire_at_ms: None,
                time_in_force: None,
                post_only: false,
                reduce_only: false,
                close_only: false,
                allow_partial_fill: true,
                fee_bps: self.config.default_fee_bps,
                slippage_bps: self.config.default_slippage_bps,
                max_fill_quantity: None,
            };
            return self.reject_order(
                request,
                0,
                ts_ms,
                format!("amend target order `{order_id}` not found"),
            );
        };

        let mut order = self.orders[index].clone();
        if order.status != V4SimulatedOrderStatus::Accepted {
            return V4SimulatedExecutionOutcome {
                events: self.reject_existing_order(
                    index,
                    "cancel-replace-amend only supports open accepted orders".to_string(),
                    ts_ms,
                ),
            };
        }
        if let Some(value) = new_reference_price {
            if !value.is_finite() || value <= 0.0 {
                return V4SimulatedExecutionOutcome {
                    events: self.reject_existing_order(
                        index,
                        "new_reference_price must be finite and positive".to_string(),
                        ts_ms,
                    ),
                };
            }
            order.reference_price = value;
        }
        if let Some(value) = new_limit_price {
            if !value.is_finite() || value <= 0.0 {
                return V4SimulatedExecutionOutcome {
                    events: self.reject_existing_order(
                        index,
                        "new_limit_price must be finite and positive".to_string(),
                        ts_ms,
                    ),
                };
            }
            order.limit_price = Some(value);
        }
        if let Some(value) = new_trigger_price {
            if !value.is_finite() || value <= 0.0 {
                return V4SimulatedExecutionOutcome {
                    events: self.reject_existing_order(
                        index,
                        "new_trigger_price must be finite and positive".to_string(),
                        ts_ms,
                    ),
                };
            }
            order.trigger_price = Some(value);
        }
        if let Some(value) = new_quantity {
            if !value.is_finite() || value <= 0.0 || value + f64::EPSILON < order.filled_quantity {
                return V4SimulatedExecutionOutcome {
                    events: self.reject_existing_order(
                        index,
                        "new_quantity must be finite, positive, and not below filled quantity"
                            .to_string(),
                        ts_ms,
                    ),
                };
            }
            order.requested_quantity = value;
            order.remaining_quantity = (value - order.filled_quantity).max(0.0);
        }
        order.amend_revision = order.amend_revision.saturating_add(1);
        order.ts_ms = ts_ms;
        self.orders[index] = order.clone();
        self.record_asset_point(ts_ms);

        V4SimulatedExecutionOutcome {
            events: vec![
                (
                    EVENT_EXECUTION_ORDER_AMENDED,
                    json!({ "order": order, "reason": "cancel_replace_amend" }),
                ),
                (
                    EVENT_EXECUTION_PORTFOLIO_CHANGED,
                    json!({ "snapshot": self.snapshot() }),
                ),
            ],
        }
    }

    fn reject_order(
        &mut self,
        request: V4SimulatedOrderRequest,
        source_event_sequence: u64,
        ts_ms: u64,
        reason: String,
    ) -> V4SimulatedExecutionOutcome {
        let mut order = self.accepted_order(&request, source_event_sequence, ts_ms);
        order.status = V4SimulatedOrderStatus::Rejected;
        order.rejection_reason = Some(reason.clone());
        order.remaining_quantity = order.requested_quantity;
        self.rejected_order_count += 1;
        self.orders.push(order.clone());
        self.trim_history();
        self.record_asset_point(ts_ms);

        V4SimulatedExecutionOutcome {
            events: vec![
                (
                    EVENT_EXECUTION_ORDER_REJECTED,
                    json!({ "order": order, "reason": reason }),
                ),
                (
                    EVENT_EXECUTION_PORTFOLIO_CHANGED,
                    json!({ "snapshot": self.snapshot() }),
                ),
            ],
        }
    }

    fn expire_orders(&mut self, now_ms: u64) -> V4SimulatedExecutionOutcome {
        let mut events = Vec::new();
        for index in 0..self.orders.len() {
            let Some(expire_at_ms) = self.orders[index].expire_at_ms else {
                continue;
            };
            if self.orders[index].status != V4SimulatedOrderStatus::Accepted
                || expire_at_ms > now_ms
            {
                continue;
            }
            let mut order = self.orders[index].clone();
            order.status = V4SimulatedOrderStatus::Expired;
            order.ts_ms = now_ms;
            self.orders[index] = order.clone();
            events.push((
                EVENT_EXECUTION_ORDER_EXPIRED,
                json!({ "order": order, "expire_at_ms": expire_at_ms }),
            ));
        }
        if !events.is_empty() {
            self.record_asset_point(now_ms);
            events.push((
                EVENT_EXECUTION_PORTFOLIO_CHANGED,
                json!({ "snapshot": self.snapshot() }),
            ));
        }
        self.trim_history();
        V4SimulatedExecutionOutcome { events }
    }

    fn trim_history(&mut self) {
        if self.fills.len() > V4_SIMULATED_MAX_FILL_HISTORY {
            let overflow = self.fills.len() - V4_SIMULATED_MAX_FILL_HISTORY;
            self.fills.drain(0..overflow);
        }

        if self.orders.len() > V4_SIMULATED_MAX_ORDER_HISTORY {
            let mut remaining = self.orders.len() - V4_SIMULATED_MAX_ORDER_HISTORY;
            self.orders.retain(|order| {
                if remaining > 0 && order.status != V4SimulatedOrderStatus::Accepted {
                    remaining -= 1;
                    false
                } else {
                    true
                }
            });
        }
    }

    fn update_market_price(
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

    fn fill_existing_order(
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

    fn cancel_oco_siblings(
        &mut self,
        filled_order: &V4SimulatedOrder,
        ts_ms: u64,
    ) -> Vec<(&'static str, Value)> {
        let Some(group_id) = filled_order
            .oco_group_id
            .clone()
            .or_else(|| filled_order.parent_order_id.clone())
        else {
            return Vec::new();
        };
        let mut events = Vec::new();
        for index in 0..self.orders.len() {
            let should_cancel = {
                let order = &self.orders[index];
                order.status == V4SimulatedOrderStatus::Accepted
                    && order.order_id != filled_order.order_id
                    && (order.order_id == group_id
                        || order.oco_group_id.as_deref() == Some(group_id.as_str())
                        || order.parent_order_id.as_deref() == Some(group_id.as_str()))
            };
            if !should_cancel {
                continue;
            }
            let mut order = self.orders[index].clone();
            order.status = V4SimulatedOrderStatus::Canceled;
            order.ts_ms = ts_ms;
            self.orders[index] = order.clone();
            events.push((
                EVENT_EXECUTION_ORDER_CANCELED,
                json!({
                    "order": order,
                    "reason": "oco_sibling_filled",
                    "filled_order_id": filled_order.order_id,
                    "oco_group_id": group_id.clone(),
                }),
            ));
        }
        events
    }

    fn reject_existing_order(
        &mut self,
        index: usize,
        reason: String,
        ts_ms: u64,
    ) -> Vec<(&'static str, Value)> {
        let Some(mut order) = self.orders.get(index).cloned() else {
            return Vec::new();
        };
        order.status = V4SimulatedOrderStatus::Rejected;
        order.rejection_reason = Some(reason.clone());
        order.ts_ms = ts_ms;
        self.orders[index] = order.clone();
        self.rejected_order_count += 1;
        self.record_asset_point(ts_ms);
        vec![
            (
                EVENT_EXECUTION_ORDER_REJECTED,
                json!({ "order": order, "reason": reason }),
            ),
            (
                EVENT_EXECUTION_PORTFOLIO_CHANGED,
                json!({ "snapshot": self.snapshot() }),
            ),
        ]
    }

    fn accepted_order(
        &mut self,
        request: &V4SimulatedOrderRequest,
        source_event_sequence: u64,
        ts_ms: u64,
    ) -> V4SimulatedOrder {
        let order_id = request.order_id.clone().unwrap_or_else(|| {
            self.order_sequence += 1;
            format!("v4-sim-order-{}", self.order_sequence)
        });
        V4SimulatedOrder {
            order_id,
            client_order_id: request.client_order_id.clone(),
            venue_id: request.venue_id.clone(),
            symbol: request.symbol.clone(),
            action: request.action,
            side: request.action.side(),
            order_type: request.order_type,
            time_in_force: request.time_in_force,
            requested_quantity: request.quantity,
            filled_quantity: 0.0,
            remaining_quantity: request.quantity,
            reference_price: request.reference_price,
            limit_price: request.limit_price,
            trigger_price: request.trigger_price,
            take_profit_price: request.take_profit_price,
            stop_loss_price: request.stop_loss_price,
            trailing_offset_bps: request.trailing_offset_bps,
            expire_at_ms: request.expire_at_ms,
            parent_order_id: None,
            oco_group_id: None,
            trailing_peak_price: if request.order_type == V4SimulatedOrderType::TrailingStop {
                Some(request.reference_price)
            } else {
                None
            },
            trailing_trough_price: if request.order_type == V4SimulatedOrderType::TrailingStop {
                Some(request.reference_price)
            } else {
                None
            },
            amend_revision: 0,
            fill_price: None,
            status: V4SimulatedOrderStatus::Accepted,
            rejection_reason: None,
            fee_bps: request.fee_bps,
            slippage_bps: request.slippage_bps,
            ts_ms,
            source_event_sequence,
        }
    }

    fn validate_position_action(&self, request: &V4SimulatedOrderRequest) -> Result<(), String> {
        let current_qty = self
            .positions
            .get(&(request.venue_id.clone(), request.symbol.clone()))
            .map(|position| position.net_quantity)
            .unwrap_or(0.0);

        match request.action {
            V4SimulatedPositionAction::CloseLong => {
                if current_qty <= 0.0 {
                    return Err("close_long 需要已有多头持仓".to_string());
                }
                if request.quantity > current_qty + f64::EPSILON && !request.allow_partial_fill {
                    return Err("close_long 数量超过已有多头持仓，且未启用部分成交".to_string());
                }
            }
            V4SimulatedPositionAction::CloseShort => {
                if current_qty >= 0.0 {
                    return Err("close_short 需要已有空头持仓".to_string());
                }
                if request.quantity > current_qty.abs() + f64::EPSILON
                    && !request.allow_partial_fill
                {
                    return Err("close_short 数量超过已有空头持仓，且未启用部分成交".to_string());
                }
            }
            V4SimulatedPositionAction::Sell => {
                if current_qty <= 0.0 && (request.reduce_only || request.close_only) {
                    return Err("卖出 reduce_only/close_only 需要已有多头持仓".to_string());
                }
            }
            V4SimulatedPositionAction::Buy => {
                if current_qty >= 0.0 && request.close_only {
                    return Err("buy close_only 需要已有空头持仓".to_string());
                }
            }
            V4SimulatedPositionAction::OpenLong | V4SimulatedPositionAction::OpenShort => {}
        }

        if request.reduce_only {
            match request.action {
                V4SimulatedPositionAction::OpenLong | V4SimulatedPositionAction::OpenShort => {
                    return Err("reduce_only 不能打开新持仓".to_string());
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn pre_execution_rejection_reason(
        &self,
        request: &V4SimulatedOrderRequest,
        side: V4SimulatedOrderSide,
    ) -> Option<String> {
        if request.post_only
            && request.order_type == V4SimulatedOrderType::Limit
            && limit_order_is_marketable(request, side)
        {
            return Some("post_only 限价单会在本地模拟盘口中主动吃单".to_string());
        }
        None
    }

    fn non_executable_resting_reason(
        &self,
        request: &V4SimulatedOrderRequest,
        side: V4SimulatedOrderSide,
    ) -> Option<String> {
        match request.order_type {
            V4SimulatedOrderType::Market => None,
            V4SimulatedOrderType::Limit => {
                if limit_order_is_marketable(request, side) {
                    None
                } else {
                    Some("限价单已登记为挂单；当前本地 runtime 路径尚未接入 open-order trigger engine".to_string())
                }
            }
            V4SimulatedOrderType::StopMarket
            | V4SimulatedOrderType::StopLimit
            | V4SimulatedOrderType::TakeProfitMarket
            | V4SimulatedOrderType::TakeProfitLimit
            | V4SimulatedOrderType::OcoBracket
            | V4SimulatedOrderType::TrailingStop => {
                if request.trigger_price.is_some() {
                    Some("条件单已登记；当前本地 runtime 路径尚未接入 trigger engine".to_string())
                } else {
                    Some("条件单在本地模拟成交前需要提供 trigger_price".to_string())
                }
            }
        }
    }

    fn apply_fill_to_ledger(&mut self, fill: &V4SimulatedFill) {
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

    fn record_asset_point(&mut self, ts_ms: u64) {
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

    fn snapshot(&self) -> V4SimulatedExecutionSnapshot {
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

    fn microstructure_metrics(&self) -> qrpc_core_ir::v4::V4BacktestMicrostructureMetrics {
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

fn transition_source_matches(
    expected_source: Option<&str>,
    event: &V4RuntimeEventEnvelope,
) -> bool {
    expected_source
        .map(|source| source == event.source)
        .unwrap_or(true)
}

fn transition_freshness_matches(
    freshness: Option<EventFreshnessRequirement>,
    _event: &V4RuntimeEventEnvelope,
) -> bool {
    matches!(
        freshness,
        None | Some(EventFreshnessRequirement::FreshOnly)
            | Some(EventFreshnessRequirement::FreshOrStale)
            | Some(EventFreshnessRequirement::RecoveringAllowed)
    )
}

fn validate_payload_field_type(
    field: &MachineEventPayloadField,
    value: &Value,
) -> Result<(), String> {
    let type_name = field.type_name.trim().to_ascii_lowercase();
    let ok = match type_name.as_str() {
        "string" | "symbol" | "venue" | "account" | "side" | "position_side" | "order_type"
        | "time_in_force" | "freshness" | "runtime_mode" | "order_permission" => value.is_string(),
        "bool" | "boolean" => value.is_boolean(),
        "u64" | "uint" => value.as_u64().is_some(),
        "i64" | "int" | "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "f64" | "decimal" | "number" | "price" | "quantity" | "notional" | "percent" | "ratio"
        | "fee" | "slippage" | "leverage" => value.as_f64().is_some_and(f64::is_finite),
        "object" | "map" => value.is_object(),
        "array" | "list" => value.is_array(),
        other => return Err(format!("unsupported catalog type `{}`", other)),
    };

    if ok {
        Ok(())
    } else {
        Err(format!(
            "expected `{}`, got {}",
            field.type_name,
            payload_type_label(value)
        ))
    }
}

fn payload_type_label(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(number) if number.is_i64() => "i64",
        Value::Number(number) if number.is_u64() => "u64",
        Value::Number(_) => "f64",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[allow(dead_code)]
fn recovery_policy_allows_async(policy: &MachineRecoveryPolicy) -> bool {
    matches!(policy, MachineRecoveryPolicy::AsyncRecover)
}

impl V4SimulatedPositionAction {
    fn side(self) -> V4SimulatedOrderSide {
        match self {
            V4SimulatedPositionAction::Buy
            | V4SimulatedPositionAction::OpenLong
            | V4SimulatedPositionAction::CloseShort => V4SimulatedOrderSide::Buy,
            V4SimulatedPositionAction::Sell
            | V4SimulatedPositionAction::OpenShort
            | V4SimulatedPositionAction::CloseLong => V4SimulatedOrderSide::Sell,
        }
    }
    fn is_reducing(self) -> bool {
        matches!(
            self,
            V4SimulatedPositionAction::CloseLong | V4SimulatedPositionAction::CloseShort
        )
    }
}

fn is_conditional_order_type(order_type: V4SimulatedOrderType) -> bool {
    matches!(
        order_type,
        V4SimulatedOrderType::StopMarket
            | V4SimulatedOrderType::StopLimit
            | V4SimulatedOrderType::TakeProfitMarket
            | V4SimulatedOrderType::TakeProfitLimit
            | V4SimulatedOrderType::TrailingStop
    )
}

fn conditional_order_execution_type(order_type: V4SimulatedOrderType) -> V4SimulatedOrderType {
    match order_type {
        V4SimulatedOrderType::StopMarket | V4SimulatedOrderType::TakeProfitMarket => {
            V4SimulatedOrderType::Market
        }
        V4SimulatedOrderType::TrailingStop => V4SimulatedOrderType::Market,
        V4SimulatedOrderType::StopLimit | V4SimulatedOrderType::TakeProfitLimit => {
            V4SimulatedOrderType::Limit
        }
        other => other,
    }
}

fn conditional_order_is_triggered(
    order: &V4SimulatedOrder,
    market_price: f64,
    trigger_price: f64,
) -> bool {
    match order.order_type {
        V4SimulatedOrderType::StopMarket
        | V4SimulatedOrderType::StopLimit
        | V4SimulatedOrderType::TrailingStop => match order.side {
            V4SimulatedOrderSide::Buy => market_price >= trigger_price,
            V4SimulatedOrderSide::Sell => market_price <= trigger_price,
        },
        V4SimulatedOrderType::TakeProfitMarket | V4SimulatedOrderType::TakeProfitLimit => {
            match order.side {
                V4SimulatedOrderSide::Buy => market_price <= trigger_price,
                V4SimulatedOrderSide::Sell => market_price >= trigger_price,
            }
        }
        _ => false,
    }
}

fn validate_simulated_execution_config(config: &V4SimulatedExecutionConfig) -> Result<()> {
    if !config.starting_cash.is_finite() {
        return Err(anyhow!("模拟执行 starting_cash 必须是有限数"));
    }
    if config.quote_asset.trim().is_empty() {
        return Err(anyhow!("模拟执行 quote_asset 不能为空"));
    }
    if config.default_venue_id.trim().is_empty() {
        return Err(anyhow!("模拟执行 default_venue_id 不能为空"));
    }
    if config.default_symbol.trim().is_empty() {
        return Err(anyhow!("模拟执行 default_symbol 不能为空"));
    }
    if !config.default_quantity.is_finite() || config.default_quantity <= 0.0 {
        return Err(anyhow!("模拟执行 default_quantity 必须是有限数且大于 0"));
    }
    if !config.default_price.is_finite() || config.default_price <= 0.0 {
        return Err(anyhow!("模拟执行 default_price 必须是有限数且大于 0"));
    }
    if !config.default_fee_bps.is_finite() || config.default_fee_bps < 0.0 {
        return Err(anyhow!("模拟执行 default_fee_bps 必须是有限数且不小于 0"));
    }
    if !config.default_slippage_bps.is_finite() || config.default_slippage_bps < 0.0 {
        return Err(anyhow!(
            "模拟执行 default_slippage_bps 必须是有限数且不小于 0"
        ));
    }
    Ok(())
}

fn validate_simulated_order_request(request: &V4SimulatedOrderRequest) -> Result<(), String> {
    if request.venue_id.trim().is_empty() {
        return Err("本地模拟订单 venue_id 不能为空".to_string());
    }
    if request.symbol.trim().is_empty() {
        return Err("本地模拟订单 symbol 不能为空".to_string());
    }
    if !request.quantity.is_finite() || request.quantity <= 0.0 {
        return Err("quantity 必须是有限数且大于 0".to_string());
    }
    if !request.reference_price.is_finite() || request.reference_price <= 0.0 {
        return Err("reference_price 必须是有限数且大于 0".to_string());
    }
    if request
        .limit_price
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        return Err("提供 limit_price 时必须是有限数且大于 0".to_string());
    }
    if request
        .trigger_price
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        return Err("提供 trigger_price 时必须是有限数且大于 0".to_string());
    }
    if request
        .take_profit_price
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        return Err("take_profit_price must be finite and positive".to_string());
    }
    if request
        .stop_loss_price
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        return Err("stop_loss_price must be finite and positive".to_string());
    }
    if request
        .trailing_offset_bps
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        return Err("trailing_offset_bps must be finite and positive".to_string());
    }
    if matches!(request.time_in_force, Some(V4SimulatedTimeInForce::Gtd))
        && request.expire_at_ms.is_none()
    {
        return Err("GTD order requires expire_at_ms".to_string());
    }
    if !request.fee_bps.is_finite() || request.fee_bps < 0.0 {
        return Err("fee_bps 必须是有限数且不小于 0".to_string());
    }
    if !request.slippage_bps.is_finite() || request.slippage_bps < 0.0 {
        return Err("slippage_bps 必须是有限数且不小于 0".to_string());
    }
    Ok(())
}

fn simulated_order_required_capabilities(
    request: &V4SimulatedOrderRequest,
) -> BTreeSet<ExecutionCapabilityKind> {
    let mut capabilities = BTreeSet::new();
    capabilities.insert(match request.order_type {
        V4SimulatedOrderType::Market => ExecutionCapabilityKind::Market,
        V4SimulatedOrderType::Limit => ExecutionCapabilityKind::Limit,
        V4SimulatedOrderType::StopMarket => ExecutionCapabilityKind::StopMarket,
        V4SimulatedOrderType::StopLimit => ExecutionCapabilityKind::StopLimit,
        V4SimulatedOrderType::TakeProfitMarket => ExecutionCapabilityKind::TakeProfitMarket,
        V4SimulatedOrderType::TakeProfitLimit => ExecutionCapabilityKind::TakeProfitLimit,
        V4SimulatedOrderType::OcoBracket => ExecutionCapabilityKind::OcoBracket,
        V4SimulatedOrderType::TrailingStop => ExecutionCapabilityKind::TrailingStop,
    });
    if let Some(time_in_force) = request.time_in_force {
        capabilities.insert(match time_in_force {
            V4SimulatedTimeInForce::Gtc => ExecutionCapabilityKind::Gtc,
            V4SimulatedTimeInForce::Ioc => ExecutionCapabilityKind::Ioc,
            V4SimulatedTimeInForce::Fok => ExecutionCapabilityKind::Fok,
            V4SimulatedTimeInForce::Day => ExecutionCapabilityKind::Day,
            V4SimulatedTimeInForce::Gtd => ExecutionCapabilityKind::Gtd,
        });
    }
    match request.action {
        V4SimulatedPositionAction::OpenLong => {
            capabilities.insert(ExecutionCapabilityKind::OpenLong);
        }
        V4SimulatedPositionAction::CloseLong => {
            capabilities.insert(ExecutionCapabilityKind::CloseLong);
        }
        V4SimulatedPositionAction::OpenShort => {
            capabilities.insert(ExecutionCapabilityKind::OpenShort);
        }
        V4SimulatedPositionAction::CloseShort => {
            capabilities.insert(ExecutionCapabilityKind::CloseShort);
        }
        V4SimulatedPositionAction::Buy | V4SimulatedPositionAction::Sell => {}
    }
    if request.post_only {
        capabilities.insert(ExecutionCapabilityKind::PostOnly);
    }
    if request.reduce_only {
        capabilities.insert(ExecutionCapabilityKind::ReduceOnly);
    }
    if request.close_only {
        capabilities.insert(ExecutionCapabilityKind::CloseOnly);
    }
    if request.client_order_id.is_some() {
        capabilities.insert(ExecutionCapabilityKind::ClientOrderId);
    }
    capabilities
}

fn compute_simulated_fill_price(
    request: &V4SimulatedOrderRequest,
    side: V4SimulatedOrderSide,
) -> f64 {
    let base_price = match request.order_type {
        V4SimulatedOrderType::Limit | V4SimulatedOrderType::StopLimit => {
            request.limit_price.unwrap_or(request.reference_price)
        }
        _ => request.reference_price,
    };
    let slippage_ratio = request.slippage_bps.max(0.0) / 10_000.0;
    match side {
        V4SimulatedOrderSide::Buy => base_price * (1.0 + slippage_ratio),
        V4SimulatedOrderSide::Sell => base_price * (1.0 - slippage_ratio),
    }
}

fn limit_order_is_marketable(
    request: &V4SimulatedOrderRequest,
    side: V4SimulatedOrderSide,
) -> bool {
    let limit = request.limit_price.unwrap_or(request.reference_price);
    match side {
        V4SimulatedOrderSide::Buy => request.reference_price <= limit,
        V4SimulatedOrderSide::Sell => request.reference_price >= limit,
    }
}

fn payload_string(payload: &Value, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        payload
            .get(*name)
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string)
    })
}

fn payload_f64(payload: &Value, names: &[&str]) -> Option<f64> {
    names.iter().find_map(|name| {
        let value = payload.get(*name)?;
        value
            .as_f64()
            .or_else(|| value.as_str().and_then(|raw| raw.parse::<f64>().ok()))
            .filter(|value| value.is_finite())
    })
}

fn payload_u64(payload: &Value, names: &[&str]) -> Option<u64> {
    names.iter().find_map(|name| {
        let value = payload.get(*name)?;
        value
            .as_u64()
            .or_else(|| value.as_str().and_then(|raw| raw.parse::<u64>().ok()))
    })
}

fn payload_bool(payload: &Value, names: &[&str]) -> Option<bool> {
    names.iter().find_map(|name| {
        let value = payload.get(*name)?;
        value.as_bool().or_else(|| match value.as_str()? {
            "true" | "True" | "TRUE" | "1" => Some(true),
            "false" | "False" | "FALSE" | "0" => Some(false),
            _ => None,
        })
    })
}

fn metadata_string(metadata: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
}

fn parse_position_action(raw: &str) -> Option<V4SimulatedPositionAction> {
    match normalize_token(raw).as_str() {
        "buy" => Some(V4SimulatedPositionAction::Buy),
        "sell" => Some(V4SimulatedPositionAction::Sell),
        "openlong" => Some(V4SimulatedPositionAction::OpenLong),
        "closelong" => Some(V4SimulatedPositionAction::CloseLong),
        "openshort" => Some(V4SimulatedPositionAction::OpenShort),
        "closeshort" => Some(V4SimulatedPositionAction::CloseShort),
        _ => None,
    }
}

fn parse_order_type(raw: &str) -> Option<V4SimulatedOrderType> {
    match normalize_token(raw).as_str() {
        "market" => Some(V4SimulatedOrderType::Market),
        "limit" => Some(V4SimulatedOrderType::Limit),
        "stopmarket" => Some(V4SimulatedOrderType::StopMarket),
        "stoplimit" => Some(V4SimulatedOrderType::StopLimit),
        "takeprofitmarket" => Some(V4SimulatedOrderType::TakeProfitMarket),
        "takeprofitlimit" => Some(V4SimulatedOrderType::TakeProfitLimit),
        "ocobracket" | "oco" => Some(V4SimulatedOrderType::OcoBracket),
        "trailingstop" => Some(V4SimulatedOrderType::TrailingStop),
        _ => None,
    }
}

fn parse_time_in_force(raw: &str) -> Option<V4SimulatedTimeInForce> {
    match normalize_token(raw).as_str() {
        "gtc" => Some(V4SimulatedTimeInForce::Gtc),
        "ioc" => Some(V4SimulatedTimeInForce::Ioc),
        "fok" => Some(V4SimulatedTimeInForce::Fok),
        "day" => Some(V4SimulatedTimeInForce::Day),
        "gtd" => Some(V4SimulatedTimeInForce::Gtd),
        _ => None,
    }
}

fn normalize_token(raw: &str) -> String {
    raw.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn round_money(value: f64) -> f64 {
    (value * 100_000_000.0).round() / 100_000_000.0
}

