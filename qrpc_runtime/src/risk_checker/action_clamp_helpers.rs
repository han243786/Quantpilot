use super::*;

pub(super) fn clamp_actions(
    actions: &mut [qrpc_core::ProposedAction],
    risk: &RiskPolicy,
    portfolio: &PortfolioState,
    equity: f64,
    symbol: &Symbol,
    status: &mut DecisionStatus,
    reason_codes: &mut Vec<RiskReasonCode>,
    reason_text: &mut String,
) {
    for action in actions.iter_mut() {
        action.quantity_ratio = action.quantity_ratio.min(risk.max_position_ratio).max(0.0);
    }
    if clamp_sell_actions_to_inventory(actions, portfolio, symbol, equity) {
        *status = DecisionStatus::Clamp;
        push_reason_code(reason_codes, RiskReasonCode::InsufficientInventory);
        *reason_text = "sell actions clamped to available spot inventory".into();
    }

    if let Some(max_symbol_net_exposure_ratio) = risk.max_symbol_net_exposure_ratio {
        if clamp_buy_actions_to_symbol_net_exposure(
            actions,
            portfolio,
            symbol,
            max_symbol_net_exposure_ratio.max(0.0),
            equity,
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedSymbolNetExposure);
        }
    }

    let total_buy_headroom = (risk.max_total_leverage - portfolio.total_leverage)
        .max(0.0)
        .min((portfolio.available_cash_balance / equity).max(0.0));
    if clamp_buy_actions_to_total_headroom(actions, total_buy_headroom) {
        *status = DecisionStatus::Clamp;
        push_reason_code(reason_codes, RiskReasonCode::ExceedTotalLeverage);
        *reason_text = "buy actions clamped to total leverage or cash headroom".into();
    }

    if let Some(max_portfolio_net_exposure_ratio) = risk.max_portfolio_net_exposure_ratio {
        if clamp_buy_actions_to_portfolio_net_exposure(
            actions,
            portfolio,
            max_portfolio_net_exposure_ratio.max(0.0),
            equity,
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedPortfolioNetExposure);
        }
    }

    let exchange_headroom = portfolio
        .exchange_exposures
        .iter()
        .map(|item| {
            (
                item.exchange.clone(),
                (risk.max_exchange_leverage - item.leverage).max(0.0),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if clamp_buy_actions_to_exchange_headroom(actions, &exchange_headroom) {
        *status = DecisionStatus::Clamp;
        push_reason_code(reason_codes, RiskReasonCode::ExceedExchangeLeverage);
        *reason_text = "buy actions clamped to exchange leverage headroom".into();
    }
}

fn clamp_sell_actions_to_inventory(
    actions: &mut [qrpc_core::ProposedAction],
    portfolio: &PortfolioState,
    symbol: &Symbol,
    equity: f64,
) -> bool {
    let mut used_ratio_by_exchange = BTreeMap::<Exchange, f64>::new();
    let mut clamped = false;

    for action in actions
        .iter_mut()
        .filter(|item| matches!(item.side, OrderSide::Sell))
    {
        let available_ratio = available_sell_ratio(
            portfolio,
            &action.exchange,
            symbol,
            action.reference_price,
            equity,
        );
        let consumed = used_ratio_by_exchange
            .get(&action.exchange)
            .copied()
            .unwrap_or_default();
        let remaining = (available_ratio - consumed).max(0.0);
        if action.quantity_ratio > remaining {
            action.quantity_ratio = remaining;
            clamped = true;
        }
        *used_ratio_by_exchange
            .entry(action.exchange.clone())
            .or_default() += action.quantity_ratio;
    }

    clamped
}

fn clamp_buy_actions_to_total_headroom(
    actions: &mut [qrpc_core::ProposedAction],
    total_headroom: f64,
) -> bool {
    let total_buy_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Buy))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    if total_buy_ratio <= total_headroom + 1e-9 {
        return false;
    }

    if !total_headroom.is_finite() || total_headroom <= 0.0 {
        for action in actions
            .iter_mut()
            .filter(|item| matches!(item.side, OrderSide::Buy))
        {
            action.quantity_ratio = 0.0;
        }
        return true;
    }

    let scale = total_headroom / total_buy_ratio;
    for action in actions
        .iter_mut()
        .filter(|item| matches!(item.side, OrderSide::Buy))
    {
        action.quantity_ratio *= scale;
    }
    true
}

fn clamp_buy_actions_to_exchange_headroom(
    actions: &mut [qrpc_core::ProposedAction],
    exchange_headroom: &BTreeMap<Exchange, f64>,
) -> bool {
    let mut total_buy_by_exchange = BTreeMap::<Exchange, f64>::new();
    for action in actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Buy))
    {
        *total_buy_by_exchange
            .entry(action.exchange.clone())
            .or_default() += action.quantity_ratio;
    }

    let mut scale_by_exchange = BTreeMap::<Exchange, f64>::new();
    let mut clamped = false;
    for (exchange, total_ratio) in total_buy_by_exchange {
        let headroom = exchange_headroom
            .get(&exchange)
            .copied()
            .unwrap_or(f64::INFINITY);
        if total_ratio > headroom + 1e-9 {
            let scale = if !headroom.is_finite() || headroom <= 0.0 {
                0.0
            } else {
                headroom / total_ratio
            };
            scale_by_exchange.insert(exchange, scale);
            clamped = true;
        }
    }

    if !clamped {
        return false;
    }
    for action in actions
        .iter_mut()
        .filter(|item| matches!(item.side, OrderSide::Buy))
    {
        if let Some(scale) = scale_by_exchange.get(&action.exchange) {
            action.quantity_ratio *= *scale;
        }
    }
    true
}

fn clamp_buy_actions_to_symbol_net_exposure(
    actions: &mut [qrpc_core::ProposedAction],
    portfolio: &PortfolioState,
    symbol: &Symbol,
    max_symbol_net_exposure_ratio: f64,
    equity: f64,
) -> bool {
    let total_buy_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Buy))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    if !total_buy_ratio.is_finite() || total_buy_ratio <= 0.0 {
        return false;
    }
    let total_sell_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Sell))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    let current_ratio = symbol_net_exposure_ratio(portfolio, symbol, equity);
    let remaining_headroom =
        (max_symbol_net_exposure_ratio - (current_ratio - total_sell_ratio).max(0.0)).max(0.0);

    if total_buy_ratio <= remaining_headroom + 1e-9 {
        return false;
    }

    let scale = if !remaining_headroom.is_finite() || remaining_headroom <= 0.0 {
        0.0
    } else {
        remaining_headroom / total_buy_ratio
    };
    for action in actions
        .iter_mut()
        .filter(|item| matches!(item.side, OrderSide::Buy))
    {
        action.quantity_ratio *= scale;
    }
    true
}

fn clamp_buy_actions_to_portfolio_net_exposure(
    actions: &mut [qrpc_core::ProposedAction],
    portfolio: &PortfolioState,
    max_portfolio_net_exposure_ratio: f64,
    equity: f64,
) -> bool {
    let total_buy_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Buy))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    if !total_buy_ratio.is_finite() || total_buy_ratio <= 0.0 {
        return false;
    }
    let total_sell_ratio = actions
        .iter()
        .filter(|item| matches!(item.side, OrderSide::Sell))
        .map(|item| item.quantity_ratio)
        .sum::<f64>();
    let current_ratio = portfolio_net_exposure_ratio(portfolio, equity);
    let remaining_headroom =
        (max_portfolio_net_exposure_ratio - (current_ratio - total_sell_ratio).max(0.0)).max(0.0);

    if total_buy_ratio <= remaining_headroom + 1e-9 {
        return false;
    }

    let scale = if !remaining_headroom.is_finite() || remaining_headroom <= 0.0 {
        0.0
    } else {
        remaining_headroom / total_buy_ratio
    };
    for action in actions
        .iter_mut()
        .filter(|item| matches!(item.side, OrderSide::Buy))
    {
        action.quantity_ratio *= scale;
    }
    true
}
