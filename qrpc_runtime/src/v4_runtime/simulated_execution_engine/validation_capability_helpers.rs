use super::*;

impl V4SimulatedExecutionRuntimeState {
    pub(super) fn validate_position_action(
        &self,
        request: &V4SimulatedOrderRequest,
    ) -> Result<(), String> {
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

    pub(super) fn pre_execution_rejection_reason(
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

    pub(super) fn non_executable_resting_reason(
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
}

impl V4SimulatedPositionAction {
    pub(super) fn side(self) -> V4SimulatedOrderSide {
        match self {
            V4SimulatedPositionAction::Buy
            | V4SimulatedPositionAction::OpenLong
            | V4SimulatedPositionAction::CloseShort => V4SimulatedOrderSide::Buy,
            V4SimulatedPositionAction::Sell
            | V4SimulatedPositionAction::OpenShort
            | V4SimulatedPositionAction::CloseLong => V4SimulatedOrderSide::Sell,
        }
    }

    pub(super) fn is_reducing(self) -> bool {
        matches!(
            self,
            V4SimulatedPositionAction::CloseLong | V4SimulatedPositionAction::CloseShort
        )
    }
}

pub(in crate::v4_runtime) fn is_conditional_order_type(order_type: V4SimulatedOrderType) -> bool {
    matches!(
        order_type,
        V4SimulatedOrderType::StopMarket
            | V4SimulatedOrderType::StopLimit
            | V4SimulatedOrderType::TakeProfitMarket
            | V4SimulatedOrderType::TakeProfitLimit
            | V4SimulatedOrderType::TrailingStop
    )
}

pub(in crate::v4_runtime) fn conditional_order_execution_type(
    order_type: V4SimulatedOrderType,
) -> V4SimulatedOrderType {
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

pub(in crate::v4_runtime) fn conditional_order_is_triggered(
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

pub(in crate::v4_runtime) fn validate_simulated_execution_config(
    config: &V4SimulatedExecutionConfig,
) -> Result<()> {
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

pub(in crate::v4_runtime) fn validate_simulated_order_request(
    request: &V4SimulatedOrderRequest,
) -> Result<(), String> {
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

pub(in crate::v4_runtime) fn simulated_order_required_capabilities(
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

pub(in crate::v4_runtime) fn compute_simulated_fill_price(
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

pub(in crate::v4_runtime) fn limit_order_is_marketable(
    request: &V4SimulatedOrderRequest,
    side: V4SimulatedOrderSide,
) -> bool {
    let limit = request.limit_price.unwrap_or(request.reference_price);
    match side {
        V4SimulatedOrderSide::Buy => request.reference_price <= limit,
        V4SimulatedOrderSide::Sell => request.reference_price >= limit,
    }
}
