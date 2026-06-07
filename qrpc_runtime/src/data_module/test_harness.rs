use super::exchange_surface::{parse_okx_candles, parse_okx_ticker};
use super::mock_data_generation::pseudo_random;
use super::*;
use qrpc_core::{MarketType, RuntimeEventType, Symbol};
use qrpc_core_ir::{
    CoreMetadata, CoreSourceKind, CoreStrategyIr, CoreTimeInForce, DataBinding, DataBindingKind,
    ExecutionRule, ExecutionSizingKind,
};
use serde_json::json;
use std::collections::BTreeMap;

fn sample_core_ir_with_quote_binding() -> CoreStrategyIr {
    let mut source_hints = BTreeMap::new();
    source_hints.insert("exchange".into(), "binance".into());
    source_hints.insert("symbol".into(), "BTCUSDT".into());
    CoreStrategyIr {
        ir_version: qrpc_core::CORE_IR_V1_VERSION.to_string(),
        metadata: CoreMetadata {
            strategy_id: "data_test".into(),
            name: "Data Test".into(),
            source_kind: CoreSourceKind::RuntimeProtocol,
        },
        data_bindings: vec![DataBinding {
            data_id: "binance_btc_quote".into(),
            kind: DataBindingKind::Quote,
            source_hints,
        }],
        indicators: vec![],
        signal_rules: vec![],
        agent_policies: vec![],
        risk_policies: vec![],
        edges: vec![],
        execution: ExecutionRule {
            execution_id: "exec".into(),
            venue_kind: "paper".into(),
            sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
            slippage_bps: 5.0,
            taker_fee_bps: 10.0,
            total_cost_buffer_bps: 20.0,
            time_in_force: CoreTimeInForce::Gtc,
            params: BTreeMap::new(),
        },
    }
}

fn sample_core_ir_with_quote_and_kline_bindings() -> CoreStrategyIr {
    let mut quote_hints = BTreeMap::new();
    quote_hints.insert("exchange".into(), "binance".into());
    quote_hints.insert("symbol".into(), "BTCUSDT".into());

    let mut kline_hints = BTreeMap::new();
    kline_hints.insert("exchange".into(), "okx".into());
    kline_hints.insert("symbol".into(), "BTCUSDT".into());
    kline_hints.insert("timeframe".into(), "1m".into());

    let mut core_ir = sample_core_ir_with_quote_binding();
    core_ir.data_bindings.push(DataBinding {
        data_id: "okx_btc_kline".into(),
        kind: DataBindingKind::KlineSeries,
        source_hints: kline_hints,
    });
    core_ir.data_bindings[0].source_hints = quote_hints;
    core_ir
}

#[test]
fn okx_candles_are_parsed_in_time_order() {
    let payload = json!({
        "code": "0",
        "msg": "",
        "data": [
            ["1712707200000", "71000", "71500", "70500", "71200", "100", "1", "1", "1"],
            ["1712620800000", "70000", "71200", "69800", "71000", "120", "1", "1", "1"]
        ]
    });

    let source = DataSourceConfig {
        data_id: "okx_btc_1d".into(),
        exchange: Exchange::Okx,
        symbol: Symbol::BtcUsdt,
        market_type: MarketType::Spot,
        kind: DataKind::KlineSeries,
        days: Some(2),
        interval: Some("1d".into()),
        ping_enabled: false,
        request_interval_ms: None,
        enabled: true,
    };

    let bars = parse_okx_candles(&payload, &source).unwrap();
    assert_eq!(bars.len(), 2);
    assert!(bars[0].open_time < bars[1].open_time);
    assert_eq!(bars[0].open, 70_000.0);
    assert_eq!(bars[1].close, 71_200.0);
}

#[test]
fn okx_ticker_is_parsed_into_quote() {
    let payload = json!({
        "code": "0",
        "msg": "",
        "data": [{
            "bidPx": "71047.5",
            "askPx": "71047.6",
            "bidSz": "0.86246875",
            "askSz": "0.37879107",
            "ts": "1775718339217"
        }]
    });

    let quote = parse_okx_ticker(&payload).unwrap();
    assert_eq!(quote.best_bid, 71_047.5);
    assert_eq!(quote.best_ask, 71_047.6);
    assert_eq!(quote.ts, 1_775_718_339_217);
}

#[test]
fn builtin_data_module_keeps_mock_for_non_okx_sources() {
    let core_ir = sample_core_ir_with_quote_binding();
    let mut counts = BTreeMap::new();

    let output = BuiltinDataModule::default()
        .collect(DataCollectionRequest {
            cycle_name: "fast",
            core_ir: &core_ir,
            data_fetch_counts: &mut counts,
            now_ms: 10,
            trace_id: "trace",
        })
        .unwrap();

    assert_eq!(output.normalized_data.len(), 1);
    assert_eq!(output.events.len(), 2);
    assert_eq!(output.events[0].event_type, RuntimeEventType::DataUpdated);
    assert_eq!(
        output.events[0].payload["provider_key"],
        "builtin.data.mock"
    );
    assert_eq!(output.events[1].event_type, RuntimeEventType::RuntimeError);
    assert_eq!(output.events[1].payload["source_health"], "Missing");
    assert_eq!(counts.get("binance_btc_quote").copied(), Some(1));
}

#[test]
fn builtin_data_module_collects_mixed_sources_in_fast_cycle() {
    let core_ir = sample_core_ir_with_quote_and_kline_bindings();
    let mut counts = BTreeMap::new();

    let output = BuiltinDataModule::default()
        .collect(DataCollectionRequest {
            cycle_name: "fast",
            core_ir: &core_ir,
            data_fetch_counts: &mut counts,
            now_ms: 10,
            trace_id: "trace",
        })
        .unwrap();

    assert_eq!(output.normalized_data.len(), 2);
    assert_eq!(counts.get("binance_btc_quote").copied(), Some(1));
    assert_eq!(counts.get("okx_btc_kline").copied(), Some(1));
}

#[test]
fn data_sources_from_core_ir_restores_request_controls_from_source_hints() {
    let mut core_ir = sample_core_ir_with_quote_binding();
    core_ir.data_bindings[0]
        .source_hints
        .insert("ping_enabled".into(), "true".into());
    core_ir.data_bindings[0]
        .source_hints
        .insert("request_interval_ms".into(), "1500".into());

    let sources = data_sources_from_core_ir(&core_ir);

    assert_eq!(sources.len(), 1);
    assert!(sources[0].ping_enabled);
    assert_eq!(sources[0].request_interval_ms, Some(1_500));
}

#[test]
fn request_interval_uses_cached_snapshot_before_next_fetch_window() {
    let module = BuiltinDataModule::default();
    let source = DataSourceConfig {
        data_id: "binance_btc_quote".into(),
        exchange: Exchange::Binance,
        symbol: Symbol::BtcUsdt,
        market_type: MarketType::Spot,
        kind: DataKind::Quote,
        days: None,
        interval: None,
        ping_enabled: false,
        request_interval_ms: Some(5_000),
        enabled: true,
    };
    let mut counts = BTreeMap::new();

    let first = module
        .fetch_and_normalize(&source, 10_000, &mut counts)
        .unwrap();
    let second = module
        .fetch_and_normalize(&source, 12_000, &mut counts)
        .unwrap();

    assert_eq!(first.1.fallback, Some("mock"));
    assert_eq!(second.1.fallback, Some("request_interval"));
    assert_eq!(second.1.source_status, SourceStatus::Healthy);
    assert_eq!(counts.get("binance_btc_quote").copied(), Some(2));
}

#[test]
fn pseudo_random_is_deterministic() {
    let a = pseudo_random(42, 7);
    let b = pseudo_random(42, 7);
    assert_eq!(a, b, "same inputs produce same output");
}

#[test]
fn pseudo_random_range_is_normalized() {
    for i in 0..1000 {
        let val = pseudo_random(i, 12345);
        assert!(val >= -1.0 && val <= 1.0, "value {val} out of [-1,1]");
    }
}

#[test]
fn pseudo_random_different_seed_different_output() {
    let a = pseudo_random(42, 7);
    let b = pseudo_random(42, 8);
    assert_ne!(a, b, "different seeds produce different output");
}
