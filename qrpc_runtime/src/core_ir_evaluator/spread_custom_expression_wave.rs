use super::*;

pub(super) fn evaluate_custom(
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
            "custom expression {:.4} {} {:.4}",
            left_value,
            comparison_name(&spec.predicate.op),
            right_value
        ),
        ttl_ms: 60_000,
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

pub(super) fn evaluate_spread(
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
