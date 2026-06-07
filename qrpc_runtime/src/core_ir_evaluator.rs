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
    // v2.4.0 P2-J1: 濠婃垵濮╃粣妤€褰?O(N) 閺囧じ鍞?O(N*period)
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
mod test_harness;
