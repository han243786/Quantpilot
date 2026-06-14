use super::super::lower_script_to_runtime_config;
use crate::parse_quant_script_module;
use qrpc_compiler::compile_runtime_protocol_config;
use qrpc_core::IntentKind;

#[test]
fn rejects_non_admitted_cross_source_spread_formula_for_formal_lowering() {
    let module = parse_quant_script_module(
            r#"
fn strategy() {
    let data_binance_series = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let data_okx_series = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let intent_spread_signal = (data_okx_series.last() - data_binance_series.last()) / data_binance_series.last()
    if intent_spread_signal > 0.005 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

    let err = lower_script_to_runtime_config(&module).unwrap_err();
    assert!(err.to_string().contains("QPQSLOW001"));
}

#[test]
fn rejects_non_admitted_asymmetric_window_spread_formula_for_formal_lowering() {
    let module = parse_quant_script_module(
            r#"
fn strategy() {
    let data_binance_series = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let data_okx_series = fetch("BTCUSDT", exchange="okx", interval="5m", lookback=200)?
    let intent_spread_signal = (data_okx_series[3..].mean() - data_binance_series.last()) / data_binance_series.last()
    if intent_spread_signal > 0.004 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

    let err = lower_script_to_runtime_config(&module).unwrap_err();
    assert!(err.to_string().contains("QPQSLOW001"));
}

#[test]
fn lowers_admitted_explicit_spread_helper_into_quote_observe() {
    let module = parse_quant_script_module(
            r#"
fn strategy() {
    let data_binance_series = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let data_okx_series = fetch("BTCUSDT", exchange="okx", interval="5m", lookback=200)?
    let buy_leg = align_asof(resample(field(data_binance_series, name="bid"), every="5m", agg="last"), direction="backward", tolerance_ms=10000)
    let sell_leg = align_asof(field(data_okx_series, name="ask"), direction="backward", tolerance_ms=10000)
    let intent_spread_signal = spread(buy_leg, sell_leg, output="bps")
    if intent_spread_signal > 45 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    let intent = config
        .intents
        .iter()
        .find(|intent| intent.kind == IntentKind::QuoteObserve)
        .unwrap();

    assert_eq!(intent.params.get("spread_output_code"), Some(&1.0));
    assert_eq!(intent.params.get("spread_trigger_bps"), Some(&45.0));
    assert_eq!(intent.params.get("left_field_code"), Some(&1.0));
    assert_eq!(intent.params.get("right_field_code"), Some(&2.0));
    assert_eq!(
        intent.params.get("left_resample_period_ms"),
        Some(&300_000.0)
    );
    assert_eq!(intent.params.get("left_resample_agg_code"), Some(&0.0));
    assert_eq!(intent.params.get("align_direction_code"), Some(&0.0));
    assert_eq!(intent.params.get("max_time_diff_ms"), Some(&10_000.0));
    assert_eq!(intent.params.get("comparison_shape_code"), Some(&1.0));
    assert_eq!(intent.params.get("comparison_op_code"), Some(&2.0));
    assert_eq!(intent.params.get("comparison_threshold"), Some(&45.0));

    let compiled = compile_runtime_protocol_config(&config).unwrap();
    let spread_spec = compiled.core_ir.indicators[0].spread_spec.as_ref().unwrap();
    assert_eq!(spread_spec.output, qrpc_core_ir::SpreadValueKind::Bps);
    assert_eq!(
        spread_spec.align.direction,
        qrpc_core_ir::AlignDirection::Backward
    );
    assert_eq!(spread_spec.align.tolerance_ms, 10_000);
    match &spread_spec.left {
        qrpc_core_ir::SeriesExpr::Resample {
            period_ms,
            agg,
            input,
        } => {
            assert_eq!(*period_ms, 300_000);
            assert_eq!(*agg, qrpc_core_ir::SeriesAggregation::Last);
            match input.as_ref() {
                qrpc_core_ir::SeriesExpr::DataField { field, .. } => {
                    assert_eq!(*field, qrpc_core_ir::SeriesField::BidOrClose);
                }
                other => panic!("expected data field under resample, got {other:?}"),
            }
        }
        other => panic!("expected resample left leg, got {other:?}"),
    }
    assert_eq!(
        compiled.core_ir.signal_rules[0].condition,
        qrpc_core_ir::ScalarExpr::Compare {
            left: Box::new(qrpc_core_ir::ScalarExpr::Ref {
                name: "intent_spread".into(),
            }),
            op: qrpc_core_ir::ComparisonOp::Gt,
            right: Box::new(qrpc_core_ir::ScalarExpr::Number { value: 45.0 }),
        }
    );
}

#[test]
fn rejects_non_admitted_helper_annotated_formula_spread() {
    let module = parse_quant_script_module(
            r#"
fn strategy() {
    let data_binance_series = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let data_okx_series = fetch("BTCUSDT", exchange="okx", interval="5m", lookback=200)?
    let buy_leg = field(data_binance_series, name="bid")
    let sell_leg = align(resample(field(data_okx_series, name="ask"), every="5m", agg="last"), direction="nearest", tolerance_ms=7500)
    let intent_spread_signal = (sell_leg - buy_leg) / buy_leg
    if intent_spread_signal > 0.0045 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

    let err = lower_script_to_runtime_config(&module).unwrap_err();
    assert!(err.to_string().contains("QPQSLOW001"));
}
