// v1.1.2: main 函数体已门控为 dev_tools feature
// 生产构建中此二进制仅打印提示并退出
// v2.4.0 G8: 此工具直接构造 RuntimeProtocolCoreConfig 绕过 QS 管道,
// 仅用于开发截图生成, GP §1.1/§1.3 豁免 — 正常编译路径不变。

#[cfg(feature = "dev_tools")]
fn save_backtest(label: &str, config: &RuntimeProtocolCoreConfig) {
    let compiled = compile_runtime_protocol_config(config).unwrap();
    let mut sandbox = FastBacktestSandbox::with_mock_replay(compiled, 1_730_000_000_000).unwrap();
    sandbox.start().unwrap();
    let output = sandbox.run_backtest().unwrap();

    let digest = serde_json::json!({"algorithm":"sha256_canonical_json","value":""});
    let exec = serde_json::json!({"initial_cash_balance":100000.0,"taker_fee_bps":config.taker_fee_bps,"default_slippage_bps":config.default_slippage_bps,"total_cost_buffer_bps":config.total_cost_buffer_bps,"time_in_force":"Gtc","allow_partial_fills":false,"latency_assumption_ms":null});
    let manifest = serde_json::json!({
        "schema_version":"quantpilot/reproducibility-manifest/v1","manifest_id":format!("manifest_{label}"),
        "backtest_id":label,"graph_id":format!("graph_{label}"),"compile_id":format!("compile_{label}"),
        "created_at_ms":1_730_000_000_000u64,"protocol_name":format!("Strategy {label}"),"config_hash":format!("hash_{label}"),
        "account":{"equity_estimate":100000.0,"cash_balance":output.final_portfolio.cash_balance,"available_cash_balance":output.final_portfolio.available_cash_balance,"frozen_cash_balance":output.final_portfolio.frozen_cash_balance,"total_leverage":0.0,"total_gross_notional":0.0,"total_net_notional":0.0,"positions":output.final_portfolio.positions.len(),"open_order_count":output.final_portfolio.open_orders.len(),"open_orders":[]},
        "summary":output.summary,
        "backtest_spec":{
            "schema_version":"quantpilot/backtest-spec/v1","backtest_id":label,"replay_source":"deterministic_mock","requested_at_ms":1_730_000_000_000u64,
            "run_spec":{"schema_version":"quantpilot/run-spec/v1","run_mode":"backtest","graph_id":format!("graph_{label}"),"compile_id":format!("compile_{label}"),"runtime_mode":"backtest","protocol_name":format!("Strategy {label}"),"config_hash":format!("hash_{label}"),"datasets":[],"execution_assumptions":exec,"core_ir_digest":digest},
            "market_data_snapshot":{"snapshot_id":format!("snap_{label}"),"replay_source":"deterministic_mock","captured_at_ms":1_730_000_000_000u64,"datasets":[],"quotes":[],"klines":[]}
        },
        "compile_artifacts":null,
        "governance":{"schema_version":"quantpilot/runtime-governance/v1","governance_source":"legacy_default","capability_hash":"sha256:screenshot_test","strategy_version":"v1","parameter_version":"v1","deployment_revision":"v1","capability_api_version":"quantpilot/capabilities/v1","runtime_support_boundary":{"runtime_modes":["paper"],"execution_module_keys":["builtin.execution.paper"]},"indicator_kinds":["ma_cross","rsi","macd","momentum","spread","z_score"],"attested_at_ms":1_730_000_000_000u64,"attestation_signature":"","permission_boundary":{"model_version":"quantpilot/permission-boundary/v1","execution_owner_module":"builtin.execution.paper","live_execution_allowed":false,"ai_write_policy":"proposal_only","plugin_network_default":"deny","non_execution_order_access":"deny"}},
        "output_artifacts":[],"backtest_output_digest":digest
    });

    let dir = std::path::PathBuf::from("storage")
        .join("backtests")
        .join(label);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(
        dir.join("backtest_output.json"),
        serde_json::to_string_pretty(&output).unwrap(),
    )
    .unwrap();
    fs::write(dir.join("event_log.json"), serde_json::json!({"schema_version":"v1","artifact_id":"","backtest_id":label,"digest":digest,"event_count":0,"events":[]}).to_string()).unwrap();
    fs::write(dir.join("trade_ledger.json"), serde_json::json!({"schema_version":"v1","artifact_id":"","backtest_id":label,"digest":digest,"trades":[],"trade_count":0,"summary":null}).to_string()).unwrap();
    fs::write(dir.join("equity_curve.json"), serde_json::json!({"schema_version":"v1","artifact_id":"","backtest_id":label,"digest":digest,"points":output.equity_curve,"point_count":output.equity_curve.len()}).to_string()).unwrap();
    fs::write(dir.join("metrics.json"), serde_json::json!({"schema_version":"v1","artifact_id":"","backtest_id":label,"digest":digest,"summary":output.summary,"event_count":0,"session_count":output.sessions.len(),"started_at_ms":output.started_at_ms,"ended_at_ms":output.ended_at_ms,"final_account":{"cash_balance":output.final_portfolio.cash_balance,"available_cash_balance":output.final_portfolio.available_cash_balance,"frozen_cash_balance":output.final_portfolio.frozen_cash_balance,"total_leverage":0.0,"total_gross_notional":0.0,"total_net_notional":0.0,"positions":output.final_portfolio.positions.len(),"open_order_count":output.final_portfolio.open_orders.len(),"open_orders":[]},"execution_assumptions":null}).to_string()).unwrap();

    println!(
        "{label}: steps={} trades={} return={:.2}% sharpe={:.2}",
        output.summary.step_count,
        output.summary.trade_count,
        output.summary.total_return_ratio * 100.0,
        output.summary.risk_adjusted.sharpe_ratio
    );
}

#[cfg(feature = "dev_tools")]
fn main() {
    let _ = fs::create_dir_all("storage/backtests");
    // 提高波动率使价格线像股票一样波动,产生频繁成交
    qrpc_runtime::set_mock_volatility(0.10);

    // 策略A: 激进型,满仓,低费率
    save_backtest(
        "sc_a_aggressive",
        &RuntimeProtocolCoreConfig {
            data_sources: vec![DataSourceConfig {
                data_id: "d1".into(),
                exchange: Exchange::Binance,
                symbol: Symbol::BtcUsdt,
                market_type: MarketType::Spot,
                kind: DataKind::KlineSeries,
                days: Some(300),
                interval: Some("1d".into()),
                ping_enabled: false,
                request_interval_ms: None,
                enabled: true,
            }],
            intents: vec![
                IntentConfig {
                    intent_id: "buy".into(),
                    name: "Buy".into(),
                    kind: IntentKind::LongTermBuy,
                    input_data_ids: vec!["d1".into()],
                    params: BTreeMap::new(),
                    enabled: true,
                },
                IntentConfig {
                    intent_id: "sell".into(),
                    name: "Sell".into(),
                    kind: IntentKind::LongTermSell,
                    input_data_ids: vec!["d1".into()],
                    params: BTreeMap::new(),
                    enabled: true,
                },
            ],
            agents: vec![AgentConfig {
                agent_id: "a1".into(),
                name: "Agent".into(),
                input_intent_ids: vec!["buy".into(), "sell".into()],
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
                risk_id: "r1".into(),
                name: "Risk".into(),
                observed_agent_ids: vec!["a1".into()],
                max_position_ratio: 1.0,
                max_single_weight: None,
                max_concentration_ratio: None,
                max_symbol_net_exposure_ratio: None,
                max_portfolio_net_exposure_ratio: None,
                max_turnover: None,
                min_trade_weight: None,
                max_new_positions_per_rebalance: None,
                max_total_leverage: 1.0,
                max_exchange_leverage: 1.0,
                min_action_interval_ms: 0,
                enabled: true,
            }],
            initial_cash_balance: 100_000.0,
            taker_fee_bps: 5.0,
            default_slippage_bps: 3.0,
            total_cost_buffer_bps: 10.0,
        },
    );

    // 策略B: 保守型,半仓,高费率
    save_backtest(
        "sc_b_conservative",
        &RuntimeProtocolCoreConfig {
            data_sources: vec![DataSourceConfig {
                data_id: "d1".into(),
                exchange: Exchange::Binance,
                symbol: Symbol::BtcUsdt,
                market_type: MarketType::Spot,
                kind: DataKind::KlineSeries,
                days: Some(300),
                interval: Some("1d".into()),
                ping_enabled: false,
                request_interval_ms: None,
                enabled: true,
            }],
            intents: vec![
                IntentConfig {
                    intent_id: "buy".into(),
                    name: "Buy".into(),
                    kind: IntentKind::LongTermBuy,
                    input_data_ids: vec!["d1".into()],
                    params: BTreeMap::new(),
                    enabled: true,
                },
                IntentConfig {
                    intent_id: "sell".into(),
                    name: "Sell".into(),
                    kind: IntentKind::LongTermSell,
                    input_data_ids: vec!["d1".into()],
                    params: BTreeMap::new(),
                    enabled: true,
                },
            ],
            agents: vec![AgentConfig {
                agent_id: "a1".into(),
                name: "Agent".into(),
                input_intent_ids: vec!["buy".into(), "sell".into()],
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
                risk_id: "r1".into(),
                name: "Risk".into(),
                observed_agent_ids: vec!["a1".into()],
                max_position_ratio: 0.3,
                max_single_weight: None,
                max_concentration_ratio: None,
                max_symbol_net_exposure_ratio: None,
                max_portfolio_net_exposure_ratio: None,
                max_turnover: None,
                min_trade_weight: None,
                max_new_positions_per_rebalance: None,
                max_total_leverage: 1.0,
                max_exchange_leverage: 1.0,
                min_action_interval_ms: 500,
                enabled: true,
            }],
            initial_cash_balance: 100_000.0,
            taker_fee_bps: 15.0,
            default_slippage_bps: 8.0,
            total_cost_buffer_bps: 25.0,
        },
    );

    qrpc_runtime::set_mock_volatility(0.0);
    println!("Done.");
}

#[cfg(not(feature = "dev_tools"))]
fn main() {
    eprintln!("gen_screenshots: 此工具需启用 dev_tools feature (cargo run --features dev_tools --bin gen_screenshots)");
}
