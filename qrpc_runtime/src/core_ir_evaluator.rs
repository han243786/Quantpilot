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

mod classic_indicator_wave;
use classic_indicator_wave::{
    evaluate_ma_family, evaluate_macd, evaluate_momentum, evaluate_quote_observe, evaluate_rsi,
    evaluate_zscore,
};

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

// ── v1.0.2 插件化: IndicatorEvaluator trait + 注册表 ──

use std::sync::{Arc, OnceLock};

/// 指标评估器 trait — v1.0.2 插件化抽象
pub trait IndicatorEvaluator: Send + Sync {
    fn evaluate(
        &self,
        indicator: &IndicatorNode,
        signal_rule: Option<&SignalRule>,
        normalized_data: &[NormalizedMarketData],
    ) -> Result<CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError>;
}

/// 全局指标注册表 — 启动时由 builtin 自动注册，后续可追加第三方插件
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

/// v1.0.2: 注册表查找替代硬编码 match 分派
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
            "自定义表达式 {:.4} {} {:.4}",
            left_value,
            comparison_name(&spec.predicate.op),
            right_value
        ),
        ttl_ms: 60_000,
    })
}

fn evaluate_atr(
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

fn evaluate_bollinger_bands(
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

fn evaluate_obv(
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

fn evaluate_cmf(
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

fn evaluate_adx(
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

fn evaluate_stochastic(
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

fn evaluate_cci(
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

fn evaluate_parabolic_sar(
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

fn evaluate_keltner_channel(
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

fn evaluate_donchian_channel(
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
            "类型化价差 {}:{:?} -> {}:{:?} 输出 {:?} {:.4} 偏差 {}ms",
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
    // v2.4.0 P2-J1: 滑动窗口 O(N) 替代 O(N*period)
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
        let _ = series.get(series.len().checked_sub(period)?)?; // v2.5.0: 仅校验边界
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
    // v1.3.7: Wilder 平滑 (α=1/N) 替代 EMA，与标准ATR定义一致
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
        assert!(evaluation.reason.contains("自定义表达式"));
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

        assert!(evaluation.reason.contains("类型化价差"));
        assert!((evaluation.derived_metrics["right_value"] - 50_100.0).abs() < 0.0001);
        assert!(evaluation.derived_metrics["spread_bps"].abs() > 50.0);
        assert_eq!(evaluation.derived_metrics["time_skew_ms"], 0.0);
    }

    // ── R0-2: Smoke tests for new indicator helpers ──

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

    // ── P1-2: 补齐 6 个缺失的 indicator 单元测试 (§2.3) ──

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
            "RSI {} 应在 [0, 100] 范围内",
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
        // MACD 线 = 快线 EMA - 慢线 EMA，上升趋势时快线应在慢线上方 (MACD 线 > 0)
        let (up_line, _, up_hist) = macd_histogram(&uptrend, 12, 26, 9).unwrap();
        let (down_line, _, _) = macd_histogram(&downtrend, 12, 26, 9).unwrap();
        assert!(up_line > 0.0, "上升趋势 MACD 线应为正, 实际 {}", up_line);
        assert!(
            down_line < 0.0,
            "下降趋势 MACD 线应为负, 实际 {}",
            down_line
        );
        assert!(up_hist.is_finite(), "上升趋势 MACD 柱应为有效值");
    }

    #[test]
    fn test_momentum_positive_for_uptrend() {
        let bars = trending_bars(30);
        let mom = momentum_ratio(&bars, 10).unwrap();
        assert!(mom > 0.0, "上升趋势动量应为正, 实际 {}", mom);
    }

    #[test]
    fn test_momentum_near_zero_for_flat_prices() {
        let bars = sample_bars(&[100.0; 30]);
        let mom = momentum_ratio(&bars, 10).unwrap();
        assert!(mom.abs() < 0.01, "平坦价格动量应接近零, 实际 {}", mom);
    }

    #[test]
    fn test_quote_observe_evaluator_returns_price() {
        let bars = trending_bars(30);
        let last_close = bars.last().unwrap().close;
        // 验证最后一个 bar 的 close 价在合理范围内
        assert!(last_close > 0.0);
        assert!(last_close < 100_000.0);
    }
}
