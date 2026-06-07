use super::*;

pub(super) fn clamp_portfolio_target_limits(
    target_decision: &mut qrpc_core::PortfolioTargetDecision,
    risk: &RiskPolicy,
    portfolio: &PortfolioState,
    equity: f64,
    status: &mut DecisionStatus,
    reason_codes: &mut Vec<RiskReasonCode>,
) {
    let single_weight_limit = risk
        .max_single_weight
        .unwrap_or(risk.max_position_ratio)
        .min(risk.max_position_ratio)
        .clamp(0.0, 1.0);
    let total_buy_headroom = (risk.max_total_leverage - portfolio.total_leverage)
        .max(0.0)
        .min((portfolio.available_cash_balance / equity).max(0.0));
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

    refresh_portfolio_target_current_weights(&mut target_decision.target.target_weights, portfolio);
    if let Some(limit) = risk.max_new_positions_per_rebalance {
        if clamp_portfolio_target_new_positions(
            &mut target_decision.target.target_weights,
            limit as usize,
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedNewPositionsLimit);
        }
    }
    if clamp_portfolio_target_single_weight(
        &mut target_decision.target.target_weights,
        single_weight_limit,
    ) {
        *status = DecisionStatus::Clamp;
        push_reason_code(reason_codes, RiskReasonCode::ExceedSingleWeight);
    }
    if let Some(max_concentration_ratio) = risk.max_concentration_ratio {
        if clamp_portfolio_target_single_weight(
            &mut target_decision.target.target_weights,
            max_concentration_ratio.clamp(0.0, 1.0),
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedConcentration);
        }
    }
    if let Some(max_symbol_net_exposure_ratio) = risk.max_symbol_net_exposure_ratio {
        if clamp_portfolio_target_single_weight(
            &mut target_decision.target.target_weights,
            max_symbol_net_exposure_ratio.clamp(0.0, 1.0),
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedSymbolNetExposure);
        }
    }
    if let Some(max_turnover) = risk.max_turnover {
        if clamp_portfolio_target_turnover(
            &mut target_decision.target.target_weights,
            max_turnover.max(0.0),
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedTurnover);
        }
    }
    if let Some(max_portfolio_net_exposure_ratio) = risk.max_portfolio_net_exposure_ratio {
        if clamp_portfolio_target_portfolio_net_exposure(
            &mut target_decision.target.target_weights,
            max_portfolio_net_exposure_ratio.max(0.0),
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::ExceedPortfolioNetExposure);
        }
    }
    if clamp_portfolio_target_total_buy_headroom(
        &mut target_decision.target.target_weights,
        total_buy_headroom,
    ) {
        *status = DecisionStatus::Clamp;
        push_reason_code(reason_codes, RiskReasonCode::ExceedTotalLeverage);
    }
    if clamp_portfolio_target_exchange_headroom(
        &mut target_decision.target.target_weights,
        &exchange_headroom,
    ) {
        *status = DecisionStatus::Clamp;
        push_reason_code(reason_codes, RiskReasonCode::ExceedExchangeLeverage);
    }
    if let Some(min_trade_weight) = risk.min_trade_weight {
        if clamp_portfolio_target_min_trade_weight(
            &mut target_decision.target.target_weights,
            min_trade_weight.max(0.0),
        ) {
            *status = DecisionStatus::Clamp;
            push_reason_code(reason_codes, RiskReasonCode::TradeBelowMinimum);
        }
    }
}

fn refresh_portfolio_target_current_weights(
    target_weights: &mut [qrpc_core::TargetWeight],
    portfolio: &PortfolioState,
) {
    for item in target_weights {
        item.current_weight = available_sell_ratio(
            portfolio,
            &item.exchange,
            &item.symbol,
            item.reference_price,
            portfolio_equity(portfolio).abs().max(1.0),
        );
    }
}

fn clamp_portfolio_target_new_positions(
    target_weights: &mut [qrpc_core::TargetWeight],
    max_new_positions: usize,
) -> bool {
    let mut candidates = target_weights
        .iter()
        .enumerate()
        .filter(|(_, item)| item.current_weight <= 1e-9 && item.target_weight > 1e-9)
        .map(|(index, item)| {
            (
                index,
                item.signal_score.unwrap_or_default(),
                item.target_weight,
                item.symbol.as_str().to_string(),
            )
        })
        .collect::<Vec<_>>();
    if candidates.len() <= max_new_positions {
        return false;
    }
    candidates.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                right
                    .2
                    .partial_cmp(&left.2)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| left.3.cmp(&right.3))
    });
    let kept = candidates
        .into_iter()
        .take(max_new_positions)
        .map(|(index, _, _, _)| index)
        .collect::<BTreeSet<_>>();
    let mut clamped = false;
    for (index, item) in target_weights.iter_mut().enumerate() {
        if item.current_weight <= 1e-9 && item.target_weight > 1e-9 && !kept.contains(&index) {
            item.target_weight = 0.0;
            clamped = true;
        }
    }
    clamped
}

fn clamp_portfolio_target_single_weight(
    target_weights: &mut [qrpc_core::TargetWeight],
    max_single_weight: f64,
) -> bool {
    let mut clamped = false;
    for item in target_weights {
        let bounded = item.target_weight.clamp(0.0, max_single_weight);
        if (bounded - item.target_weight).abs() > 1e-9 {
            item.target_weight = bounded;
            clamped = true;
        }
    }
    clamped
}

fn clamp_portfolio_target_turnover(
    target_weights: &mut [qrpc_core::TargetWeight],
    max_turnover: f64,
) -> bool {
    let total_turnover = target_weights
        .iter()
        .map(|item| (item.target_weight - item.current_weight).abs())
        .sum::<f64>();
    if total_turnover <= max_turnover + 1e-9 {
        return false;
    }
    let scale = if !max_turnover.is_finite() || max_turnover <= 0.0 || total_turnover <= 0.0 {
        0.0
    } else {
        (max_turnover / total_turnover).min(1.0) // v2.3.0: clamp防NaN
    };
    for item in target_weights {
        let delta = item.target_weight - item.current_weight;
        item.target_weight = (item.current_weight + delta * scale).max(0.0);
    }
    true
}

fn clamp_portfolio_target_total_buy_headroom(
    target_weights: &mut [qrpc_core::TargetWeight],
    total_buy_headroom: f64,
) -> bool {
    let total_buy = target_weights
        .iter()
        .map(|item| (item.target_weight - item.current_weight).max(0.0))
        .sum::<f64>();
    if total_buy <= total_buy_headroom + 1e-9 {
        return false;
    }
    let scale = if !total_buy_headroom.is_finite() || total_buy_headroom <= 0.0 {
        0.0
    } else {
        total_buy_headroom / total_buy
    };
    for item in target_weights {
        let buy_delta = (item.target_weight - item.current_weight).max(0.0);
        if buy_delta.is_finite() && buy_delta > 0.0 {
            item.target_weight = item.current_weight + buy_delta * scale;
        }
    }
    true
}

fn clamp_portfolio_target_exchange_headroom(
    target_weights: &mut [qrpc_core::TargetWeight],
    exchange_headroom: &BTreeMap<Exchange, f64>,
) -> bool {
    let mut total_buy_by_exchange = BTreeMap::<Exchange, f64>::new();
    for item in target_weights.iter() {
        *total_buy_by_exchange
            .entry(item.exchange.clone())
            .or_default() += (item.target_weight - item.current_weight).max(0.0);
    }
    let mut scale_by_exchange = BTreeMap::<Exchange, f64>::new();
    let mut clamped = false;
    for (exchange, total_buy) in total_buy_by_exchange {
        let headroom = exchange_headroom
            .get(&exchange)
            .copied()
            .unwrap_or(f64::INFINITY);
        if total_buy > headroom + 1e-9 {
            scale_by_exchange.insert(
                exchange,
                if !headroom.is_finite() || headroom <= 0.0 {
                    0.0
                } else {
                    headroom / total_buy
                },
            );
            clamped = true;
        }
    }
    if !clamped {
        return false;
    }
    for item in target_weights {
        if let Some(scale) = scale_by_exchange.get(&item.exchange) {
            let buy_delta = (item.target_weight - item.current_weight).max(0.0);
            if buy_delta.is_finite() && buy_delta > 0.0 {
                item.target_weight = item.current_weight + buy_delta * *scale;
            }
        }
    }
    true
}

fn clamp_portfolio_target_min_trade_weight(
    target_weights: &mut [qrpc_core::TargetWeight],
    min_trade_weight: f64,
) -> bool {
    let mut clamped = false;
    for item in target_weights {
        let delta = (item.target_weight - item.current_weight).abs();
        if delta < min_trade_weight && delta > 1e-9 {
            item.target_weight = item.current_weight;
            clamped = true;
        }
    }
    clamped
}

fn clamp_portfolio_target_portfolio_net_exposure(
    target_weights: &mut [qrpc_core::TargetWeight],
    max_portfolio_net_exposure_ratio: f64,
) -> bool {
    // v2.4.0 P1-C1: 净敞口不使用 abs(), 分别计算多头和空头后取净值
    // abs() 求和会将空头转为正敞口, 导致含空头仓位的策略被不当限制
    let (long_sum, short_sum): (f64, f64) = target_weights
        .iter()
        .map(|item| item.target_weight)
        .fold((0.0, 0.0), |(long, short), w| {
            if w > 0.0 {
                (long + w, short)
            } else {
                (long, short + w.abs())
            }
        });
    let total_target_ratio = (long_sum - short_sum).abs();
    if total_target_ratio <= max_portfolio_net_exposure_ratio + 1e-9 {
        return false;
    }
    let scale = if !max_portfolio_net_exposure_ratio.is_finite()
        || max_portfolio_net_exposure_ratio <= 0.0
    {
        0.0
    } else {
        max_portfolio_net_exposure_ratio / total_target_ratio
    };
    for item in target_weights {
        item.target_weight *= scale;
    }
    true
}
