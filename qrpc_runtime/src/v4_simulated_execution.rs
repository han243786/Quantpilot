impl V4SimulatedExecutionRuntimeState {
    pub(super) fn new(config: V4SimulatedExecutionConfig, sequence: u64) -> Self {
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

    pub(super) fn submit_order(
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

