use std::collections::BTreeMap;

use super::*;

fn sample_runtime_protocol() -> RuntimeProtocolCoreConfig {
    RuntimeProtocolCoreConfig {
        data_sources: vec![DataSourceConfig {
            data_id: "binance_btc_1d".into(),
            exchange: Exchange::Binance,
            symbol: Symbol::BtcUsdt,
            market_type: MarketType::Spot,
            kind: DataKind::KlineSeries,
            days: Some(200),
            interval: Some("1d".into()),
            ping_enabled: false,
            request_interval_ms: None,
            enabled: true,
        }],
        intents: vec![IntentConfig {
            intent_id: "intent_rsi".into(),
            name: "RSI".into(),
            kind: IntentKind::Rsi,
            input_data_ids: vec!["binance_btc_1d".into()],
            params: BTreeMap::new(),
            enabled: true,
        }],
        agents: vec![AgentConfig {
            agent_id: "agent_main".into(),
            name: "Main Agent".into(),
            input_intent_ids: vec!["intent_rsi".into()],
            rebalance_symbols: vec![],
            rebalance_schedule: None,
            rebalance_allocation_kind: None,
            rebalance_rank_method: None,
            rebalance_score_normalize: None,
            rebalance_target_weights: vec![],
            params: BTreeMap::new(),
            enabled: true,
        }],
        risks: vec![RiskConfig {
            risk_id: "risk_main".into(),
            name: "Main Risk".into(),
            observed_agent_ids: vec!["agent_main".into()],
            max_position_ratio: 0.2,
            max_single_weight: None,
            max_concentration_ratio: None,
            max_symbol_net_exposure_ratio: None,
            max_portfolio_net_exposure_ratio: None,
            max_turnover: None,
            min_trade_weight: None,
            max_new_positions_per_rebalance: None,
            max_total_leverage: 3.0,
            max_exchange_leverage: 3.0,
            min_action_interval_ms: 100,
            enabled: true,
        }],
        initial_cash_balance: 100_000.0,
        taker_fee_bps: 10.0,
        default_slippage_bps: 5.0,
        total_cost_buffer_bps: 20.0,
    }
}

#[test]
fn canonical_digest_is_stable_for_equivalent_payloads() {
    let left = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_test",
        "mode": "paper"
    });
    let right = serde_json::json!({
        "compile_id": "compile_test",
        "mode": "paper",
        "graph_id": "graph_test"
    });

    let left_digest = canonical_json_sha256_digest(&left).unwrap();
    let right_digest = canonical_json_sha256_digest(&right).unwrap();

    assert_eq!(left_digest, right_digest);
    assert_eq!(
        left_digest.algorithm,
        ArtifactDigestAlgorithm::Sha256CanonicalJson
    );
}

#[test]
fn run_and_backtest_specs_capture_protocol_boundary() {
    let config = sample_runtime_protocol();
    let core_ir_digest = canonical_json_sha256_digest(&serde_json::json!({
        "ir_version": "quantpilot/core-ir/v1"
    }))
    .unwrap();

    let run_spec = RunSpec::from_runtime_protocol(
        RunSpecRuntimeProtocolInput {
            graph_id: "graph_test".to_string(),
            compile_id: "compile_test".to_string(),
            run_mode: RunModeSpec::Backtest,
            runtime_mode: "paper".to_string(),
            protocol_name: "quantpilot/minimal-sim/v1".to_string(),
            config_hash: "runtime-spec-hash".to_string(),
            core_ir_digest: core_ir_digest.clone(),
        },
        &config,
    );
    let snapshot = MarketDataSnapshotSpec::from_runtime_protocol(
        "snapshot_test",
        BacktestReplaySource::DeterministicMock,
        1_700_000_000_000,
        &config,
    );
    let backtest_spec = BacktestSpec::new(
        "backtest_test",
        BacktestReplaySource::DeterministicMock,
        1_700_000_000_000,
        run_spec.clone(),
        snapshot.clone(),
    );

    assert_eq!(run_spec.schema_version, RUN_SPEC_V1_VERSION);
    assert_eq!(run_spec.datasets.len(), 1);
    assert_eq!(
        run_spec.execution_assumptions.time_in_force,
        TimeInForce::Gtc
    );
    assert_eq!(snapshot.datasets[0].data_id, "binance_btc_1d");
    assert_eq!(backtest_spec.schema_version, BACKTEST_SPEC_V1_VERSION);
    assert_eq!(backtest_spec.run_spec.core_ir_digest, core_ir_digest);
    assert_eq!(backtest_spec.market_data_snapshot, snapshot);
}
