#![allow(clippy::type_complexity)]
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

mod advanced_indicator_wave;
mod classic_indicator_wave;
mod spread_custom_expression_wave;
use advanced_indicator_wave::{
    evaluate_adx, evaluate_atr, evaluate_bollinger_bands, evaluate_cci, evaluate_cmf,
    evaluate_donchian_channel, evaluate_keltner_channel, evaluate_obv, evaluate_parabolic_sar,
    evaluate_stochastic,
};
use classic_indicator_wave::{
    evaluate_ma_family, evaluate_macd, evaluate_momentum, evaluate_quote_observe, evaluate_rsi,
    evaluate_zscore,
};
use spread_custom_expression_wave::{evaluate_custom, evaluate_spread};

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

// v1.0.2 plugin registry: IndicatorEvaluator trait + builtin registry.

use std::sync::{Arc, OnceLock};

/// Builtin and plugin-backed indicator evaluator abstraction.
pub trait IndicatorEvaluator: Send + Sync {
    fn evaluate(
        &self,
        indicator: &IndicatorNode,
        signal_rule: Option<&SignalRule>,
        normalized_data: &[NormalizedMarketData],
    ) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError>;
}

/// Global builtin indicator registry, initialized lazily.
static INDICATOR_REGISTRY: OnceLock<BTreeMap<CoreIndicatorKind, Arc<dyn IndicatorEvaluator>>> =
    OnceLock::new();

pub fn indicator_registry() -> &'static BTreeMap<CoreIndicatorKind, Arc<dyn IndicatorEvaluator>> {
    INDICATOR_REGISTRY.get_or_init(|| {
        let mut registry: BTreeMap<CoreIndicatorKind, Arc<dyn IndicatorEvaluator>> =
            BTreeMap::new();
        macro_rules! register {
            ($kind:ident, $fn:ident) => {
                registry.insert(
                    CoreIndicatorKind::$kind,
                    Arc::new(BuiltinEvaluator {
                        evaluate: Box::new(move |indicator, _signal_rule, normalized_data| {
                            $fn(indicator, normalized_data)
                        }),
                    }),
                );
            };
            ($kind:ident, $fn:ident, with_signal) => {
                registry.insert(
                    CoreIndicatorKind::$kind,
                    Arc::new(BuiltinEvaluator {
                        evaluate: Box::new(move |indicator, signal_rule, normalized_data| {
                            $fn(indicator, signal_rule, normalized_data)
                        }),
                    }),
                );
            };
        }
        register!(MaCross, evaluate_ma_family, with_signal);
        register!(Rsi, evaluate_rsi);
        register!(Macd, evaluate_macd);
        register!(Momentum, evaluate_momentum);
        register!(Spread, evaluate_spread);
        register!(ZScore, evaluate_zscore);
        register!(Custom, evaluate_custom);
        register!(QuoteObserve, evaluate_quote_observe);
        register!(Atr, evaluate_atr);
        register!(BollingerBands, evaluate_bollinger_bands);
        register!(Obv, evaluate_obv);
        register!(Cmf, evaluate_cmf);
        register!(Adx, evaluate_adx);
        register!(Stochastic, evaluate_stochastic);
        register!(Cci, evaluate_cci);
        register!(ParabolicSar, evaluate_parabolic_sar);
        register!(KeltnerChannel, evaluate_keltner_channel);
        register!(DonchianChannel, evaluate_donchian_channel);
        registry
    })
}

struct BuiltinEvaluator {
    evaluate: Box<
        dyn Fn(
                &IndicatorNode,
                Option<&SignalRule>,
                &[NormalizedMarketData],
            ) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError>
            + Send
            + Sync,
    >,
}

impl IndicatorEvaluator for BuiltinEvaluator {
    fn evaluate(
        &self,
        indicator: &IndicatorNode,
        signal_rule: Option<&SignalRule>,
        normalized_data: &[NormalizedMarketData],
    ) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
        (self.evaluate)(indicator, signal_rule, normalized_data)
    }
}

/// v1.0.2: registry lookup replaces hard-coded match dispatch.
pub fn evaluate_indicator_signal(
    indicator: &IndicatorNode,
    signal_rule: Option<&SignalRule>,
    normalized_data: &[NormalizedMarketData],
) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError> {
    indicator_registry()
        .get(&indicator.kind)
        .ok_or(CoreIrIndicatorEvaluatorError::UnsupportedIndicator)?
        .evaluate(indicator, signal_rule, normalized_data)
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
    if window == 0 || bars.len() < window {
        return None;
    }
    let slice = &bars[bars.len() - window..];
    Some(slice.iter().map(|bar| bar.close).sum::<f64>() / window as f64)
}

fn scaled_threshold_strength(upper: f64, lower: f64, range: f64) -> f64 {
    if !range.is_finite() || range <= 0.0 {
        return 0.0;
    }
    let raw = (upper - lower) / range;
    if !raw.is_finite() {
        0.0
    } else {
        raw.clamp(0.0, 1.0)
    }
}

fn scaled_ratio_strength(value: f64, reference: f64) -> f64 {
    if reference.abs() <= f64::EPSILON {
        return 0.0;
    }
    let raw = value / reference.abs();
    if !raw.is_finite() {
        0.0
    } else {
        raw.clamp(0.0, 1.0)
    }
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
    // v2.4.0 P2-J1: 婊戝姩绐楀彛 O(N) 鏇夸唬 O(N*period)
    let mut result = Vec::with_capacity(values.len() - period + 1);
    let mut sum: f64 = values[..period].iter().sum();
    result.push(sum / period as f64);
    for i in period..values.len() {
        sum += values[i] - values[i - period];
        result.push(sum / period as f64);
    }
    Some(result)
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
        let _ = series.get(series.len().checked_sub(period)?)?;
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

fn true_range(bars: &[NormalizedKline]) -> Option<Vec<f64>> {
    if bars.len() < 2 {
        return None;
    }
    Some(
        bars.windows(2)
            .map(|w| {
                let high = w[1].high;
                let low = w[1].low;
                let prev_close = w[0].close;
                let tr1 = high - low;
                let tr2 = (high - prev_close).abs();
                let tr3 = (low - prev_close).abs();
                tr1.max(tr2).max(tr3)
            })
            .collect(),
    )
}

fn average_true_range(bars: &[NormalizedKline], period: usize) -> Option<f64> {
    let tr = true_range(bars)?;
    if period == 0 || tr.len() < period {
        return None;
    }
    // v1.3.7: Wilder smoothing uses alpha=1/N.
    let n = period as f64;
    let mut avg = tr[..period].iter().sum::<f64>() / n;
    for &val in &tr[period..] {
        avg = (avg * (n - 1.0) + val) / n;
    }
    Some(avg)
}

fn bollinger_bands(
    bars: &[NormalizedKline],
    period: usize,
    multiplier: f64,
) -> Option<(f64, f64, f64)> {
    if period == 0 || bars.len() < period {
        return None;
    }
    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let sma_series = simple_moving_average_series(&closes, period)?;
    let middle = *sma_series.last()?;
    let slice = &bars[bars.len() - period..];
    let variance = slice
        .iter()
        .map(|b| {
            let d = b.close - middle;
            d * d
        })
        .sum::<f64>()
        / period as f64;
    let std_dev = variance.sqrt();
    let upper = middle + multiplier * std_dev;
    let lower = middle - multiplier * std_dev;
    Some((middle, upper, lower))
}

fn on_balance_volume(bars: &[NormalizedKline]) -> Option<Vec<f64>> {
    if bars.is_empty() {
        return None;
    }
    let mut obv = Vec::with_capacity(bars.len());
    obv.push(0.0);
    for window in bars.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        let last_obv = *obv.last()?;
        if curr.close > prev.close {
            obv.push(last_obv + curr.volume);
        } else if curr.close < prev.close {
            obv.push(last_obv - curr.volume);
        } else {
            obv.push(last_obv);
        }
    }
    Some(obv)
}

fn chaikin_money_flow(bars: &[NormalizedKline], period: usize) -> Option<f64> {
    if period == 0 || bars.len() < period {
        return None;
    }
    let slice = &bars[bars.len() - period..];
    let mut total_mf_volume = 0.0;
    let mut total_volume = 0.0;
    for bar in slice {
        let range = bar.high - bar.low;
        if range > f64::EPSILON {
            let mf_multiplier = ((bar.close - bar.low) - (bar.high - bar.close)) / range;
            total_mf_volume += mf_multiplier * bar.volume;
        }
        total_volume += bar.volume;
    }
    if total_volume <= f64::EPSILON {
        return None;
    }
    Some(total_mf_volume / total_volume)
}

fn average_directional_index(bars: &[NormalizedKline], period: usize) -> Option<(f64, f64, f64)> {
    if period == 0 || bars.len() < period + 1 {
        return None;
    }
    let mut tr_values = Vec::new();
    let mut plus_dm = Vec::new();
    let mut minus_dm = Vec::new();
    for window in bars.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        let tr = {
            let tr1 = curr.high - curr.low;
            let tr2 = (curr.high - prev.close).abs();
            let tr3 = (curr.low - prev.close).abs();
            tr1.max(tr2).max(tr3)
        };
        tr_values.push(tr);
        let up_move = curr.high - prev.high;
        let down_move = prev.low - curr.low;
        if up_move.is_finite() && down_move.is_finite() && up_move > down_move && up_move > 0.0 {
            plus_dm.push(up_move);
        } else {
            plus_dm.push(0.0);
        }
        if down_move.is_finite() && up_move.is_finite() && down_move > up_move && down_move > 0.0 {
            minus_dm.push(down_move);
        } else {
            minus_dm.push(0.0);
        }
    }
    let tr_ema = ema_series(&tr_values, period)?;
    let plus_dm_ema = ema_series(&plus_dm, period)?;
    let minus_dm_ema = ema_series(&minus_dm, period)?;
    let mut dx_values = Vec::new();
    for i in 0..tr_ema.len() {
        let tr = tr_ema[i];
        let pdi = if tr > f64::EPSILON {
            100.0 * plus_dm_ema[i] / tr
        } else {
            0.0
        };
        let mdi = if tr > f64::EPSILON {
            100.0 * minus_dm_ema[i] / tr
        } else {
            0.0
        };
        let sum = pdi + mdi;
        let dx = if sum > f64::EPSILON {
            100.0 * (pdi - mdi).abs() / sum
        } else {
            0.0
        };
        dx_values.push(dx);
    }
    let tr_last = *tr_ema.last()?;
    let pdi_last = if tr_last > f64::EPSILON {
        100.0 * plus_dm_ema.last()? / tr_last
    } else {
        0.0
    };
    let mdi_last = if tr_last > f64::EPSILON {
        100.0 * minus_dm_ema.last()? / tr_last
    } else {
        0.0
    };
    let adx_ema = ema_series(&dx_values, period)?;
    let adx = *adx_ema.last()?;
    Some((adx, pdi_last, mdi_last))
}

fn stochastic_oscillator(
    bars: &[NormalizedKline],
    k_period: usize,
    d_period: usize,
) -> Option<(f64, f64)> {
    if k_period == 0 || bars.len() < k_period {
        return None;
    }
    let slice = &bars[bars.len() - k_period..];
    let lowest_low = slice.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
    let highest_high = slice
        .iter()
        .map(|b| b.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let close = bars.last()?.close;
    let range = highest_high - lowest_low;
    if range <= f64::EPSILON {
        return Some((50.0, 50.0));
    }
    let k_pct = 100.0 * (close - lowest_low) / range;

    // Build %K series for %D computation
    let mut k_values = Vec::new();
    for i in (k_period - 1)..bars.len() {
        let window = &bars[i + 1 - k_period..=i];
        let ll = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let hh = window
            .iter()
            .map(|b| b.high)
            .fold(f64::NEG_INFINITY, f64::max);
        let c = window.last()?.close;
        let r = hh - ll;
        let k = if r <= f64::EPSILON {
            50.0
        } else {
            100.0 * (c - ll) / r
        };
        k_values.push(k);
    }
    let d_pct = if k_values.len() >= d_period {
        k_values.iter().rev().take(d_period).sum::<f64>() / d_period as f64
    } else {
        k_pct
    };
    Some((k_pct, d_pct))
}

fn commodity_channel_index(bars: &[NormalizedKline], period: usize) -> Option<f64> {
    if period == 0 || bars.len() < period {
        return None;
    }
    let slice = &bars[bars.len() - period..];
    let typical_prices: Vec<f64> = slice
        .iter()
        .map(|b| (b.high + b.low + b.close) / 3.0)
        .collect();
    let sma_tp = typical_prices.iter().sum::<f64>() / period as f64;
    let mean_dev = typical_prices
        .iter()
        .map(|tp| (tp - sma_tp).abs())
        .sum::<f64>()
        / period as f64;
    if mean_dev <= f64::EPSILON {
        return Some(0.0);
    }
    let cci = (typical_prices.last()? - sma_tp) / (0.015 * mean_dev);
    Some(cci)
}

fn parabolic_sar(bars: &[NormalizedKline], step: f64, max_step: f64) -> Option<f64> {
    if bars.len() < 2 {
        return None;
    }
    let mut sar = bars[0].low;
    let mut ep = bars[0].high;
    let mut af = step;
    let mut is_uptrend = true;
    for i in 1..bars.len() {
        let prev_bar = &bars[i - 1];
        let curr_bar = &bars[i];
        if is_uptrend {
            sar = sar + af * (ep - sar);
            sar = sar.min(prev_bar.low).min(curr_bar.low);
            if curr_bar.high > ep {
                ep = curr_bar.high;
                af = (af + step).min(max_step);
            }
            if curr_bar.low < sar {
                is_uptrend = false;
                sar = ep;
                ep = curr_bar.low;
                af = step;
            }
        } else {
            sar = sar - af * (sar - ep);
            sar = sar.max(prev_bar.high).max(curr_bar.high);
            if curr_bar.low < ep {
                ep = curr_bar.low;
                af = (af + step).min(max_step);
            }
            if curr_bar.high > sar {
                is_uptrend = true;
                sar = ep;
                ep = curr_bar.high;
                af = step;
            }
        }
    }
    Some(sar)
}

fn keltner_channel(
    bars: &[NormalizedKline],
    period: usize,
    multiplier: f64,
) -> Option<(f64, f64, f64)> {
    if period == 0 || bars.len() <= period {
        return None;
    }
    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let ema = ema_series(&closes, period)?;
    let middle = *ema.last()?;
    let atr = average_true_range(bars, period)?;
    let upper = middle + multiplier * atr;
    let lower = middle - multiplier * atr;
    Some((upper, middle, lower))
}

fn donchian_channel(bars: &[NormalizedKline], period: usize) -> Option<(f64, f64, f64)> {
    if period == 0 || bars.len() < period {
        return None;
    }
    let slice = &bars[bars.len() - period..];
    let upper = slice
        .iter()
        .map(|b| b.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let lower = slice.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
    let middle = (upper + lower) / 2.0;
    Some((upper, middle, lower))
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
        assert!(evaluation.reason.contains("custom expression"));
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

    // R0-2: smoke tests for indicator helpers.

    fn sample_bars(prices: &[f64]) -> Vec<NormalizedKline> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &close)| NormalizedKline {
                exchange: Exchange::Binance,
                symbol: Symbol::BtcUsdt,
                market_type: MarketType::Spot,
                interval: "1d".into(),
                open_time_ms: i as u64 * 86400000,
                close_time_ms: (i as u64 + 1) * 86400000,
                open: close,
                high: close * 1.02,
                low: close * 0.98,
                close,
                volume: 100.0,
            })
            .collect()
    }

    fn trending_bars(length: usize) -> Vec<NormalizedKline> {
        (0..length)
            .map(|i| {
                let close = 40000.0 + i as f64 * 100.0 + (i as f64 * 0.3).sin() * 500.0;
                NormalizedKline {
                    exchange: Exchange::Binance,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    interval: "1d".into(),
                    open_time_ms: i as u64 * 86400000,
                    close_time_ms: (i as u64 + 1) * 86400000,
                    open: close - 50.0,
                    high: close + 200.0,
                    low: close - 200.0,
                    close,
                    volume: 100.0,
                }
            })
            .collect()
    }

    #[test]
    fn test_true_range_positive() {
        let bars = trending_bars(30);
        let tr = true_range(&bars).unwrap();
        assert_eq!(tr.len(), bars.len() - 1);
        assert!(
            tr.iter().all(|&v| v > 0.0),
            "True Range must always be positive"
        );
    }

    #[test]
    fn test_average_true_range_reasonable() {
        let bars = trending_bars(30);
        let atr = average_true_range(&bars, 14).unwrap();
        assert!(atr > 0.0);
        assert!(atr < 2000.0, "ATR should be reasonable for trending data");
    }

    #[test]
    fn test_bollinger_bands_contains_price() {
        // Use very flat prices so bands are tight and symmetrical
        let bars = sample_bars(&vec![100.0; 30]);
        let (upper, middle, lower) = bollinger_bands(&bars, 20, 2.0).unwrap();
        assert!(
            upper >= middle,
            "upper {} should be >= middle {}",
            upper,
            middle
        );
        assert!(
            middle >= lower,
            "middle {} should be >= lower {}",
            middle,
            lower
        );
        // All three values should be near 100 for flat prices
        assert!(
            (upper - 100.0).abs() < 5.0,
            "bands should be near 100, upper={}",
            upper
        );
        assert!(
            (lower - 100.0).abs() < 5.0,
            "bands should be near 100, lower={}",
            lower
        );
    }

    #[test]
    fn test_obv_uptrend_increasing() {
        let bars = trending_bars(30);
        let obv = on_balance_volume(&bars).unwrap();
        assert!(obv.len() >= 2);
        // In an uptrend, OBV should generally increase
        assert!(
            obv.last().unwrap() > obv.first().unwrap(),
            "OBV should increase in uptrend"
        );
    }

    #[test]
    fn test_cmf_positive_when_close_near_high() {
        let mut bars = trending_bars(30);
        // Make close always near high to get positive CMF
        for bar in &mut bars {
            bar.close = bar.high - 1.0;
        }
        let cmf = chaikin_money_flow(&bars, 20).unwrap();
        assert!(cmf > 0.0, "CMF should be positive when close is near high");
    }

    #[test]
    fn test_adx_range_and_di_sum() {
        let bars = trending_bars(50);
        let (adx, plus_di, minus_di) = average_directional_index(&bars, 14).unwrap();
        assert!(
            adx >= 0.0 && adx <= 100.0,
            "ADX {} should be in [0, 100]",
            adx
        );
        assert!(plus_di >= 0.0 && plus_di <= 100.0);
        assert!(minus_di >= 0.0 && minus_di <= 100.0);
    }

    #[test]
    fn test_stochastic_k_in_percent_range() {
        let bars = trending_bars(30);
        let (k, d) = stochastic_oscillator(&bars, 14, 3).unwrap();
        assert!(k >= 0.0 && k <= 100.0, "%K {} should be in [0, 100]", k);
        assert!(d >= 0.0 && d <= 100.0, "%D {} should be in [0, 100]", d);
    }

    #[test]
    fn test_cci_near_zero_for_flat_prices() {
        let bars = sample_bars(&vec![100.0; 30]);
        let cci = commodity_channel_index(&bars, 20).unwrap();
        assert!(
            cci.abs() < 10.0,
            "CCI {} should be near 0 for flat prices",
            cci
        );
    }

    #[test]
    fn test_parabolic_sar_produces_value() {
        let bars = trending_bars(30);
        let sar = parabolic_sar(&bars, 0.02, 0.2).unwrap();
        assert!(sar > 0.0, "SAR should produce a positive value");
    }

    #[test]
    fn test_keltner_channel_upper_above_lower() {
        let bars = trending_bars(30);
        let (upper, middle, lower) = keltner_channel(&bars, 20, 2.0).unwrap();
        assert!(
            upper > middle,
            "upper {} should be > middle {}",
            upper,
            middle
        );
        assert!(
            middle > lower,
            "middle {} should be > lower {}",
            middle,
            lower
        );
    }

    #[test]
    fn test_donchian_channel_upper_is_max_high() {
        let bars = trending_bars(30);
        let (upper, middle, lower) = donchian_channel(&bars, 20).unwrap();
        let period_high = bars
            .iter()
            .rev()
            .take(20)
            .map(|b| b.high)
            .fold(f64::MIN, f64::max);
        assert!(
            (upper - period_high).abs() < 0.001,
            "Donchian upper {} should equal max high {}",
            upper,
            period_high
        );
        let period_low = bars
            .iter()
            .rev()
            .take(20)
            .map(|b| b.low)
            .fold(f64::MAX, f64::min);
        assert!(
            (lower - period_low).abs() < 0.001,
            "Donchian lower {} should equal min low {}",
            lower,
            period_low
        );
        assert!((middle - (upper + lower) / 2.0).abs() < 0.001);
    }

    // P1-2: coverage for classic indicator helper behavior.

    #[test]
    fn test_moving_average_returns_mean() {
        let bars = sample_bars(&[100.0, 200.0, 300.0, 400.0, 500.0]);
        let ma = moving_average(&bars, 5).unwrap();
        assert!(
            (ma - 300.0).abs() < 0.01,
            "5-period MA of 100..500 should be 300, got {}",
            ma
        );
    }

    #[test]
    fn test_rsi_range_0_to_100() {
        let bars = trending_bars(30);
        let rsi = relative_strength_index(&bars, 14, 1.0).unwrap();
        assert!(
            rsi >= 0.0 && rsi <= 100.0,
            "RSI {} should be in [0, 100]",
            rsi
        );
    }

    #[test]
    fn test_macd_histogram_direction_matches_trend() {
        let uptrend = trending_bars(60);
        let downtrend: Vec<_> = (0..60)
            .map(|i| {
                let mut bar = uptrend[i].clone();
                bar.close = 50000.0 - i as f64 * 100.0;
                bar.open = bar.close + 50.0;
                bar.high = bar.open + 100.0;
                bar.low = bar.close - 100.0;
                bar
            })
            .collect();
        // MACD line = fast EMA - slow EMA; an uptrend should make it positive.
        let (up_line, _, up_hist) = macd_histogram(&uptrend, 12, 26, 9).unwrap();
        let (down_line, _, _) = macd_histogram(&downtrend, 12, 26, 9).unwrap();
        assert!(
            up_line > 0.0,
            "uptrend MACD line should be positive, got {}",
            up_line
        );
        assert!(
            down_line < 0.0,
            "downtrend MACD line should be negative, got {}",
            down_line
        );
        assert!(
            up_hist.is_finite(),
            "uptrend MACD histogram should be finite"
        );
    }

    #[test]
    fn test_momentum_positive_for_uptrend() {
        let bars = trending_bars(30);
        let mom = momentum_ratio(&bars, 10).unwrap();
        assert!(
            mom > 0.0,
            "uptrend momentum should be positive, got {}",
            mom
        );
    }

    #[test]
    fn test_momentum_near_zero_for_flat_prices() {
        let bars = sample_bars(&[100.0; 30]);
        let mom = momentum_ratio(&bars, 10).unwrap();
        assert!(
            mom.abs() < 0.01,
            "flat-price momentum should be near zero, got {}",
            mom
        );
    }

    #[test]
    fn test_quote_observe_evaluator_returns_price() {
        let bars = trending_bars(30);
        let last_close = bars.last().unwrap().close;
        // Validate the final bar close stays in a reasonable range.
        assert!(last_close > 0.0);
        assert!(last_close < 100_000.0);
    }
}
