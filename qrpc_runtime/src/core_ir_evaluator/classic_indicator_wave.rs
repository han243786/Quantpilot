use super::*;

pub(super) fn evaluate_ma_family(
    indicator: &IndicatorNode,
    signal_rule: Option<&SignalRule>,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let fast_period = param_or_default(indicator, "fast_period", 50.0).round() as usize;
    let slow_period = param_or_default(indicator, "slow_period", 150.0).round() as usize;
    let baseline_period =
        param_or_default(indicator, "baseline_period", slow_period as f64).round() as usize;
    let lookback = param_or_default(indicator, "lookback", fast_period as f64).round() as usize;
    let ma_fast = moving_average(
        &series.bars,
        if indicator.params.contains_key("lookback") {
            lookback
        } else {
            fast_period
        },
    )
    .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;

    let intent_variant = indicator
        .params
        .get("intent_variant")
        .and_then(|value| value.as_str())
        .unwrap_or("");

    if intent_variant == "long_term_buy" || indicator.params.contains_key("entry_ratio") {
        let ma_slow = moving_average(&series.bars, slow_period)
            .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
        let entry_ratio = param_or_default(indicator, "entry_ratio", 0.8);
        let threshold = entry_ratio * ma_slow;
        let triggered = threshold.is_finite() && threshold > 0.0 && ma_fast >= threshold;
        let strength = if triggered {
            let raw = (ma_fast / threshold) - 1.0;
            if raw.is_finite() {
                raw.clamp(0.0, 1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };
        Ok(CoreIrIndicatorEvaluation {
            exchange_scope: vec![series.exchange.clone()],
            symbol_scope: vec![series.symbol.clone()],
            side: if triggered {
                SignalSide::Long
            } else {
                SignalSide::Neutral
            },
            strength,
            confidence: 0.9,
            reference_price: series.bars.last().map(|item| item.close),
            derived_metrics: BTreeMap::from([
                ("ma_slow".into(), ma_slow),
                ("ma_fast".into(), ma_fast),
                ("threshold".into(), threshold),
            ]),
            reason: format!(
                "MA{} {:.2} {} {:.2} * MA{} {:.2}",
                fast_period,
                ma_fast,
                if triggered { ">=" } else { "<" },
                entry_ratio,
                slow_period,
                threshold
            ),
            ttl_ms: 86_400_000,
        })
    } else {
        let ma_baseline = moving_average(&series.bars, baseline_period)
            .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
        let threshold_ratio = param_or_default(indicator, "threshold_ratio", 1.4);
        let threshold = threshold_ratio * ma_baseline;
        let triggered = threshold.is_finite() && threshold > 0.0 && ma_fast > threshold;
        let strength = if triggered {
            let raw = (ma_fast / threshold) - 1.0;
            if raw.is_finite() {
                raw.clamp(0.0, 1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };
        let signal_kind = signal_rule.map(|rule| &rule.signal_kind);
        let is_short = matches!(signal_kind, Some(SignalKind::Short));
        Ok(CoreIrIndicatorEvaluation {
            exchange_scope: vec![series.exchange.clone()],
            symbol_scope: vec![series.symbol.clone()],
            side: if triggered {
                if is_short {
                    SignalSide::Short
                } else {
                    SignalSide::Long
                }
            } else {
                SignalSide::Neutral
            },
            strength: if is_short { -strength } else { strength },
            confidence: 0.85,
            reference_price: series.bars.last().map(|item| item.close),
            derived_metrics: BTreeMap::from([
                ("ma_baseline".into(), ma_baseline),
                ("ma_fast".into(), ma_fast),
                ("threshold".into(), threshold),
            ]),
            reason: format!(
                "MA{} {:.2} {} {:.2} * MA{} {:.2}",
                lookback,
                ma_fast,
                if triggered { ">" } else { "<=" },
                threshold_ratio,
                baseline_period,
                threshold
            ),
            ttl_ms: 86_400_000,
        })
    }
}

pub(super) fn evaluate_rsi(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let period = param_or_default(indicator, "period", 14.0).round() as usize;
    let oversold_threshold = param_or_default(indicator, "oversold_threshold", 30.0);
    let overbought_threshold = param_or_default(indicator, "overbought_threshold", 70.0);
    let smoothing_method = param_or_default(indicator, "smoothing_method", 0.0);
    let rsi = relative_strength_index(&series.bars, period, smoothing_method)
        .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let (side, strength) = if rsi <= oversold_threshold {
        (
            SignalSide::Long,
            scaled_threshold_strength(oversold_threshold, rsi, oversold_threshold),
        )
    } else if rsi >= overbought_threshold {
        (
            SignalSide::Short,
            -scaled_threshold_strength(rsi, overbought_threshold, 100.0 - overbought_threshold),
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
            ("rsi".into(), rsi),
            ("smoothing_method".into(), smoothing_method),
            ("oversold_threshold".into(), oversold_threshold),
            ("overbought_threshold".into(), overbought_threshold),
        ]),
        reason: format!(
            "RSI{} mode {:.0} {:.2} within [{:.2}, {:.2}]",
            period, smoothing_method, rsi, oversold_threshold, overbought_threshold
        ),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_macd(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let fast_period = param_or_default(indicator, "fast_period", 12.0).round() as usize;
    let slow_period = param_or_default(indicator, "slow_period", 26.0).round() as usize;
    let signal_period = param_or_default(indicator, "signal_period", 9.0).round() as usize;
    let histogram_threshold = param_or_default(indicator, "histogram_threshold", 0.0);
    let (macd_line, signal_line, histogram) =
        macd_histogram(&series.bars, fast_period, slow_period, signal_period)
            .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let (side, strength) = if histogram > histogram_threshold {
        (
            SignalSide::Long,
            scaled_ratio_strength(
                histogram.abs(),
                series
                    .bars
                    .last()
                    .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?
                    .close,
            ),
        )
    } else if histogram < -histogram_threshold {
        (
            SignalSide::Short,
            -scaled_ratio_strength(
                histogram.abs(),
                series
                    .bars
                    .last()
                    .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?
                    .close,
            ),
        )
    } else {
        (SignalSide::Neutral, 0.0)
    };
    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![series.exchange.clone()],
        symbol_scope: vec![series.symbol.clone()],
        side,
        strength,
        confidence: 0.9,
        reference_price: series.bars.last().map(|item| item.close),
        derived_metrics: BTreeMap::from([
            ("macd".into(), macd_line),
            ("signal".into(), signal_line),
            ("histogram".into(), histogram),
        ]),
        reason: format!(
            "MACD({}, {}, {}) histogram {:.4}",
            fast_period, slow_period, signal_period, histogram
        ),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_momentum(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let lookback = param_or_default(indicator, "lookback", 10.0).round() as usize;
    const DEFAULT_THRESHOLD_RATIO: f64 = 0.02;
    let threshold_ratio = param_or_default(indicator, "threshold_ratio", DEFAULT_THRESHOLD_RATIO);
    let momentum = momentum_ratio(&series.bars, lookback)
        .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let (side, strength) = if momentum >= threshold_ratio {
        (
            SignalSide::Long,
            scaled_ratio_strength(
                (momentum.abs() - threshold_ratio).max(0.0),
                threshold_ratio.max(0.01),
            ),
        )
    } else if momentum <= -threshold_ratio {
        (
            SignalSide::Short,
            -scaled_ratio_strength(
                (momentum.abs() - threshold_ratio).max(0.0),
                threshold_ratio.max(0.01),
            ),
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
        derived_metrics: BTreeMap::from([
            ("momentum".into(), momentum),
            ("threshold_ratio".into(), threshold_ratio),
        ]),
        reason: format!("momentum{} {:.4}", lookback, momentum),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_zscore(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let window = param_or_default(indicator, "window", 20.0).round() as usize;
    let entry_z = param_or_default(indicator, "entry_z", 2.0).max(0.1);
    let z_score =
        z_score(&series.bars, window).ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;
    let (side, strength) = if z_score <= -entry_z {
        (
            SignalSide::Long,
            scaled_ratio_strength((z_score.abs() - entry_z).max(0.0), entry_z),
        )
    } else if z_score >= entry_z {
        (
            SignalSide::Short,
            -scaled_ratio_strength((z_score.abs() - entry_z).max(0.0), entry_z),
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
        derived_metrics: BTreeMap::from([("z_score".into(), z_score), ("entry_z".into(), entry_z)]),
        reason: format!("z_score{} {:.4}", window, z_score),
        ttl_ms: 86_400_000,
    })
}

pub(super) fn evaluate_quote_observe(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let quote = find_quote_snapshot(normalized_data, indicator)?;
    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![quote.exchange.clone()],
        symbol_scope: vec![quote.symbol.clone()],
        side: SignalSide::Neutral,
        strength: 0.0,
        confidence: 0.95,
        reference_price: Some(quote.mid_price),
        derived_metrics: BTreeMap::from([("mid_price".into(), quote.mid_price)]),
        reason: format!("quote observed on {:?}", quote.exchange),
        ttl_ms: 5_000,
    })
}
