use super::*;

impl V4SimulatedExecutionRuntimeState {
    pub(super) fn submit_oco_bracket(
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

    pub(in crate::v4_runtime) fn amend_order(
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

    pub(super) fn reject_order(
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

    pub(in crate::v4_runtime) fn expire_orders(
        &mut self,
        now_ms: u64,
    ) -> V4SimulatedExecutionOutcome {
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

    pub(super) fn trim_history(&mut self) {
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

    pub(super) fn cancel_oco_siblings(
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

    pub(super) fn reject_existing_order(
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

    pub(super) fn accepted_order(
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
}
