use qrpc_core::{
    Exchange, KlineSeriesSnapshot, NormalizedKline, NormalizedMarketData, QuoteSnapshot,
    SignalSide, Symbol,
};
use qrpc_core_ir::{
    AlignDirection, ArithmeticOp, ArithmeticUnaryOp, ComparisonOp, CoreIndicatorKind,
    CustomExprSpec, CustomValueExpr, IndicatorNode, SeriesAggregation, SeriesExpr, SeriesField,
    SignalKind, SignalRule, SpreadSpec, SpreadValueKind,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct CoreIrIndicatorEvaluation {
    pub exchange_scope: Vec<Exchange>,
    pub symbol_scope: Vec<Symbol>,
    pub side: SignalSide,
    pub strength: f64,
    pub confidence: f64,
    pub reference_price: Option<f64>,
    pub derived_metrics: BTreeMap<String, f64>,
    pub reason: String,
    pub ttl_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreIrIndicatorEvaluatorError {
    MissingInputData,
    InsufficientData,
    UnsupportedIndicator,
    InvalidCustomExpression,
}

pub fn evaluate_indicator_signal(
    indicator: &IndicatorNode,
    signal_rule: Option<&SignalRule>,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    match indicator.kind {
        CoreIndicatorKind::MaCross => evaluate_ma_family(indicator, signal_rule, normalized_data),
        CoreIndicatorKind::Rsi => evaluate_rsi(indicator, normalized_data),
        CoreIndicatorKind::Macd => evaluate_macd(indicator, normalized_data),
        CoreIndicatorKind::Momentum => evaluate_momentum(indicator, normalized_data),
        CoreIndicatorKind::Spread => evaluate_spread(indicator, normalized_data),
        CoreIndicatorKind::ZScore => evaluate_zscore(indicator, normalized_data),
        CoreIndicatorKind::Custom => evaluate_custom(indicator, normalized_data),
        CoreIndicatorKind::QuoteObserve => evaluate_quote_observe(indicator, normalized_data),
    }
}

fn evaluate_ma_family(
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
        let triggered = ma_fast >= threshold;
        let strength = if triggered {
            ((ma_fast / threshold) - 1.0).clamp(0.0, 1.0)
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
        let triggered = ma_fast > threshold;
        let strength = if triggered {
            ((ma_fast / threshold) - 1.0).clamp(0.0, 1.0)
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

fn evaluate_rsi(
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

fn evaluate_macd(
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

fn evaluate_momentum(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let series = find_kline_snapshot(normalized_data, indicator)?;
    let lookback = param_or_default(indicator, "lookback", 10.0).round() as usize;
    let threshold_ratio = param_or_default(indicator, "threshold_ratio", 0.02);
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

fn evaluate_zscore(
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

fn evaluate_custom(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let spec = indicator
        .custom_expr
        .as_ref()
        .ok_or(CoreIrIndicatorEvaluatorError::InvalidCustomExpression)?;
    let left_value = evaluate_custom_value_expr(&spec.predicate.left, normalized_data)?;
    let right_value = evaluate_custom_value_expr(&spec.predicate.right, normalized_data)?;
    let triggered = evaluate_custom_predicate(left_value, spec, right_value);
    let scope = collect_custom_scope(spec, normalized_data)?;
    let strength = custom_strength(triggered, spec, normalized_data, left_value, right_value)?;

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: scope.0.iter().cloned().collect::<Vec<_>>(),
        symbol_scope: scope.1.iter().cloned().collect::<Vec<_>>(),
        side: custom_signal_side(spec, triggered),
        strength,
        confidence: spec.confidence.clamp(0.0, 1.0),
        reference_price: custom_reference_price(spec, normalized_data),
        derived_metrics: BTreeMap::from([
            ("left_value".into(), left_value),
            ("right_value".into(), right_value),
            ("triggered".into(), if triggered { 1.0 } else { 0.0 }),
            ("strength".into(), strength.abs()),
        ]),
        reason: format!(
            "custom expr {:.4} {} {:.4}",
            left_value,
            comparison_name(&spec.predicate.op),
            right_value
        ),
        ttl_ms: 60_000,
    })
}

fn evaluate_quote_observe(
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

fn evaluate_custom_predicate(left: f64, spec: &CustomExprSpec, right: f64) -> bool {
    match &spec.predicate.op {
        ComparisonOp::Lt => left < right,
        ComparisonOp::Lte => left <= right,
        ComparisonOp::Gt => left > right,
        ComparisonOp::Gte => left >= right,
        ComparisonOp::Eq => (left - right).abs() <= f64::EPSILON,
    }
}

fn custom_strength(
    triggered: bool,
    spec: &CustomExprSpec,
    normalized_data: &[NormalizedMarketData],
    left_value: f64,
    right_value: f64,
) -> Result<f64, CoreIrIndicatorEvaluatorError> {
    if !triggered {
        return Ok(0.0);
    }

    let magnitude = if let Some(strength) = &spec.strength {
        evaluate_custom_value_expr(strength, normalized_data)?.abs()
    } else {
        scaled_ratio_strength((left_value - right_value).abs(), right_value.abs().max(1.0))
    }
    .clamp(0.0, 1.0);

    Ok(match spec.signal_kind {
        SignalKind::Short => -magnitude,
        _ => magnitude,
    })
}

fn custom_signal_side(spec: &CustomExprSpec, triggered: bool) -> SignalSide {
    if !triggered {
        return SignalSide::Neutral;
    }
    match spec.signal_kind {
        SignalKind::Long => SignalSide::Long,
        SignalKind::Short => SignalSide::Short,
        SignalKind::Observe | SignalKind::Raw => SignalSide::Neutral,
    }
}

fn comparison_name(op: &ComparisonOp) -> &'static str {
    match op {
        ComparisonOp::Lt => "<",
        ComparisonOp::Lte => "<=",
        ComparisonOp::Gt => ">",
        ComparisonOp::Gte => ">=",
        ComparisonOp::Eq => "==",
    }
}

fn evaluate_spread(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    if let Some(spec) = &indicator.spread_spec {
        return evaluate_typed_spread(spec, normalized_data);
    }

    evaluate_legacy_spread(indicator, normalized_data)
}

fn evaluate_legacy_spread(
    indicator: &IndicatorNode,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let quotes = find_quote_snapshots(normalized_data, indicator)?;
    if quotes.len() < 2 {
        return Err(CoreIrIndicatorEvaluatorError::MissingInputData);
    }

    let symbol = quotes[0].symbol.clone();
    if quotes.iter().any(|quote| quote.symbol != symbol) {
        return Err(CoreIrIndicatorEvaluatorError::UnsupportedIndicator);
    }

    let oldest_ts = quotes
        .iter()
        .map(|quote| quote.ts_ms)
        .min()
        .ok_or(CoreIrIndicatorEvaluatorError::MissingInputData)?;
    let newest_ts = quotes
        .iter()
        .map(|quote| quote.ts_ms)
        .max()
        .ok_or(CoreIrIndicatorEvaluatorError::MissingInputData)?;
    let time_skew_ms = newest_ts.saturating_sub(oldest_ts);
    let max_time_diff_ms = param_or_default(indicator, "max_time_diff_ms", 5_000.0)
        .max(0.0)
        .round() as u64;
    if time_skew_ms > max_time_diff_ms {
        return Err(CoreIrIndicatorEvaluatorError::InsufficientData);
    }

    let buy_quote = quotes
        .iter()
        .min_by(|left, right| left.mid_price.total_cmp(&right.mid_price))
        .ok_or(CoreIrIndicatorEvaluatorError::MissingInputData)?;
    let sell_quote = quotes
        .iter()
        .max_by(|left, right| left.mid_price.total_cmp(&right.mid_price))
        .ok_or(CoreIrIndicatorEvaluatorError::MissingInputData)?;
    if buy_quote.mid_price <= f64::EPSILON {
        return Err(CoreIrIndicatorEvaluatorError::InsufficientData);
    }

    let spread_ratio = (sell_quote.mid_price - buy_quote.mid_price) / buy_quote.mid_price;
    let confidence = if max_time_diff_ms == 0 {
        1.0
    } else {
        (1.0 - (time_skew_ms as f64 / max_time_diff_ms as f64)).clamp(0.5, 1.0)
    };

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![buy_quote.exchange.clone(), sell_quote.exchange.clone()],
        symbol_scope: vec![symbol],
        side: SignalSide::Neutral,
        strength: spread_ratio.clamp(0.0, 1.0),
        confidence,
        reference_price: Some(buy_quote.mid_price),
        derived_metrics: BTreeMap::from([
            ("buy_mid_price".into(), buy_quote.mid_price),
            ("sell_mid_price".into(), sell_quote.mid_price),
            ("spread_ratio".into(), spread_ratio),
            ("spread_bps".into(), spread_ratio * 10_000.0),
            ("time_skew_ms".into(), time_skew_ms as f64),
            ("max_time_diff_ms".into(), max_time_diff_ms as f64),
            ("input_count".into(), quotes.len() as f64),
        ]),
        reason: format!(
            "spread observe {:?}->{:?} {:.2}bps within {}ms skew",
            buy_quote.exchange,
            sell_quote.exchange,
            spread_ratio * 10_000.0,
            time_skew_ms
        ),
        ttl_ms: max_time_diff_ms.max(1_000),
    })
}

#[derive(Debug, Clone)]
struct SeriesPoint {
    data_id: String,
    ts_ms: u64,
    value: f64,
    exchange: Exchange,
    symbol: Symbol,
}

fn evaluate_typed_spread(
    spec: &SpreadSpec,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    let left_series = evaluate_series_expr(&spec.left, normalized_data)?;
    let right_series = evaluate_series_expr(&spec.right, normalized_data)?;
    let left_point = left_series
        .last()
        .ok_or(CoreIrIndicatorEvaluatorError::MissingInputData)?;
    let right_point = align_series_point(
        left_point.ts_ms,
        &right_series,
        &spec.align.direction,
        spec.align.tolerance_ms,
    )
    .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData)?;

    if left_point.symbol != right_point.symbol {
        return Err(CoreIrIndicatorEvaluatorError::UnsupportedIndicator);
    }
    if left_point.value.abs() <= f64::EPSILON {
        return Err(CoreIrIndicatorEvaluatorError::InsufficientData);
    }

    let spread_absolute = right_point.value - left_point.value;
    let spread_ratio = spread_absolute / left_point.value.abs();
    let spread_bps = spread_ratio * 10_000.0;
    let output_value = match spec.output {
        SpreadValueKind::Ratio => spread_ratio,
        SpreadValueKind::Bps => spread_bps,
        SpreadValueKind::Absolute => spread_absolute,
    };
    let time_skew_ms = left_point.ts_ms.abs_diff(right_point.ts_ms);
    let confidence = if spec.align.tolerance_ms == 0 {
        1.0
    } else {
        (1.0 - (time_skew_ms as f64 / spec.align.tolerance_ms as f64)).clamp(0.5, 1.0)
    };

    Ok(CoreIrIndicatorEvaluation {
        exchange_scope: vec![left_point.exchange.clone(), right_point.exchange.clone()],
        symbol_scope: vec![left_point.symbol.clone()],
        side: SignalSide::Neutral,
        strength: spread_ratio.abs().clamp(0.0, 1.0),
        confidence,
        reference_price: Some(left_point.value),
        derived_metrics: BTreeMap::from([
            ("left_value".into(), left_point.value),
            ("right_value".into(), right_point.value),
            ("buy_mid_price".into(), left_point.value),
            ("sell_mid_price".into(), right_point.value),
            ("spread_absolute".into(), spread_absolute),
            ("spread_ratio".into(), spread_ratio),
            ("spread_bps".into(), spread_bps),
            ("spread_output_value".into(), output_value),
            ("left_ts_ms".into(), left_point.ts_ms as f64),
            ("right_ts_ms".into(), right_point.ts_ms as f64),
            ("time_skew_ms".into(), time_skew_ms as f64),
            ("max_time_diff_ms".into(), spec.align.tolerance_ms as f64),
        ]),
        reason: format!(
            "typed spread {}:{:?} -> {}:{:?} output {:?} {:.4} skew {}ms",
            left_point.data_id,
            left_point.exchange,
            right_point.data_id,
            right_point.exchange,
            spec.output,
            output_value,
            time_skew_ms
        ),
        ttl_ms: spec.align.tolerance_ms.max(1_000),
    })
}

fn evaluate_series_expr(
    expr: &SeriesExpr,
    normalized_data: &[NormalizedMarketData],
) -> Result<Vec<SeriesPoint>, CoreIrIndicatorEvaluatorError> {
    match expr {
        SeriesExpr::DataRef { data_id } => {
            materialize_data_field(data_id, &SeriesField::MidOrClose, normalized_data)
        }
        SeriesExpr::DataField { data_id, field } => {
            materialize_data_field(data_id, field, normalized_data)
        }
        SeriesExpr::Resample {
            input,
            period_ms,
            agg,
        } => {
            let points = evaluate_series_expr(input, normalized_data)?;
            Ok(resample_series_points(&points, *period_ms, agg))
        }
        SeriesExpr::WindowAgg {
            input,
            window_size,
            agg,
        } => {
            let points = evaluate_series_expr(input, normalized_data)?;
            Ok(window_aggregate_series_points(&points, *window_size, agg))
        }
        SeriesExpr::IndicatorRef { .. } | SeriesExpr::RawText { .. } => {
            Err(CoreIrIndicatorEvaluatorError::UnsupportedIndicator)
        }
    }
}

fn materialize_data_field(
    data_id: &str,
    field: &SeriesField,
    normalized_data: &[NormalizedMarketData],
) -> Result<Vec<SeriesPoint>, CoreIrIndicatorEvaluatorError> {
    let item = normalized_data
        .iter()
        .find(|item| match item {
            NormalizedMarketData::KlineSeries(series) => series.data_id == data_id,
            NormalizedMarketData::Quote(quote) => quote.data_id == data_id,
        })
        .ok_or(CoreIrIndicatorEvaluatorError::MissingInputData)?;

    match item {
        NormalizedMarketData::Quote(quote) => Ok(vec![SeriesPoint {
            data_id: quote.data_id.clone(),
            ts_ms: quote.ts_ms,
            value: quote_field_value(quote, field)?,
            exchange: quote.exchange.clone(),
            symbol: quote.symbol.clone(),
        }]),
        NormalizedMarketData::KlineSeries(series) => Ok(series
            .bars
            .iter()
            .map(|bar| {
                Ok(SeriesPoint {
                    data_id: series.data_id.clone(),
                    ts_ms: bar.close_time_ms,
                    value: kline_field_value(bar, field)?,
                    exchange: series.exchange.clone(),
                    symbol: series.symbol.clone(),
                })
            })
            .collect::<Result<Vec<_>, CoreIrIndicatorEvaluatorError>>()?),
    }
}

fn quote_field_value(
    quote: &QuoteSnapshot,
    field: &SeriesField,
) -> Result<f64, CoreIrIndicatorEvaluatorError> {
    match field {
        SeriesField::MidOrClose | SeriesField::Close => Ok(quote.mid_price),
        SeriesField::BidOrClose => Ok(quote.best_bid),
        SeriesField::AskOrClose => Ok(quote.best_ask),
        _ => Err(CoreIrIndicatorEvaluatorError::UnsupportedIndicator),
    }
}

fn kline_field_value(
    bar: &NormalizedKline,
    field: &SeriesField,
) -> Result<f64, CoreIrIndicatorEvaluatorError> {
    match field {
        SeriesField::MidOrClose
        | SeriesField::BidOrClose
        | SeriesField::AskOrClose
        | SeriesField::Close => Ok(bar.close),
        SeriesField::Open => Ok(bar.open),
        SeriesField::High => Ok(bar.high),
        SeriesField::Low => Ok(bar.low),
        SeriesField::Volume => Ok(bar.volume),
    }
}

fn resample_series_points(
    points: &[SeriesPoint],
    period_ms: u64,
    agg: &SeriesAggregation,
) -> Vec<SeriesPoint> {
    if period_ms == 0 || points.len() <= 1 {
        return points.to_vec();
    }

    let mut buckets: Vec<Vec<SeriesPoint>> = Vec::new();
    let mut current_bucket_key = None;
    for point in points {
        let bucket_key = point.ts_ms / period_ms;
        if current_bucket_key != Some(bucket_key) {
            buckets.push(Vec::new());
            current_bucket_key = Some(bucket_key);
        }
        if let Some(bucket) = buckets.last_mut() {
            bucket.push(point.clone());
        }
    }

    buckets
        .into_iter()
        .filter_map(|bucket| aggregate_bucket(bucket, agg, period_ms))
        .collect()
}

fn aggregate_bucket(
    bucket: Vec<SeriesPoint>,
    agg: &SeriesAggregation,
    period_ms: u64,
) -> Option<SeriesPoint> {
    let last = bucket.last()?.clone();
    let values = bucket.iter().map(|point| point.value).collect::<Vec<_>>();
    let ts_ms = if period_ms == 0 {
        last.ts_ms
    } else {
        ((last.ts_ms / period_ms) + 1) * period_ms
    };
    Some(SeriesPoint {
        ts_ms,
        value: aggregate_values(&values, agg)?,
        ..last
    })
}

fn window_aggregate_series_points(
    points: &[SeriesPoint],
    window_size: usize,
    agg: &SeriesAggregation,
) -> Vec<SeriesPoint> {
    if window_size <= 1 {
        return points.to_vec();
    }
    if points.len() < window_size {
        return Vec::new();
    }

    let mut output = Vec::new();
    for idx in (window_size - 1)..points.len() {
        let slice = &points[idx + 1 - window_size..=idx];
        let values = slice.iter().map(|point| point.value).collect::<Vec<_>>();
        let mut point = points[idx].clone();
        if let Some(value) = aggregate_values(&values, agg) {
            point.value = value;
            output.push(point);
        }
    }
    output
}

fn aggregate_values(values: &[f64], agg: &SeriesAggregation) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    match agg {
        SeriesAggregation::Last => values.last().copied(),
        SeriesAggregation::Mean => Some(values.iter().sum::<f64>() / values.len() as f64),
        SeriesAggregation::Sum => Some(values.iter().sum::<f64>()),
        SeriesAggregation::Min => values.iter().copied().reduce(f64::min),
        SeriesAggregation::Max => values.iter().copied().reduce(f64::max),
        SeriesAggregation::StdDev => {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance = values
                .iter()
                .map(|value| {
                    let delta = value - mean;
                    delta * delta
                })
                .sum::<f64>()
                / values.len() as f64;
            Some(variance.sqrt())
        }
    }
}

fn align_series_point<'a>(
    anchor_ts_ms: u64,
    points: &'a [SeriesPoint],
    direction: &AlignDirection,
    tolerance_ms: u64,
) -> Option<&'a SeriesPoint> {
    let within_tolerance =
        |candidate_ts_ms: u64| anchor_ts_ms.abs_diff(candidate_ts_ms) <= tolerance_ms;
    match direction {
        AlignDirection::Backward => points
            .iter()
            .filter(|point| point.ts_ms <= anchor_ts_ms && within_tolerance(point.ts_ms))
            .max_by_key(|point| point.ts_ms),
        AlignDirection::Forward => points
            .iter()
            .filter(|point| point.ts_ms >= anchor_ts_ms && within_tolerance(point.ts_ms))
            .min_by_key(|point| point.ts_ms),
        AlignDirection::Nearest => points
            .iter()
            .filter(|point| within_tolerance(point.ts_ms))
            .min_by_key(|point| anchor_ts_ms.abs_diff(point.ts_ms)),
    }
}

fn evaluate_custom_value_expr(
    expr: &CustomValueExpr,
    normalized_data: &[NormalizedMarketData],
) -> Result<f64, CoreIrIndicatorEvaluatorError> {
    match expr {
        CustomValueExpr::Number { value } => Ok(*value),
        CustomValueExpr::Input { data_id, field } => {
            latest_field_value(normalized_data, data_id, *field)
        }
        CustomValueExpr::WindowAgg {
            data_id,
            field,
            window_size,
            agg,
        } => aggregate_window_field(normalized_data, data_id, *field, *window_size, *agg),
        CustomValueExpr::Binary { left, op, right } => {
            let left = evaluate_custom_value_expr(left, normalized_data)?;
            let right = evaluate_custom_value_expr(right, normalized_data)?;
            match op {
                ArithmeticOp::Add => Ok(left + right),
                ArithmeticOp::Sub => Ok(left - right),
                ArithmeticOp::Mul => Ok(left * right),
                ArithmeticOp::Div => {
                    if right.abs() <= f64::EPSILON {
                        Err(CoreIrIndicatorEvaluatorError::InvalidCustomExpression)
                    } else {
                        Ok(left / right)
                    }
                }
            }
        }
        CustomValueExpr::Unary { op, value } => {
            let value = evaluate_custom_value_expr(value, normalized_data)?;
            Ok(match op {
                ArithmeticUnaryOp::Abs => value.abs(),
                ArithmeticUnaryOp::Negate => -value,
            })
        }
    }
}

fn latest_field_value(
    normalized_data: &[NormalizedMarketData],
    data_id: &str,
    field: SeriesField,
) -> Result<f64, CoreIrIndicatorEvaluatorError> {
    let Some(item) = normalized_data.iter().find(|item| match item {
        NormalizedMarketData::KlineSeries(series) => series.data_id == data_id,
        NormalizedMarketData::Quote(quote) => quote.data_id == data_id,
    }) else {
        return Err(CoreIrIndicatorEvaluatorError::MissingInputData);
    };

    match item {
        NormalizedMarketData::KlineSeries(series) => series
            .bars
            .last()
            .map(|bar| kline_field_value(bar, &field))
            .transpose()?
            .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData),
        NormalizedMarketData::Quote(quote) => quote_field_value(quote, &field),
    }
}

fn aggregate_window_field(
    normalized_data: &[NormalizedMarketData],
    data_id: &str,
    field: SeriesField,
    window_size: usize,
    agg: SeriesAggregation,
) -> Result<f64, CoreIrIndicatorEvaluatorError> {
    let series = normalized_data
        .iter()
        .find_map(|item| match item {
            NormalizedMarketData::KlineSeries(series) if series.data_id == data_id => Some(series),
            _ => None,
        })
        .ok_or(CoreIrIndicatorEvaluatorError::MissingInputData)?;
    if window_size == 0 || series.bars.len() < window_size {
        return Err(CoreIrIndicatorEvaluatorError::InsufficientData);
    }
    let window = &series.bars[series.bars.len() - window_size..];
    let values = window
        .iter()
        .map(|bar| kline_field_value(bar, &field))
        .collect::<Result<Vec<_>, _>>()?;
    aggregate_numeric_values(&values, agg)
}

fn aggregate_numeric_values(
    values: &[f64],
    agg: SeriesAggregation,
) -> Result<f64, CoreIrIndicatorEvaluatorError> {
    if values.is_empty() {
        return Err(CoreIrIndicatorEvaluatorError::InsufficientData);
    }
    match agg {
        SeriesAggregation::Last => values
            .last()
            .copied()
            .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData),
        SeriesAggregation::Mean => Ok(values.iter().sum::<f64>() / values.len() as f64),
        SeriesAggregation::Sum => Ok(values.iter().sum::<f64>()),
        SeriesAggregation::Min => values
            .iter()
            .copied()
            .min_by(|left, right| left.total_cmp(right))
            .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData),
        SeriesAggregation::Max => values
            .iter()
            .copied()
            .max_by(|left, right| left.total_cmp(right))
            .ok_or(CoreIrIndicatorEvaluatorError::InsufficientData),
        SeriesAggregation::StdDev => {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance = values
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / values.len() as f64;
            Ok(variance.sqrt())
        }
    }
}

fn collect_custom_scope(
    spec: &CustomExprSpec,
    normalized_data: &[NormalizedMarketData],
) -> Result<(BTreeSet<Exchange>, BTreeSet<Symbol>), CoreIrIndicatorEvaluatorError> {
    let mut data_ids = BTreeSet::new();
    collect_custom_data_ids_from_expr(&spec.predicate.left, &mut data_ids);
    collect_custom_data_ids_from_expr(&spec.predicate.right, &mut data_ids);
    if let Some(strength) = &spec.strength {
        collect_custom_data_ids_from_expr(strength, &mut data_ids);
    }
    if data_ids.is_empty() {
        return Err(CoreIrIndicatorEvaluatorError::InvalidCustomExpression);
    }

    let mut exchanges = BTreeSet::new();
    let mut symbols = BTreeSet::new();
    for data_id in data_ids {
        let item = normalized_data
            .iter()
            .find(|item| match item {
                NormalizedMarketData::KlineSeries(series) => series.data_id == data_id,
                NormalizedMarketData::Quote(quote) => quote.data_id == data_id,
            })
            .ok_or(CoreIrIndicatorEvaluatorError::MissingInputData)?;
        match item {
            NormalizedMarketData::KlineSeries(series) => {
                exchanges.insert(series.exchange.clone());
                symbols.insert(series.symbol.clone());
            }
            NormalizedMarketData::Quote(quote) => {
                exchanges.insert(quote.exchange.clone());
                symbols.insert(quote.symbol.clone());
            }
        }
    }
    Ok((exchanges, symbols))
}

fn collect_custom_data_ids_from_expr(expr: &CustomValueExpr, out: &mut BTreeSet<String>) {
    match expr {
        CustomValueExpr::Number { .. } => {}
        CustomValueExpr::Input { data_id, .. } | CustomValueExpr::WindowAgg { data_id, .. } => {
            out.insert(data_id.clone());
        }
        CustomValueExpr::Binary { left, right, .. } => {
            collect_custom_data_ids_from_expr(left, out);
            collect_custom_data_ids_from_expr(right, out);
        }
        CustomValueExpr::Unary { value, .. } => collect_custom_data_ids_from_expr(value, out),
    }
}

fn custom_reference_price(
    spec: &CustomExprSpec,
    normalized_data: &[NormalizedMarketData],
) -> Option<f64> {
    let mut data_ids = BTreeSet::new();
    collect_custom_data_ids_from_expr(&spec.predicate.left, &mut data_ids);
    collect_custom_data_ids_from_expr(&spec.predicate.right, &mut data_ids);
    let first = data_ids.into_iter().next()?;
    latest_field_value(normalized_data, &first, SeriesField::MidOrClose).ok()
}

fn param_or_default(indicator: &IndicatorNode, key: &str, default: f64) -> f64 {
    indicator
        .params
        .get(key)
        .and_then(|value| value.as_f64())
        .unwrap_or(default)
}

fn find_kline_snapshot<'a>(
    normalized_data: &'a [NormalizedMarketData],
    indicator: &IndicatorNode,
) -> Result<&'a KlineSeriesSnapshot, CoreIrIndicatorEvaluatorError> {
    let Some(data_id) = indicator.inputs.iter().find_map(|input| match input {
        qrpc_core_ir::SeriesExpr::DataRef { data_id } => Some(data_id.as_str()),
        _ => None,
    }) else {
        return Err(CoreIrIndicatorEvaluatorError::MissingInputData);
    };

    normalized_data
        .iter()
        .find_map(|item| match item {
            NormalizedMarketData::KlineSeries(series) if series.data_id == data_id => Some(series),
            _ => None,
        })
        .ok_or(CoreIrIndicatorEvaluatorError::MissingInputData)
}

fn find_quote_snapshot<'a>(
    normalized_data: &'a [NormalizedMarketData],
    indicator: &IndicatorNode,
) -> Result<&'a QuoteSnapshot, CoreIrIndicatorEvaluatorError> {
    let Some(data_id) = indicator.inputs.iter().find_map(|input| match input {
        qrpc_core_ir::SeriesExpr::DataRef { data_id } => Some(data_id.as_str()),
        _ => None,
    }) else {
        return Err(CoreIrIndicatorEvaluatorError::MissingInputData);
    };

    normalized_data
        .iter()
        .find_map(|item| match item {
            NormalizedMarketData::Quote(quote) if quote.data_id == data_id => Some(quote),
            _ => None,
        })
        .ok_or(CoreIrIndicatorEvaluatorError::MissingInputData)
}

fn find_quote_snapshots<'a>(
    normalized_data: &'a [NormalizedMarketData],
    indicator: &IndicatorNode,
) -> Result<Vec<&'a QuoteSnapshot>, CoreIrIndicatorEvaluatorError> {
    let data_ids = indicator
        .inputs
        .iter()
        .filter_map(|input| match input {
            qrpc_core_ir::SeriesExpr::DataRef { data_id } => Some(data_id.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if data_ids.is_empty() {
        return Err(CoreIrIndicatorEvaluatorError::MissingInputData);
    }

    let mut quotes = Vec::with_capacity(data_ids.len());
    for data_id in data_ids {
        let quote = normalized_data
            .iter()
            .find_map(|item| match item {
                NormalizedMarketData::Quote(quote) if quote.data_id == data_id => Some(quote),
                _ => None,
            })
            .ok_or(CoreIrIndicatorEvaluatorError::MissingInputData)?;
        quotes.push(quote);
    }

    Ok(quotes)
}

fn moving_average(bars: &[NormalizedKline], window: usize) -> Option<f64> {
    if bars.len() < window {
        return None;
    }
    let slice = &bars[bars.len() - window..];
    Some(slice.iter().map(|bar| bar.close).sum::<f64>() / window as f64)
}

fn scaled_threshold_strength(upper: f64, lower: f64, range: f64) -> f64 {
    if range <= 0.0 {
        return 0.0;
    }
    ((upper - lower) / range).clamp(0.0, 1.0)
}

fn scaled_ratio_strength(value: f64, reference: f64) -> f64 {
    if reference.abs() <= f64::EPSILON {
        return 0.0;
    }
    (value / reference.abs()).clamp(0.0, 1.0)
}

fn ema_series(values: &[f64], period: usize) -> Option<Vec<f64>> {
    if period == 0 || values.len() < period {
        return None;
    }
    let multiplier = 2.0 / (period as f64 + 1.0);
    let seed = values.iter().take(period).sum::<f64>() / period as f64;
    let mut ema_values = vec![seed];
    for value in values.iter().skip(period) {
        let next = (*value - ema_values.last()?) * multiplier + ema_values.last()?;
        ema_values.push(next);
    }
    Some(ema_values)
}

fn simple_moving_average_series(values: &[f64], period: usize) -> Option<Vec<f64>> {
    if period == 0 || values.len() < period {
        return None;
    }
    Some(
        values
            .windows(period)
            .map(|window| window.iter().sum::<f64>() / period as f64)
            .collect(),
    )
}

fn relative_strength_index(
    bars: &[NormalizedKline],
    period: usize,
    smoothing_method: f64,
) -> Option<f64> {
    if period == 0 || bars.len() <= period {
        return None;
    }

    let mut gains = Vec::with_capacity(bars.len().saturating_sub(1));
    let mut losses = Vec::with_capacity(bars.len().saturating_sub(1));
    for window in bars.windows(2) {
        let delta = window[1].close - window[0].close;
        gains.push(delta.max(0.0));
        losses.push((-delta).max(0.0));
    }

    let average_tail = |series: &[f64]| -> Option<f64> {
        let window = series.get(series.len().checked_sub(period)?)?;
        let _ = window;
        Some(series[series.len() - period..].iter().sum::<f64>() / period as f64)
    };

    let (average_gain, average_loss) = if smoothing_method >= 1.5 {
        let gain_series = simple_moving_average_series(&gains, period)?;
        let loss_series = simple_moving_average_series(&losses, period)?;
        (*gain_series.last()?, *loss_series.last()?)
    } else if smoothing_method >= 0.5 {
        let gain_series = ema_series(&gains, period)?;
        let loss_series = ema_series(&losses, period)?;
        (*gain_series.last()?, *loss_series.last()?)
    } else {
        let mut average_gain = average_tail(&gains)?;
        let mut average_loss = average_tail(&losses)?;
        for index in period..gains.len() {
            average_gain = ((average_gain * (period as f64 - 1.0)) + gains[index]) / period as f64;
            average_loss = ((average_loss * (period as f64 - 1.0)) + losses[index]) / period as f64;
        }
        (average_gain, average_loss)
    };

    if average_loss <= f64::EPSILON {
        return Some(100.0);
    }
    let rs = average_gain / average_loss;
    Some(100.0 - (100.0 / (1.0 + rs)))
}

fn macd_histogram(
    bars: &[NormalizedKline],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Option<(f64, f64, f64)> {
    if fast_period == 0 || slow_period == 0 || signal_period == 0 || fast_period >= slow_period {
        return None;
    }
    let closes: Vec<f64> = bars.iter().map(|bar| bar.close).collect();
    let fast_ema = ema_series(&closes, fast_period)?;
    let slow_ema = ema_series(&closes, slow_period)?;
    let mut macd_values = Vec::new();
    let offset = slow_period.saturating_sub(fast_period);
    for (index, slow_value) in slow_ema.iter().enumerate() {
        let fast_index = index + offset;
        let fast_value = *fast_ema.get(fast_index)?;
        macd_values.push(fast_value - slow_value);
    }
    let signal_values = ema_series(&macd_values, signal_period)?;
    let macd_line = *macd_values.last()?;
    let signal_line = *signal_values.last()?;
    Some((macd_line, signal_line, macd_line - signal_line))
}

fn momentum_ratio(bars: &[NormalizedKline], lookback: usize) -> Option<f64> {
    if lookback == 0 || bars.len() <= lookback {
        return None;
    }
    let last = bars.last()?.close;
    let base = bars.get(bars.len() - lookback - 1)?.close;
    if base.abs() <= f64::EPSILON {
        return None;
    }
    Some((last / base) - 1.0)
}

fn z_score(bars: &[NormalizedKline], window: usize) -> Option<f64> {
    if window == 0 || bars.len() < window {
        return None;
    }
    let slice = &bars[bars.len() - window..];
    let mean = slice.iter().map(|bar| bar.close).sum::<f64>() / window as f64;
    let variance = slice
        .iter()
        .map(|bar| {
            let delta = bar.close - mean;
            delta * delta
        })
        .sum::<f64>()
        / window as f64;
    let std_dev = variance.sqrt();
    if std_dev <= f64::EPSILON {
        return Some(0.0);
    }
    Some((slice.last()?.close - mean) / std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::{
        DataQualitySnapshot, Exchange, KlineSeriesSnapshot, MarketType, QuoteSnapshot,
        SourceStatus, Symbol,
    };
    use qrpc_core_ir::{
        AlignAsofSpec, AlignDirection, ArithmeticOp, ComparisonOp, CustomExprSpec,
        CustomPredicateExpr, CustomValueExpr, SeriesAggregation, SeriesExpr, SeriesField,
        SignalKind, SpreadSpec, SpreadValueKind, CUSTOM_EXPR_V1_VERSION,
    };
    use serde_json::json;

    fn quote(
        data_id: &str,
        exchange: Exchange,
        mid_price: f64,
        ts_ms: u64,
    ) -> NormalizedMarketData {
        NormalizedMarketData::Quote(QuoteSnapshot {
            data_id: data_id.into(),
            exchange,
            symbol: Symbol::BtcUsdt,
            market_type: MarketType::Spot,
            best_bid: mid_price - 5.0,
            best_ask: mid_price + 5.0,
            bid_size: 10.0,
            ask_size: 10.0,
            mid_price,
            ts_ms,
            source_latency_ms: 0,
            source_status: SourceStatus::Healthy,
            data_quality: DataQualitySnapshot::default(),
        })
    }

    fn kline(data_id: &str, exchange: Exchange, closes: &[(u64, f64)]) -> NormalizedMarketData {
        NormalizedMarketData::KlineSeries(KlineSeriesSnapshot {
            data_id: data_id.into(),
            exchange: exchange.clone(),
            symbol: Symbol::BtcUsdt,
            market_type: MarketType::Spot,
            interval: "1m".into(),
            bars: closes
                .iter()
                .map(|(close_time_ms, close)| NormalizedKline {
                    exchange: exchange.clone(),
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    interval: "1m".into(),
                    open_time_ms: close_time_ms.saturating_sub(60_000),
                    close_time_ms: *close_time_ms,
                    open: *close - 10.0,
                    high: *close + 10.0,
                    low: *close - 20.0,
                    close: *close,
                    volume: 100.0,
                })
                .collect(),
            window_len: closes.len(),
            ts_ms: closes.last().map(|(ts_ms, _)| *ts_ms).unwrap_or_default(),
            source_latency_ms: 0,
            source_status: SourceStatus::Healthy,
            data_quality: DataQualitySnapshot::default(),
        })
    }

    fn sample_kline_series(data_id: &str, prices: &[f64]) -> NormalizedMarketData {
        let closes = prices
            .iter()
            .enumerate()
            .map(|(index, close)| ((index as u64 + 1) * 60_000, *close))
            .collect::<Vec<_>>();
        kline(data_id, Exchange::Binance, &closes)
    }

    #[test]
    fn evaluates_spread_indicator_from_two_quote_sources() {
        let indicator = IndicatorNode {
            indicator_id: "spread_1".into(),
            kind: CoreIndicatorKind::Spread,
            inputs: vec![
                SeriesExpr::DataRef {
                    data_id: "binance_quote".into(),
                },
                SeriesExpr::DataRef {
                    data_id: "okx_quote".into(),
                },
            ],
            spread_spec: None,
            custom_expr: None,
            params: BTreeMap::from([("max_time_diff_ms".into(), json!(5_000.0))]),
        };

        let evaluation = evaluate_indicator_signal(
            &indicator,
            None,
            &[
                quote("binance_quote", Exchange::Binance, 50_000.0, 100),
                quote("okx_quote", Exchange::Okx, 50_350.0, 102),
            ],
        )
        .unwrap();

        assert_eq!(
            evaluation.exchange_scope,
            vec![Exchange::Binance, Exchange::Okx]
        );
        assert!(evaluation.derived_metrics["spread_bps"] > 60.0);
        assert_eq!(evaluation.derived_metrics["time_skew_ms"], 2.0);
    }

    #[test]
    fn evaluates_restricted_custom_indicator_against_kline_window() {
        let indicator = IndicatorNode {
            indicator_id: "custom_1".into(),
            kind: CoreIndicatorKind::Custom,
            inputs: vec![SeriesExpr::DataRef {
                data_id: "btc_kline".into(),
            }],
            spread_spec: None,
            custom_expr: Some(CustomExprSpec {
                schema_version: CUSTOM_EXPR_V1_VERSION.into(),
                signal_kind: SignalKind::Long,
                predicate: CustomPredicateExpr {
                    left: CustomValueExpr::WindowAgg {
                        data_id: "btc_kline".into(),
                        field: SeriesField::Close,
                        window_size: 3,
                        agg: SeriesAggregation::Mean,
                    },
                    op: ComparisonOp::Gt,
                    right: CustomValueExpr::Number { value: 105.0 },
                },
                strength: Some(CustomValueExpr::Binary {
                    left: Box::new(CustomValueExpr::Input {
                        data_id: "btc_kline".into(),
                        field: SeriesField::Close,
                    }),
                    op: ArithmeticOp::Sub,
                    right: Box::new(CustomValueExpr::Number { value: 100.0 }),
                }),
                confidence: 0.9,
            }),
            params: BTreeMap::new(),
        };

        let evaluation = evaluate_indicator_signal(
            &indicator,
            None,
            &[sample_kline_series(
                "btc_kline",
                &[100.0, 106.0, 110.0, 112.0],
            )],
        )
        .unwrap();

        assert_eq!(evaluation.side, SignalSide::Long);
        assert!(evaluation.strength > 0.0);
        assert_eq!(evaluation.confidence, 0.9);
        assert!(evaluation.reason.contains("custom expr"));
    }

    #[test]
    fn rejects_custom_indicator_division_by_zero_at_runtime() {
        let indicator = IndicatorNode {
            indicator_id: "custom_bad".into(),
            kind: CoreIndicatorKind::Custom,
            inputs: vec![SeriesExpr::DataRef {
                data_id: "btc_kline".into(),
            }],
            spread_spec: None,
            custom_expr: Some(CustomExprSpec {
                schema_version: CUSTOM_EXPR_V1_VERSION.into(),
                signal_kind: SignalKind::Long,
                predicate: CustomPredicateExpr {
                    left: CustomValueExpr::Binary {
                        left: Box::new(CustomValueExpr::Input {
                            data_id: "btc_kline".into(),
                            field: SeriesField::Close,
                        }),
                        op: ArithmeticOp::Div,
                        right: Box::new(CustomValueExpr::Number { value: 0.0 }),
                    },
                    op: ComparisonOp::Gt,
                    right: CustomValueExpr::Number { value: 1.0 },
                },
                strength: None,
                confidence: 0.8,
            }),
            params: BTreeMap::new(),
        };

        let err = evaluate_indicator_signal(
            &indicator,
            None,
            &[sample_kline_series(
                "btc_kline",
                &[100.0, 106.0, 110.0, 112.0],
            )],
        )
        .unwrap_err();

        assert_eq!(err, CoreIrIndicatorEvaluatorError::InvalidCustomExpression);
    }

    #[test]
    fn evaluates_typed_spread_with_resample_and_window_agg() {
        let indicator = IndicatorNode {
            indicator_id: "spread_typed".into(),
            kind: CoreIndicatorKind::Spread,
            inputs: vec![
                SeriesExpr::DataRef {
                    data_id: "binance_quote".into(),
                },
                SeriesExpr::DataRef {
                    data_id: "okx_kline".into(),
                },
            ],
            spread_spec: Some(SpreadSpec {
                left: SeriesExpr::DataField {
                    data_id: "binance_quote".into(),
                    field: SeriesField::MidOrClose,
                },
                right: SeriesExpr::WindowAgg {
                    input: Box::new(SeriesExpr::Resample {
                        input: Box::new(SeriesExpr::DataField {
                            data_id: "okx_kline".into(),
                            field: SeriesField::Close,
                        }),
                        period_ms: 60_000,
                        agg: SeriesAggregation::Last,
                    }),
                    window_size: 3,
                    agg: SeriesAggregation::Mean,
                },
                align: AlignAsofSpec {
                    direction: AlignDirection::Backward,
                    tolerance_ms: 120_000,
                },
                output: SpreadValueKind::Bps,
            }),
            custom_expr: None,
            params: BTreeMap::new(),
        };

        let evaluation = evaluate_indicator_signal(
            &indicator,
            None,
            &[
                quote("binance_quote", Exchange::Binance, 50_400.0, 240_000),
                kline(
                    "okx_kline",
                    Exchange::Okx,
                    &[(60_000, 49_800.0), (120_000, 50_100.0), (180_000, 50_400.0)],
                ),
            ],
        )
        .unwrap();

        assert!(evaluation.reason.contains("typed spread"));
        assert!((evaluation.derived_metrics["right_value"] - 50_100.0).abs() < 0.0001);
        assert!(evaluation.derived_metrics["spread_bps"].abs() > 50.0);
        assert_eq!(evaluation.derived_metrics["time_skew_ms"], 0.0);
    }
}
