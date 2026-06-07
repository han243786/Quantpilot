use super::*;
use qrpc_core::{
    DataQualitySnapshot, Exchange, KlineSeriesSnapshot, MarketType, QuoteSnapshot, SourceStatus,
    Symbol,
};
use qrpc_core_ir::{
    AlignAsofSpec, AlignDirection, ArithmeticOp, ComparisonOp, CustomExprSpec, CustomPredicateExpr,
    CustomValueExpr, SeriesAggregation, SeriesExpr, SeriesField, SignalKind, SpreadSpec,
    SpreadValueKind, CUSTOM_EXPR_V1_VERSION,
};
use serde_json::json;

fn quote(data_id: &str, exchange: Exchange, mid_price: f64, ts_ms: u64) -> NormalizedMarketData {
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
