use super::*;

pub(super) fn evaluate_atr(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let period = param_or_default(indicator, "period", 14.0).round() as usize;
    let atr = average_true_range(&series.bars, period)
        .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![series.exchange.clone()],
        symbol_scope: vec![series.symbol.clone()],
        side: SignalSide::Neutral,
        strength: 0.0,
        confidence: 0.88,
        reference_price: series.bars.last().map(|item| item.close),
        derived_metrics: BTreeMap::from([("atr".into(), atr), ("period".into(), period as f64)]),
        reason: format!("ATR{} {:.2}", period, atr),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_bollinger_bands(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let period = param_or_default(indicator, "period", 20.0).round() as usize;
    let multiplier = param_or_default(indicator, "multiplier", 2.0);
    let (middle, upper, lower) = bollinger_bands(&series.bars, period, multiplier)
        .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let close = series.bars.last().map(|b| b.close).unwrap_or(0.0);
    let (side, strength) = if close < lower {
        (
            SignalSide::Long,
            scaled_threshold_strength(lower, close, lower),
        )
    } else if close > upper {
        (
            SignalSide::Short,
            -scaled_threshold_strength(close, upper, close),
        )
    } else {
        (SignalSide::Neutral, 0.0)
    };

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![series.exchange.clone()],
        symbol_scope: vec![series.symbol.clone()],
        side,
        strength,
        confidence: 0.88,
        reference_price: series.bars.last().map(|item| item.close),
        derived_metrics: BTreeMap::from([
            ("upper".into(), upper),
            ("middle".into(), middle),
            ("lower".into(), lower),
            ("close".into(), close),
        ]),
        reason: format!(
            "BB({},{}) close={:.2} within [{:.2},{:.2}]",
            period, multiplier, close, lower, upper
        ),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_obv(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let obv_series =
        on_balance_volume(&series.bars).ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let obv = *obv_series
        .last()
        .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let sma_period = param_or_default(indicator, "sma_period", 20.0).round() as usize;
    let obv_sma = simple_moving_average_series(&obv_series, sma_period)
        .and_then(|v| v.last().copied())
        .unwrap_or(obv);
    let (side, strength) = if obv > obv_sma {
        (
            SignalSide::Long,
            scaled_ratio_strength((obv - obv_sma).abs(), obv_sma.abs().max(1.0)),
        )
    } else if obv < obv_sma {
        (
            SignalSide::Short,
            -scaled_ratio_strength((obv_sma - obv).abs(), obv_sma.abs().max(1.0)),
        )
    } else {
        (SignalSide::Neutral, 0.0)
    };

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![series.exchange.clone()],
        symbol_scope: vec![series.symbol.clone()],
        side,
        strength,
        confidence: 0.82,
        reference_price: series.bars.last().map(|item| item.close),
        derived_metrics: BTreeMap::from([("obv".into(), obv), ("obv_sma".into(), obv_sma)]),
        reason: format!("OBV {:.0} vs SMA {:.0}", obv, obv_sma),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_cmf(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let period = param_or_default(indicator, "period", 20.0).round() as usize;
    let cmf = chaikin_money_flow(&series.bars, period)
        .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let (side, strength) = if cmf > 0.0 {
        (SignalSide::Long, scaled_ratio_strength(cmf.abs(), 1.0))
    } else if cmf < 0.0 {
        (SignalSide::Short, -scaled_ratio_strength(cmf.abs(), 1.0))
    } else {
        (SignalSide::Neutral, 0.0)
    };

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![series.exchange.clone()],
        symbol_scope: vec![series.symbol.clone()],
        side,
        strength,
        confidence: 0.82,
        reference_price: series.bars.last().map(|item| item.close),
        derived_metrics: BTreeMap::from([("cmf".into(), cmf), ("period".into(), period as f64)]),
        reason: format!("CMF({}) {:.4}", period, cmf),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_adx(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let period = param_or_default(indicator, "period", 14.0).round() as usize;
    let (adx, plus_di, minus_di) = average_directional_index(&series.bars, period)
        .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let (side, strength) = if plus_di > minus_di {
        let str = if adx > 25.0 {
            scaled_ratio_strength(adx, 50.0)
        } else {
            0.3
        };
        (SignalSide::Long, str)
    } else if minus_di > plus_di {
        let str = if adx > 25.0 {
            scaled_ratio_strength(adx, 50.0)
        } else {
            0.3
        };
        (SignalSide::Short, -str)
    } else {
        (SignalSide::Neutral, 0.0)
    };

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![series.exchange.clone()],
        symbol_scope: vec![series.symbol.clone()],
        side,
        strength,
        confidence: 0.85,
        reference_price: series.bars.last().map(|item| item.close),
        derived_metrics: BTreeMap::from([
            ("adx".into(), adx),
            ("plus_di".into(), plus_di),
            ("minus_di".into(), minus_di),
        ]),
        reason: format!(
            "ADX({}) {:.2} +DI {:.2} -DI {:.2}",
            period, adx, plus_di, minus_di
        ),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_stochastic(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let k_period = param_or_default(indicator, "k_period", 14.0).round() as usize;
    let d_period = param_or_default(indicator, "d_period", 3.0).round() as usize;
    let (k_pct, d_pct) = stochastic_oscillator(&series.bars, k_period, d_period)
        .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let (side, strength) = if k_pct < 20.0 && k_pct > d_pct {
        (
            SignalSide::Long,
            scaled_threshold_strength(20.0, k_pct, 20.0),
        )
    } else if k_pct > 80.0 && k_pct < d_pct {
        (
            SignalSide::Short,
            -scaled_threshold_strength(k_pct, 80.0, 100.0 - 80.0),
        )
    } else {
        (SignalSide::Neutral, 0.0)
    };

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![series.exchange.clone()],
        symbol_scope: vec![series.symbol.clone()],
        side,
        strength,
        confidence: 0.86,
        reference_price: series.bars.last().map(|item| item.close),
        derived_metrics: BTreeMap::from([("k_pct".into(), k_pct), ("d_pct".into(), d_pct)]),
        reason: format!(
            "Stoch({},{}) %K {:.2} %D {:.2}",
            k_period, d_period, k_pct, d_pct
        ),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_cci(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let period = param_or_default(indicator, "period", 20.0).round() as usize;
    let cci = commodity_channel_index(&series.bars, period)
        .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let (side, strength) = if cci < -100.0 {
        (
            SignalSide::Long,
            scaled_threshold_strength(-100.0, cci, -100.0).min(1.0),
        )
    } else if cci > 100.0 {
        (
            SignalSide::Short,
            -scaled_threshold_strength(cci, 100.0, cci).min(1.0),
        )
    } else {
        (SignalSide::Neutral, 0.0)
    };

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![series.exchange.clone()],
        symbol_scope: vec![series.symbol.clone()],
        side,
        strength,
        confidence: 0.84,
        reference_price: series.bars.last().map(|item| item.close),
        derived_metrics: BTreeMap::from([("cci".into(), cci), ("period".into(), period as f64)]),
        reason: format!("CCI({}) {:.2}", period, cci),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_parabolic_sar(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    const DEFAULT_PSAR_STEP: f64 = 0.02;
    const DEFAULT_PSAR_MAX_STEP: f64 = 0.2;
    let step = param_or_default(indicator, "step", DEFAULT_PSAR_STEP);
    let max_step = param_or_default(indicator, "max_step", DEFAULT_PSAR_MAX_STEP);
    let sar = parabolic_sar(&series.bars, step, max_step)
        .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let close = series.bars.last().map(|b| b.close).unwrap_or(0.0);
    let (side, strength) = if close > sar {
        (
            SignalSide::Long,
            scaled_ratio_strength((close - sar).abs(), close.max(sar)),
        )
    } else if close < sar {
        (
            SignalSide::Short,
            -scaled_ratio_strength((sar - close).abs(), close.max(sar)),
        )
    } else {
        (SignalSide::Neutral, 0.0)
    };

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![series.exchange.clone()],
        symbol_scope: vec![series.symbol.clone()],
        side,
        strength,
        confidence: 0.88,
        reference_price: series.bars.last().map(|item| item.close),
        derived_metrics: BTreeMap::from([("sar".into(), sar), ("close".into(), close)]),
        reason: format!("PSAR close={:.2} SAR={:.2}", close, sar),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_keltner_channel(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let period = param_or_default(indicator, "period", 20.0).round() as usize;
    let multiplier = param_or_default(indicator, "multiplier", 2.0);
    let (upper, middle, lower) = keltner_channel(&series.bars, period, multiplier)
        .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let close = series.bars.last().map(|b| b.close).unwrap_or(0.0);
    let (side, strength) = if close > upper {
        (
            SignalSide::Long,
            scaled_ratio_strength((close - upper).abs(), close.max(upper)),
        )
    } else if close < lower {
        (
            SignalSide::Short,
            -scaled_ratio_strength((lower - close).abs(), close.max(lower)),
        )
    } else {
        (SignalSide::Neutral, 0.0)
    };

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![series.exchange.clone()],
        symbol_scope: vec![series.symbol.clone()],
        side,
        strength,
        confidence: 0.85,
        reference_price: series.bars.last().map(|item| item.close),
        derived_metrics: BTreeMap::from([
            ("upper".into(), upper),
            ("middle".into(), middle),
            ("lower".into(), lower),
            ("close".into(), close),
        ]),
        reason: format!("Keltner({},{}) close={:.2}", period, multiplier, close),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_donchian_channel(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let period = param_or_default(indicator, "period", 20.0).round() as usize;
    let (upper, middle, lower) = donchian_channel(&series.bars, period)
        .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let close = series.bars.last().map(|b| b.close).unwrap_or(0.0);
    let (side, strength) = if close >= upper {
        (
            SignalSide::Long,
            scaled_ratio_strength((close - upper).abs(), close.max(upper)),
        )
    } else if close <= lower {
        (
            SignalSide::Short,
            -scaled_ratio_strength((lower - close).abs(), close.max(lower)),
        )
    } else {
        (SignalSide::Neutral, 0.0)
    };

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![series.exchange.clone()],
        symbol_scope: vec![series.symbol.clone()],
        side,
        strength,
        confidence: 0.85,
        reference_price: series.bars.last().map(|item| item.close),
        derived_metrics: BTreeMap::from([
            ("upper".into(), upper),
            ("middle".into(), middle),
            ("lower".into(), lower),
            ("close".into(), close),
        ]),
        reason: format!("Donchian({}) close={:.2}", period, close),
        ttl_ms: 86_400_000,
    })
}
