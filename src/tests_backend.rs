use super::*;
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

fn test_storage_base(label: &str) -> PathBuf {
    static TEST_STORAGE_COUNTER: std::sync::atomic::AtomicU64 =
        std::sync::atomic::AtomicU64::new(0);
    let sequence = TEST_STORAGE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "quantpilot-test-{}-{}-{}-{}",
        label,
        std::process::id(),
        current_time_ms(),
        sequence
    ))
}

fn test_app_state() -> AppState {
    let base = test_storage_base("api");
    let graph_dir = base.join("graphs");
    let run_dir = base.join("runs");
    let backtest_dir = base.join("backtests");
    std::fs::create_dir_all(&graph_dir).unwrap();
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&backtest_dir).unwrap();
    test_app_state_from_dirs(base, graph_dir, run_dir, backtest_dir)
}

fn test_app_state_from_dirs(
    base: PathBuf,
    graph_dir: PathBuf,
    run_dir: PathBuf,
    backtest_dir: PathBuf,
) -> AppState {
    let mut state = new_app_state(graph_dir, run_dir, backtest_dir);
    state.test_storage_root = Some(Arc::new(TestStorageRoot { path: base }));
    state
}

#[test]
fn defaults_to_server_when_no_cli_args_are_provided() {
    let command = parse_cli_command_from(["quantpilot"] as [&str; 1]).unwrap();
    assert_eq!(command, CliCommand::Serve);
}

#[test]
fn parses_strategy_ir_validate_command() {
    let command = parse_cli_command_from([
        "quantpilot",
        "strategy-ir",
        "validate",
        "config/strategy_ir.v0.example.json",
    ])
    .unwrap();
    assert_eq!(
        command,
        CliCommand::StrategyIrValidate {
            path: PathBuf::from("config/strategy_ir.v0.example.json"),
        }
    );
}

#[test]
fn parses_v4_run_command() {
    let command = parse_cli_command_from(["quantpilot", "v4-run", "demo_v4"]).unwrap();
    assert_eq!(
        command,
        CliCommand::V4Run {
            graph_id_or_path: "demo_v4".to_string(),
        }
    );
}

#[test]
fn rejects_unknown_cli_command() {
    let err = parse_cli_command_from(["quantpilot", "unknown"]).unwrap_err();
    assert!(err.to_string().contains("不支持的命令"));
}

#[test]
fn parses_strategy_ir_json_with_utf8_bom() {
    let source = concat!(
            "\u{feff}",
            "{",
            "\"ir_version\":\"strategy_ir/v0\",",
            "\"metadata\":{",
            "\"strategy_id\":\"demo\",",
            "\"name\":\"Demo\",",
            "\"summary\":\"Demo strategy\",",
            "\"source\":{\"source_type\":\"manual_paper_analysis\",\"paper_title\":\"Demo\",\"paper_reference\":null}",
            "},",
            "\"signals\":[{\"signal_id\":\"s1\",\"name\":\"Signal\",\"indicator\":{\"kind\":\"rsi\",\"inputs\":[\"close\"],\"params\":{}}}],",
            "\"logic\":{\"entry_rules\":[{\"rule_id\":\"r1\",\"condition\":\"close > open\",\"action\":\"open_long\"}],\"exit_rules\":[],\"position_sizing\":{\"method\":\"fixed_ratio\",\"value\":0.1,\"unit\":\"portfolio_ratio\"},\"rebalance_rule\":null},",
            "\"risk_rules\":{\"max_position_ratio\":0.2,\"stop_loss_ratio\":0.02,\"take_profit_ratio\":null,\"max_drawdown_ratio\":null,\"max_trades_per_day\":null,\"notes\":[]},",
            "\"data_requirements\":[{\"data_id\":\"d1\",\"venue\":\"binance\",\"symbol\":\"BTCUSDT\",\"data_type\":\"kline\",\"granularity\":\"1d\",\"lookback\":100,\"fields\":[\"close\"]}],",
            "\"execution\":{\"venue_type\":\"paper\",\"order_type\":\"market\",\"time_in_force\":null,\"slippage_model\":\"fixed_bps\",\"latency_assumption_ms\":null,\"capital_base\":null},",
            "\"gap_annotations\":[],",
            "\"unknowns\":[]",
            "}"
        );
    let strategy_ir = parse_strategy_ir_json(source).unwrap();
    assert_eq!(strategy_ir.metadata.strategy_id, "demo");
}

#[test]
fn capability_response_distinguishes_supported_and_declared_only_indicator_kinds() {
    let response = build_capability_response();

    assert_eq!(response.api_version, CAPABILITY_API_VERSION);
    assert_eq!(response.schema_version, CAPABILITY_SCHEMA_VERSION);
    assert_eq!(response.chain_stages, RUNTIME_CHAIN_STAGES.to_vec());
    assert!(response.schema_hash.starts_with("sha256:"));
    assert_eq!(
        response.strategy_ir.declared_indicator_kinds,
        declared_indicator_kinds().to_vec()
    );
    assert_eq!(
        response.strategy_ir.supported_indicator_kinds,
        supported_indicator_kinds().to_vec()
    );

    let spread = response
        .strategy_ir
        .indicator_support
        .iter()
        .find(|entry| entry.kind == IndicatorKind::Spread)
        .unwrap();
    assert_eq!(spread.status, CapabilitySupportStatus::Supported);
    assert_eq!(spread.reason, None);

    let custom = response
        .strategy_ir
        .indicator_support
        .iter()
        .find(|entry| entry.kind == IndicatorKind::Custom)
        .unwrap();
    assert_eq!(custom.status, CapabilitySupportStatus::Supported);
    assert_eq!(custom.reason, None);

    let ma_cross = response
        .strategy_ir
        .indicator_support
        .iter()
        .find(|entry| entry.kind == IndicatorKind::MaCross)
        .unwrap();
    assert_eq!(ma_cross.status, CapabilitySupportStatus::Supported);
    assert_eq!(ma_cross.reason, None);
}

#[test]
fn compare_execution_assumptions_modules_reports_missing_when_either_side_absent() {
    let left = Some(ExecutionAssumptionsModule {
        summary: super::backtest_artifacts::ExecutionAssumptionsSummary {
            fee_bps: 10.0,
            slippage_bps: 5.0,
            latency_ms: 0,
            sources: None,
        },
        list_tag: backtest_artifacts::ExecutionAssumptionsTag {
            label: "fee=10 slip=5 lat=0".to_string(),
            sources_label: "fee:na slip:na lat:na".to_string(),
        },
    });

    let compared = compare_execution_assumptions_modules(left, None);
    assert_eq!(compared.status, BacktestCompareStatus::Missing);
    assert!(compared.left.is_some());
    assert!(compared.right.is_none());
    assert_eq!(
        compared.fields,
        BacktestExecutionAssumptionsFieldDiffs {
            fee_bps: BacktestExecutionAssumptionsFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            slippage_bps: BacktestExecutionAssumptionsFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            latency_ms: BacktestExecutionAssumptionsFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            sources: BacktestExecutionAssumptionsFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
        }
    );

    let metrics = compare_metrics_summaries(
        Some(qrpc_core::BacktestSummary {
            step_count: 10,
            trade_count: 2,
            total_return_ratio: 0.1,
            final_equity: 110.0,
            net_profit: 10.0,
            win_rate: 0.0,
            annualized_return: 0.0,
            annualized_volatility: 0.0,
            risk_adjusted: Default::default(),
            trade_analysis: Default::default(),
            drawdown_analysis: qrpc_core::BacktestDrawdownAnalysis {
                max_drawdown_ratio: 0.02,
                ..Default::default()
            },
            benchmark_comparison: None,
            skewness: 0.0,
            kurtosis: 0.0,
        }),
        None,
    );
    assert_eq!(metrics.status, BacktestCompareStatus::Missing);
    assert_eq!(
        metrics.fields,
        BacktestMetricsFieldDiffs {
            step_count: BacktestMetricsFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            trade_count: BacktestMetricsFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            total_return_ratio: BacktestMetricsFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            max_drawdown_ratio: BacktestMetricsFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            final_equity: BacktestMetricsFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            net_profit: BacktestMetricsFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
        }
    );

    let trade_ledger = compare_trade_ledger_summaries(
        Some(backtest_artifacts::TradeLedgerSummary {
            trade_count: 2,
            buy_fill_count: 0,
            sell_fill_count: 0,
            total_fees_paid: 3.0,
            buy_fees_paid: 0.0,
            sell_fees_paid: 0.0,
            total_filled_notional: 1000.0,
            buy_filled_notional: 0.0,
            sell_filled_notional: 0.0,
            average_fill_price: 0.0,
            average_buy_fill_price: None,
            average_sell_fill_price: None,
            average_fee_per_fill: 1.5,
            average_buy_fee: None,
            average_sell_fee: None,
        }),
        None,
    );
    assert_eq!(trade_ledger.status, BacktestCompareStatus::Missing);
    assert_eq!(
        trade_ledger.fields,
        BacktestTradeLedgerFieldDiffs {
            trade_count: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            buy_fill_count: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            sell_fill_count: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            total_fees_paid: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            buy_fees_paid: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            sell_fees_paid: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            total_filled_notional: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            buy_filled_notional: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            sell_filled_notional: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            average_fill_price: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            average_buy_fill_price: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            average_sell_fill_price: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            average_fee_per_fill: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            average_buy_fee: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            average_sell_fee: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
        }
    );
    let equity_curve = compare_equity_curve_points(
        Some(vec![qrpc_core::BacktestEquityPoint {
            ts_ms: 1_700_000_000_000,
            equity: 110.0,
            cash_balance: 90.0,
            net_notional: 20.0,
        }]),
        None,
    );

    let report_bundle =
        build_compare_report_bundle(&compared, &metrics, &trade_ledger, &equity_curve);
    let narrative = build_report_narrative_compare_block(
        &compared,
        &metrics,
        &trade_ledger,
        &equity_curve,
        &report_bundle,
    );
    assert_eq!(narrative.status, BacktestCompareStatus::Missing);
    assert!(narrative.headline.contains("cannot be fully compared"));
    assert_eq!(
        narrative.bullets,
        vec![
            "Execution assumptions: missing.".to_string(),
            "Metrics summary: missing.".to_string(),
            "Trade ledger summary: missing.".to_string(),
            "Equity curve: missing.".to_string(),
        ]
    );
    assert_eq!(
        narrative.highlights,
        vec![
            "Execution assumptions are unavailable on one or both runs.".to_string(),
            "Metrics summary is unavailable on one or both runs.".to_string(),
            "Trade ledger summary is unavailable on one or both runs.".to_string(),
            "Equity curve is unavailable on one or both runs.".to_string(),
        ]
    );
    assert_eq!(
        narrative.source_explanations,
        vec![
            "Fee source is unavailable on one or both runs.".to_string(),
            "Slippage source is unavailable on one or both runs.".to_string(),
            "Latency source is unavailable on one or both runs.".to_string(),
        ]
    );
    assert_eq!(
            narrative.sections,
            vec![
                BacktestReportNarrativeSection {
                    title: "Execution assumptions".to_string(),
                    status: BacktestCompareStatus::Missing,
                    summary: "Execution assumptions are unavailable on one or both runs."
                        .to_string(),
                    lines: vec![
                        "Status: missing.".to_string(),
                        "Left resolved values: fee_bps=10, slippage_bps=5, latency_ms=0."
                            .to_string(),
                        "Right resolved values: missing.".to_string(),
                    ],
                },
                BacktestReportNarrativeSection {
                    title: "Metrics summary".to_string(),
                    status: BacktestCompareStatus::Missing,
                    summary: "Metrics summary is unavailable on one or both runs."
                        .to_string(),
                    lines: vec![
                        "Status: missing.".to_string(),
                        "Performance drilldown: missing.".to_string(),
                        "Activity drilldown: missing.".to_string(),
                        "Cost drilldown: same.".to_string(),
                        "Left summary: steps=10, trades=2, return=0.1, drawdown=0.02, final_equity=110, net_profit=10, sharpe=0, profit_factor=0.".to_string(),
                        "Right summary: missing.".to_string(),
                    ],
                },
                BacktestReportNarrativeSection {
                    title: "Trade ledger summary".to_string(),
                    status: BacktestCompareStatus::Missing,
                    summary: "Trade ledger summary is unavailable on one or both runs."
                        .to_string(),
                    lines: vec![
                        "Status: missing.".to_string(),
                        "Left summary: trade_count=2, buy_fill_count=0, sell_fill_count=0, total_fees_paid=3, buy_fees_paid=0, sell_fees_paid=0, total_filled_notional=1000, buy_filled_notional=0, sell_filled_notional=0, average_fill_price=0, average_buy_fill_price=na, average_sell_fill_price=na, average_fee_per_fill=1.5, average_buy_fee=na, average_sell_fee=na.".to_string(),
                        "Right summary: missing.".to_string(),
                    ],
                },
                BacktestReportNarrativeSection {
                    title: "Equity curve".to_string(),
                    status: BacktestCompareStatus::Missing,
                    summary: "Equity curve is unavailable on one or both runs.".to_string(),
                    lines: vec![
                        "Status: missing.".to_string(),
                        "Sample drilldown: missing.".to_string(),
                        "Left summary: point_count=1, started_at_ms=1700000000000, ended_at_ms=1700000000000, first_equity=110, final_equity=110, min_equity=110, max_equity=110.".to_string(),
                        "Right summary: missing.".to_string(),
                    ],
                },
            ]
        );
    let compare_report =
        build_compare_report_view(&metrics, &equity_curve, &narrative, &report_bundle);
    assert_eq!(compare_report.status, BacktestCompareStatus::Missing);
    assert_eq!(
        compare_report.overview,
        BacktestCompareReportOverview {
            bullets: vec![
                "Execution assumptions: missing.".to_string(),
                "Metrics summary: missing.".to_string(),
                "Trade ledger summary: missing.".to_string(),
                "Equity curve: missing.".to_string(),
            ],
            highlights: vec![
                "Execution assumptions are unavailable on one or both runs.".to_string(),
                "Metrics summary is unavailable on one or both runs.".to_string(),
                "Trade ledger summary is unavailable on one or both runs.".to_string(),
                "Equity curve is unavailable on one or both runs.".to_string(),
            ],
        }
    );
    assert_eq!(
        compare_report
            .modules
            .execution_assumptions
            .source_explanations,
        vec![
            "Fee source is unavailable on one or both runs.".to_string(),
            "Slippage source is unavailable on one or both runs.".to_string(),
            "Latency source is unavailable on one or both runs.".to_string(),
        ]
    );
    assert_eq!(
        compare_report.modules.metrics.drilldown.performance.status,
        BacktestCompareStatus::Missing
    );
    assert_eq!(
        compare_report.modules.equity_curve.status,
        BacktestCompareStatus::Missing
    );
}

#[test]
fn compare_metrics_summaries_reports_field_level_differences() {
    let compared = compare_metrics_summaries(
        Some(qrpc_core::BacktestSummary {
            step_count: 10,
            trade_count: 2,
            total_return_ratio: 0.1,
            final_equity: 110.0,
            net_profit: 10.0,
            win_rate: 0.0,
            annualized_return: 0.0,
            annualized_volatility: 0.0,
            risk_adjusted: Default::default(),
            trade_analysis: Default::default(),
            drawdown_analysis: qrpc_core::BacktestDrawdownAnalysis {
                max_drawdown_ratio: 0.02,
                ..Default::default()
            },
            benchmark_comparison: None,
            skewness: 0.0,
            kurtosis: 0.0,
        }),
        Some(qrpc_core::BacktestSummary {
            step_count: 10,
            trade_count: 3,
            total_return_ratio: 0.08,
            final_equity: 108.0,
            net_profit: 8.0,
            win_rate: 0.0,
            annualized_return: 0.0,
            annualized_volatility: 0.0,
            risk_adjusted: Default::default(),
            trade_analysis: Default::default(),
            drawdown_analysis: qrpc_core::BacktestDrawdownAnalysis {
                max_drawdown_ratio: 0.02,
                ..Default::default()
            },
            benchmark_comparison: None,
            skewness: 0.0,
            kurtosis: 0.0,
        }),
    );

    assert_eq!(compared.status, BacktestCompareStatus::Different);
    assert_eq!(
        compared.fields,
        BacktestMetricsFieldDiffs {
            step_count: BacktestMetricsFieldDiff {
                status: BacktestCompareStatus::Same,
            },
            trade_count: BacktestMetricsFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            total_return_ratio: BacktestMetricsFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            max_drawdown_ratio: BacktestMetricsFieldDiff {
                status: BacktestCompareStatus::Same,
            },
            final_equity: BacktestMetricsFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            net_profit: BacktestMetricsFieldDiff {
                status: BacktestCompareStatus::Different,
            },
        }
    );
    assert_eq!(
        compared.drilldown,
        BacktestMetricsDrilldownCompare {
            performance: BacktestMetricsDrilldownGroupCompare {
                status: BacktestCompareStatus::Different,
                fields: vec![
                    BacktestMetricsDrilldownFieldCompare {
                        key: "total_return_ratio".to_string(),
                        status: BacktestCompareStatus::Different,
                        left_value: Some("0.1".to_string()),
                        right_value: Some("0.08".to_string()),
                    },
                    BacktestMetricsDrilldownFieldCompare {
                        key: "net_profit".to_string(),
                        status: BacktestCompareStatus::Different,
                        left_value: Some("10".to_string()),
                        right_value: Some("8".to_string()),
                    },
                    BacktestMetricsDrilldownFieldCompare {
                        key: "final_equity".to_string(),
                        status: BacktestCompareStatus::Different,
                        left_value: Some("110".to_string()),
                        right_value: Some("108".to_string()),
                    },
                    BacktestMetricsDrilldownFieldCompare {
                        key: "max_drawdown_ratio".to_string(),
                        status: BacktestCompareStatus::Same,
                        left_value: Some("0.02".to_string()),
                        right_value: Some("0.02".to_string()),
                    },
                ],
            },
            activity: BacktestMetricsDrilldownGroupCompare {
                status: BacktestCompareStatus::Different,
                fields: vec![
                    BacktestMetricsDrilldownFieldCompare {
                        key: "step_count".to_string(),
                        status: BacktestCompareStatus::Same,
                        left_value: Some("10".to_string()),
                        right_value: Some("10".to_string()),
                    },
                    BacktestMetricsDrilldownFieldCompare {
                        key: "trade_count".to_string(),
                        status: BacktestCompareStatus::Different,
                        left_value: Some("2".to_string()),
                        right_value: Some("3".to_string()),
                    },
                ],
            },
            costs: BacktestMetricsDrilldownGroupCompare {
                status: BacktestCompareStatus::Same,
                fields: vec![],
            },
        }
    );
}

#[test]
fn compare_trade_ledger_summaries_reports_field_level_differences() {
    let compared = compare_trade_ledger_summaries(
        Some(backtest_artifacts::TradeLedgerSummary {
            trade_count: 2,
            buy_fill_count: 1,
            sell_fill_count: 1,
            total_fees_paid: 3.0,
            buy_fees_paid: 1.0,
            sell_fees_paid: 2.0,
            total_filled_notional: 1000.0,
            buy_filled_notional: 450.0,
            sell_filled_notional: 550.0,
            average_fill_price: 100.0,
            average_buy_fill_price: Some(95.0),
            average_sell_fill_price: Some(105.0),
            average_fee_per_fill: 1.5,
            average_buy_fee: Some(1.0),
            average_sell_fee: Some(2.0),
        }),
        Some(backtest_artifacts::TradeLedgerSummary {
            trade_count: 2,
            buy_fill_count: 2,
            sell_fill_count: 0,
            total_fees_paid: 4.0,
            buy_fees_paid: 4.0,
            sell_fees_paid: 0.0,
            total_filled_notional: 1250.0,
            buy_filled_notional: 1250.0,
            sell_filled_notional: 0.0,
            average_fill_price: 125.0,
            average_buy_fill_price: Some(125.0),
            average_sell_fill_price: None,
            average_fee_per_fill: 2.0,
            average_buy_fee: Some(2.0),
            average_sell_fee: None,
        }),
    );

    assert_eq!(compared.status, BacktestCompareStatus::Different);
    assert_eq!(
        compared.fields,
        BacktestTradeLedgerFieldDiffs {
            trade_count: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Same,
            },
            buy_fill_count: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            sell_fill_count: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            total_fees_paid: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            buy_fees_paid: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            sell_fees_paid: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            total_filled_notional: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            buy_filled_notional: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            sell_filled_notional: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            average_fill_price: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            average_buy_fill_price: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            average_sell_fill_price: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
            average_fee_per_fill: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            average_buy_fee: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Different,
            },
            average_sell_fee: BacktestTradeLedgerFieldDiff {
                status: BacktestCompareStatus::Missing,
            },
        }
    );
}

#[test]
fn capability_response_keeps_legacy_frontend_fields_and_adds_module_support_entries() {
    let response = build_capability_response();

    assert_eq!(
        response.frontend.declared_module_keys,
        DECLARED_FRONTEND_MODULE_KEYS.to_vec()
    );
    assert_eq!(
        response.frontend.supported_module_keys,
        SUPPORTED_FRONTEND_MODULE_KEYS.to_vec()
    );
    assert!(response.frontend.unsupported_module_reasons.is_empty());

    let arbitrage = response
        .frontend
        .module_support
        .iter()
        .find(|entry| entry.module_key == "builtin.agent.arbitrage")
        .unwrap();
    assert_eq!(arbitrage.status, CapabilitySupportStatus::Supported);
    assert_eq!(arbitrage.reason, None);

    let weighted = response
        .frontend
        .module_support
        .iter()
        .find(|entry| entry.module_key == "builtin.agent.weighted")
        .unwrap();
    assert_eq!(weighted.status, CapabilitySupportStatus::Supported);
    assert_eq!(weighted.reason, None);
}

#[test]
fn capability_response_declares_workspace_surfaces_and_ui_actions() {
    let response = build_capability_response();

    let workspace_surface_keys = response
        .workspace
        .surfaces
        .iter()
        .map(|entry| entry.key)
        .collect::<Vec<_>>();
    assert_eq!(
        workspace_surface_keys,
        vec![
            "dashboard",
            "code",
            "diagnostics",
            "research",
            "monitor",
            "source",
            "template_library",
            "version_history",
            "collaboration_audit",
            "parameter_sweep",
        ]
    );
    assert!(response
        .workspace
        .surfaces
        .iter()
        .all(|entry| entry.status == CapabilitySupportStatus::Supported
            && entry.source == "backend:/api/capabilities.workspace.surfaces"));

    let ui_action_keys = response
        .ui_actions
        .actions
        .iter()
        .map(|entry| entry.key)
        .collect::<Vec<_>>();
    assert_eq!(
        ui_action_keys,
        vec![
            "open_tutorial",
            "manage_credentials",
            "reset_graph",
            "load_latest_graph",
            "save_graph",
            "export_runtime_config",
            "export_quantscript",
            "compile",
            "start_simulation",
            "start_v4_simulation",
            "run_backtest",
            "stop_runtime",
            "reset_runtime",
            "open_backtests",
            "run_parameter_sweep",
        ]
    );
    assert!(response
        .ui_actions
        .actions
        .iter()
        .all(|entry| entry.status == CapabilitySupportStatus::Supported
            && entry.source == "backend:/api/capabilities.ui_actions.actions"));
}

#[test]
fn capability_response_serializes_new_support_sections() {
    let value = serde_json::to_value(build_capability_response()).unwrap();

    assert_eq!(value["api_version"], CAPABILITY_API_VERSION);
    assert!(value["strategy_ir"]["indicator_support"].is_array());
    assert!(value["runtime"]["mode_support"].is_array());
    assert!(value["market_data"]["exchange_support"].is_array());
    assert!(value["frontend"]["module_support"].is_array());
    assert!(value["frontend"]["supported_module_keys"].is_array());
    assert!(value["workspace"]["surfaces"].is_array());
    assert!(value["ui_actions"]["actions"].is_array());
    assert_eq!(
        value["workspace"]["surfaces"][0]["source"],
        "backend:/api/capabilities.workspace.surfaces"
    );
    assert_eq!(
        value["ui_actions"]["actions"][0]["source"],
        "backend:/api/capabilities.ui_actions.actions"
    );
    assert_eq!(
        value["chain_stages"],
        serde_json::json!(RUNTIME_CHAIN_STAGES)
    );
    assert_eq!(
        value["permission_boundary"]["non_execution_order_access"],
        "deny"
    );
    assert_eq!(
        value["versioning"]["parameter_version_policy"],
        "immutable_generation_pointer"
    );
}

#[test]
fn capability_contract_drives_response_hash_and_runtime_governance() {
    let response = build_capability_response();
    let governance = runtime_governance_snapshot(
        &FrontendMetadata {
            graph_id: "graph_hash_contract".to_string(),
            compile_id: "compile_hash_contract".to_string(),
            name: "Hash Contract".to_string(),
            version: "1.2.3".to_string(),
            mode: "paper".to_string(),
        },
        Some("params_hash_contract"),
    );

    assert_eq!(response.schema_hash, current_capability_hash());
    assert_eq!(response.schema_hash, governance.capability_hash);
    assert!(governance.deployment_revision.starts_with("sha256:"));
}

#[test]
fn capability_contract_hash_changes_when_governed_fields_change() {
    let base = build_capability_contract();
    let base_hash = capability_contract_hash(&base);

    let mut changed_stage = base.clone();
    changed_stage.chain_stages.push("settlement");
    assert_ne!(capability_contract_hash(&changed_stage), base_hash);

    let mut changed_runtime_mode = base.clone();
    changed_runtime_mode.runtime_modes.push("live");
    assert_ne!(capability_contract_hash(&changed_runtime_mode), base_hash);

    let mut changed_module = base.clone();
    changed_module
        .supported_module_keys
        .push("builtin.intent.contract_test");
    assert_ne!(capability_contract_hash(&changed_module), base_hash);

    let mut changed_workspace_surface = base.clone();
    changed_workspace_surface
        .workspace_surfaces
        .push(ui_capability_entry(
            "contract_test_surface",
            "backend:/api/capabilities.workspace.surfaces",
        ));
    assert_ne!(
        capability_contract_hash(&changed_workspace_surface),
        base_hash
    );

    let mut changed_symbol = base.clone();
    changed_symbol.supported_symbols.push("DOGEUSDT");
    assert_ne!(capability_contract_hash(&changed_symbol), base_hash);

    let mut changed_policy = base;
    changed_policy.permission_boundary = CapabilityPermissionBoundarySummary {
        ai_write_policy: AiWritePolicy::Disabled,
        ..changed_policy.permission_boundary
    };
    assert_ne!(capability_contract_hash(&changed_policy), base_hash);
}

#[test]
fn capability_contract_hash_is_canonical_and_order_stable() {
    let mut left = build_capability_contract();
    left.unsupported_module_reasons = BTreeMap::from([
        ("builtin.intent.contract_b", "beta reason"),
        ("builtin.intent.contract_a", "alpha reason"),
    ]);

    let mut right = build_capability_contract();
    right.unsupported_module_reasons = BTreeMap::from([
        ("builtin.intent.contract_a", "alpha reason"),
        ("builtin.intent.contract_b", "beta reason"),
    ]);

    let left_hash = capability_contract_hash(&left);
    assert!(left_hash.starts_with("sha256:"));
    assert_eq!(left_hash, capability_contract_hash(&right));
}

#[tokio::test]
async fn capabilities_endpoint_returns_capability_response_over_router() {
    let graph_app = build_app_router(test_app_state());

    let response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/capabilities")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["api_version"], CAPABILITY_API_VERSION);
    assert!(value["schema_hash"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert_eq!(
        value["permission_boundary"]["ai_write_policy"],
        serde_json::json!("proposal_only")
    );
    assert!(value["strategy_ir"]["indicator_support"].is_array());
    assert!(value["frontend"]["module_support"].is_array());
    assert_eq!(
        value["market_data"]["supported_symbols"],
        serde_json::json!(["BTCUSDT", "ETHUSDT", "SOLUSDT"])
    );
    assert_eq!(
        value["frontend"]["unsupported_module_reasons"],
        serde_json::json!({})
    );
}

#[tokio::test]
async fn unknown_api_route_returns_json_404_instead_of_spa_fallback() {
    let graph_app = build_app_router(test_app_state());

    let response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/not-a-real-route")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["error"], "not_found");
}

#[test]
fn capability_fixture_matches_backend_response_snapshot() {
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../frontend/src/test/fixtures/capabilities/backend-capabilities-v1.json"
    ))
    .unwrap();
    let actual = serde_json::to_value(build_capability_response()).unwrap();

    assert_eq!(actual, expected);
}

/// v2.5.0: 此测试用于导出能力合约 fixture JSON 快照。
/// 仅在手动更新 backend-capabilities-v1.json 时运行, CI 中忽略。
#[test]
#[ignore]
fn export_capability_fixture_snapshot() {
    let json = serde_json::to_vec_pretty(&build_capability_response()).unwrap();
    let encoded = json
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();

    println!("__CAPABILITY_FIXTURE_START__");
    println!("{}", encoded);
    println!("__CAPABILITY_FIXTURE_END__");
}

fn sample_compile_request_json() -> serde_json::Value {
    serde_json::json!({
        "capability_context": serde_json::to_value(current_capability_context()).unwrap(),
        "runtime_config": {
            "metadata": {
                "graph_id": "graph_test",
                "compile_id": "compile_test",
                "name": "Test Graph",
                "version": "1.0.0",
                "mode": "paper"
            },
            "data_sources": [
                {
                    "id": "data_data_1",
                    "module_key": "builtin.data.kline",
                    "name": "Data",
                    "config": {
                        "exchange": "binance",
                        "instrument": "BTCUSDT",
                        "timeframe": "1d",
                        "window_size": 200
                    }
                }
            ],
            "intent_generators": [
                {
                    "id": "intent_intent_1",
                    "module_key": "builtin.intent.double_ma",
                    "name": "Intent",
                    "config": {
                        "fast_period": 20,
                        "slow_period": 50,
                        "entry_ratio": 0.2
                    },
                    "input_refs": [
                        {
                            "source_id": "data_data_1",
                            "source_port": "market_data_out",
                            "target_port": "data_input"
                        }
                    ]
                }
            ],
            "agents": [
                {
                    "id": "agent_agent_1",
                    "module_key": "builtin.agent.weighted",
                    "name": "Agent",
                    "config": {
                        "decision_threshold": 0.05,
                        "max_quantity_ratio": 0.2
                    },
                    "intent_refs": ["intent_intent_1"]
                }
            ],
            "risk_controls": [
                {
                    "id": "risk_risk_1",
                    "module_key": "builtin.risk.global",
                    "name": "Risk",
                    "config": {
                        "profile_id": "global",
                        "max_position": 0.2,
                        "max_total_leverage": 3.0,
                        "max_exchange_leverage": 3.0,
                        "min_action_interval_ms": 100
                    },
                    "agent_refs": ["agent_agent_1"]
                }
            ],
            "executions": [
                {
                    "id": "execution_execution_1",
                    "module_key": "builtin.execution.paper",
                    "name": "Execution",
                    "config": {
                        "profile_id": "paper",
                        "mode": "paper",
                        "slippage_bps": 5
                    },
                    "risk_ref": "risk_risk_1"
                }
            ],
            "runtime_control": {
                "id": "runtime_runtime_1",
                "module_key": "builtin.runtime.control",
                "name": "Runtime",
                "config": {
                    "mode": "paper"
                }
            }
        },
        "graph_json": {
            "metadata": { "graph_id": "graph_test", "name": "Test Graph", "version": "1.0.0" },
            "nodes": [
                { "id": "data_data_1", "type": "data", "module_key": "builtin.data.kline", "name": "Data", "config": { "exchange": "binance", "instrument": "BTCUSDT", "timeframe": "1d", "window_size": 200 } },
                { "id": "intent_intent_1", "type": "intent", "module_key": "builtin.intent.double_ma", "name": "Intent", "config": { "fast_period": 20, "slow_period": 50, "entry_ratio": 0.2 } },
                { "id": "agent_agent_1", "type": "agent", "module_key": "builtin.agent.weighted", "name": "Agent", "config": { "decision_threshold": 0.05, "max_quantity_ratio": 0.2 } },
                { "id": "risk_risk_1", "type": "risk", "module_key": "builtin.risk.global", "name": "Risk", "config": { "profile_id": "global", "max_position": 0.2, "max_total_leverage": 3.0, "max_exchange_leverage": 3.0, "min_action_interval_ms": 100 } },
                { "id": "execution_execution_1", "type": "execution", "module_key": "builtin.execution.paper", "name": "Execution", "config": { "profile_id": "paper", "mode": "paper", "slippage_bps": 5 } },
                { "id": "runtime_runtime_1", "type": "runtime", "module_key": "builtin.runtime.control", "name": "Runtime", "config": { "mode": "paper" } }
            ],
            "edges": [
                { "source_node_id": "data_data_1", "source_port": "market_data_out", "target_node_id": "intent_intent_1", "target_port": "data_input" },
                { "source_node_id": "intent_intent_1", "source_port": "intent_out", "target_node_id": "agent_agent_1", "target_port": "intent_input" },
                { "source_node_id": "agent_agent_1", "source_port": "agent_out", "target_node_id": "risk_risk_1", "target_port": "agent_input" },
                { "source_node_id": "risk_risk_1", "source_port": "risk_out", "target_node_id": "execution_execution_1", "target_port": "risk_input" }
            ]
        }
    })
}

fn sample_strategy_ir_compile_request_json() -> serde_json::Value {
    serde_json::json!({
        "graph_id": "strategy_graph_test",
        "compile_id": "strategy_compile_test",
        "strategy_ir": {
            "ir_version": "strategy_ir/v0",
            "metadata": {
                "strategy_id": "restricted_custom_v1",
                "name": "Restricted Custom",
                "summary": "Custom signal lowered into Core IR.",
                "source": {
                    "source_type": "manual_paper_analysis",
                    "paper_title": "Restricted Custom",
                    "paper_reference": null
                },
                "authors": ["QuantPilot"],
                "tags": ["custom"]
            },
            "signals": [
                {
                    "signal_id": "custom_signal",
                    "name": "Custom Signal",
                    "indicator": {
                        "kind": "custom",
                        "inputs": ["btc_1d"],
                        "params": {
                            "custom_expr": {
                                "schema_version": "quantpilot/custom-expr/v1",
                                "signal_kind": "long",
                                "predicate": {
                                    "left": {
                                        "kind": "window_agg",
                                        "data_id": "btc_1d",
                                        "field": "close",
                                        "window_size": 3,
                                        "agg": "mean"
                                    },
                                    "op": "gt",
                                    "right": {
                                        "kind": "number",
                                        "value": 100.0
                                    }
                                },
                                "strength": {
                                    "kind": "binary",
                                    "left": {
                                        "kind": "input",
                                        "data_id": "btc_1d",
                                        "field": "close"
                                    },
                                    "op": "sub",
                                    "right": {
                                        "kind": "number",
                                        "value": 95.0
                                    }
                                },
                                "confidence": 0.9
                            }
                        }
                    },
                    "transforms": []
                }
            ],
            "logic": {
                "entry_rules": [
                    {
                        "rule_id": "entry_rule",
                        "condition": "custom_signal > 0",
                        "action": "open_long"
                    }
                ],
                "exit_rules": [],
                "position_sizing": {
                    "method": "fixed_ratio",
                    "value": 0.2,
                    "unit": "portfolio_ratio"
                },
                "rebalance_rule": null
            },
            "risk_rules": {
                "max_position_ratio": 0.2,
                "stop_loss_ratio": 0.05,
                "take_profit_ratio": null,
                "max_drawdown_ratio": null,
                "max_trades_per_day": null,
                "notes": []
            },
            "data_requirements": [
                {
                    "data_id": "btc_1d",
                    "venue": "binance",
                    "symbol": "BTCUSDT",
                    "data_type": "kline",
                    "granularity": "1d",
                    "lookback": 200,
                    "fields": ["close"]
                }
            ],
            "execution": {
                "venue_type": "paper",
                "order_type": "market",
                "time_in_force": null,
                "slippage_model": "fixed_bps",
                "latency_assumption_ms": null,
                "capital_base": null
            },
            "gap_annotations": [],
            "unknowns": []
        }
    })
}

fn sample_spread_compile_request_json() -> serde_json::Value {
    serde_json::json!({
        "runtime_config": {
            "metadata": {
                "graph_id": "graph_spread_test",
                "compile_id": "compile_spread_test",
                "name": "Spread Test Graph",
                "version": "1.0.0",
                "mode": "paper"
            },
            "data_sources": [
                {
                    "id": "data_binance_quote",
                    "module_key": "builtin.data.quote",
                    "name": "Binance Quote",
                    "config": {
                        "exchange": "binance",
                        "instrument": "BTCUSDT"
                    }
                },
                {
                    "id": "data_okx_quote",
                    "module_key": "builtin.data.quote",
                    "name": "OKX Quote",
                    "config": {
                        "exchange": "okx",
                        "instrument": "BTCUSDT"
                    }
                }
            ],
            "intent_generators": [
                {
                    "id": "intent_spread_1",
                    "module_key": "builtin.intent.spread_observer",
                    "name": "Spread Observer",
                    "config": {
                        "max_time_diff_ms": 5000,
                        "field_code": 0,
                        "align_direction_code": 0,
                        "resample_period_ms": 60000,
                        "resample_agg_code": 0,
                        "window_size": 3,
                        "window_agg_code": 1,
                        "spread_output_code": 1
                    },
                    "input_refs": [
                        {
                            "source_id": "data_binance_quote",
                            "source_port": "market_data_out",
                            "target_port": "data_input"
                        },
                        {
                            "source_id": "data_okx_quote",
                            "source_port": "market_data_out",
                            "target_port": "data_input"
                        }
                    ]
                }
            ],
            "agents": [
                {
                    "id": "agent_arb_1",
                    "module_key": "builtin.agent.arbitrage",
                    "name": "Arbitrage Agent",
                    "config": {
                        "spread_trigger_bps": 30,
                        "max_quantity_ratio": 0.2
                    },
                    "intent_refs": ["intent_spread_1"]
                }
            ],
            "risk_controls": [
                {
                    "id": "risk_risk_1",
                    "module_key": "builtin.risk.global",
                    "name": "Risk",
                    "config": {
                        "profile_id": "global",
                        "max_position": 0.2,
                        "max_total_leverage": 3.0,
                        "max_exchange_leverage": 3.0,
                        "min_action_interval_ms": 100
                    },
                    "agent_refs": ["agent_arb_1"]
                }
            ],
            "executions": [
                {
                    "id": "execution_execution_1",
                    "module_key": "builtin.execution.paper",
                    "name": "Execution",
                    "config": {
                        "profile_id": "paper",
                        "mode": "paper",
                        "slippage_bps": 5
                    },
                    "risk_ref": "risk_risk_1"
                }
            ],
            "runtime_control": {
                "id": "runtime_runtime_1",
                "module_key": "builtin.runtime.control",
                "name": "Runtime",
                "config": {
                    "mode": "paper"
                }
            }
        }
    })
}

async fn compile_formal_quantscript_for_test(source: &str, compile_id: &str) -> serde_json::Value {
    compile_formal_quantscript_for_test_with_universe_snapshot(source, compile_id, None).await
}

async fn compile_formal_quantscript_for_test_with_universe_snapshot(
    source: &str,
    compile_id: &str,
    universe_snapshot: Option<serde_json::Value>,
) -> serde_json::Value {
    let graph_app = build_app_router(test_app_state());
    let mut payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": compile_id,
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": source,
    });
    if let Some(snapshot) = universe_snapshot {
        payload["universe_snapshot"] = snapshot;
    }

    let response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    serde_json::from_slice(&body).unwrap()
}

fn sample_formal_universe_snapshot_json() -> serde_json::Value {
    serde_json::json!({
        "snapshot_id": "authoring_view_pool_pipeline_snapshot",
        "as_of_ms": 1_710_000_000_000u64,
        "assets": [
            {
                "symbol": "BTCUSDT",
                "exchange": "Binance",
                "market_type": "Spot",
                "quote": "USDT",
                "market_cap": 1_500_000_000_000.0,
                "volume_24h": 40_000_000_000.0,
                "listed_at_ms": 1_500_000_000_000u64,
                "enabled": true
            },
            {
                "symbol": "ETHUSDT",
                "exchange": "Binance",
                "market_type": "Spot",
                "quote": "USDT",
                "market_cap": 500_000_000_000.0,
                "volume_24h": 18_000_000_000.0,
                "listed_at_ms": 1_510_000_000_000u64,
                "enabled": true
            },
            {
                "symbol": "SOLUSDT",
                "exchange": "Binance",
                "market_type": "Spot",
                "quote": "USDT",
                "market_cap": 120_000_000_000.0,
                "volume_24h": 4_000_000_000.0,
                "listed_at_ms": 1_520_000_000_000u64,
                "enabled": true
            }
        ]
    })
}

async fn compile_formal_quantscript_error_for_test(
    source: &str,
    compile_id: &str,
) -> serde_json::Value {
    compile_formal_quantscript_error_for_test_with_universe_snapshot(source, compile_id, None).await
}

async fn compile_formal_quantscript_error_for_test_with_universe_snapshot(
    source: &str,
    compile_id: &str,
    universe_snapshot: Option<serde_json::Value>,
) -> serde_json::Value {
    let graph_app = build_app_router(test_app_state());
    let mut payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": compile_id,
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": source,
    });
    if let Some(snapshot) = universe_snapshot {
        payload["universe_snapshot"] = snapshot;
    }

    let response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "{}",
        String::from_utf8_lossy(&body)
    );
    serde_json::from_slice(&body).unwrap()
}

fn formal_compile_authoring_view(value: &serde_json::Value) -> serde_json::Value {
    value["artifacts"]["strategy"]["metadata"]["quantscript_authoring_view"].clone()
}

fn formal_compile_partial_authoring_view(value: &serde_json::Value) -> serde_json::Value {
    value["partial_artifacts"]["quantscript_authoring_view"].clone()
}

async fn compile_runtime_graph_for_test(
    module_key: &str,
    config: serde_json::Value,
    compile_id: &str,
) -> serde_json::Value {
    let graph_app = build_app_router(test_app_state());
    let mut payload = sample_compile_request_json();
    payload["compile_id"] = serde_json::Value::String(compile_id.to_string());
    payload["runtime_config"]["intent_generators"][0]["module_key"] =
        serde_json::Value::String(module_key.to_string());
    payload["runtime_config"]["intent_generators"][0]["config"] = config;

    let response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    serde_json::from_slice(&body).unwrap()
}

async fn compile_runtime_spread_graph_for_test(
    config: serde_json::Value,
    compile_id: &str,
) -> serde_json::Value {
    let graph_app = build_app_router(test_app_state());
    let mut payload = sample_spread_compile_request_json();
    payload["compile_id"] = serde_json::Value::String(compile_id.to_string());
    payload["runtime_config"]["intent_generators"][0]["config"] = config;

    let response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    serde_json::from_slice(&body).unwrap()
}

async fn compile_runtime_spread_graph_error_for_test(
    config: serde_json::Value,
    compile_id: &str,
) -> serde_json::Value {
    let graph_app = build_app_router(test_app_state());
    let mut payload = sample_spread_compile_request_json();
    payload["compile_id"] = serde_json::Value::String(compile_id.to_string());
    payload["runtime_config"]["intent_generators"][0]["config"] = config;

    let response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "{}",
        String::from_utf8_lossy(&body)
    );
    serde_json::from_slice(&body).unwrap()
}

async fn compile_strategy_ir_for_test(
    signal_id: &str,
    signal_name: &str,
    indicator_kind: &str,
    indicator_params: serde_json::Value,
    condition: &str,
    compile_id: &str,
) -> serde_json::Value {
    let graph_app = build_app_router(test_app_state());
    let mut payload = sample_strategy_ir_compile_request_json();
    payload["compile_id"] = serde_json::Value::String(compile_id.to_string());
    payload["strategy_ir"]["signals"][0]["signal_id"] =
        serde_json::Value::String(signal_id.to_string());
    payload["strategy_ir"]["signals"][0]["name"] =
        serde_json::Value::String(signal_name.to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
        serde_json::Value::String(indicator_kind.to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["params"] = indicator_params;
    payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
        serde_json::Value::String(condition.to_string());
    payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
        serde_json::Value::String("open_long".to_string());

    let response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/strategy-ir/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    serde_json::from_slice(&body).unwrap()
}

async fn compile_strategy_ir_spread_for_test(
    indicator_params: serde_json::Value,
    condition: &str,
    compile_id: &str,
) -> serde_json::Value {
    let graph_app = build_app_router(test_app_state());
    let mut payload = sample_strategy_ir_compile_request_json();
    payload["compile_id"] = serde_json::Value::String(compile_id.to_string());
    payload["strategy_ir"]["signals"][0]["signal_id"] =
        serde_json::Value::String("spread_signal".to_string());
    payload["strategy_ir"]["signals"][0]["name"] =
        serde_json::Value::String("Spread Signal".to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
        serde_json::Value::String("spread".to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["inputs"] =
        serde_json::json!(["binance_btc_quote", "okx_btc_quote"]);
    payload["strategy_ir"]["signals"][0]["indicator"]["params"] = indicator_params;
    payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
        serde_json::Value::String(condition.to_string());
    payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
        serde_json::Value::String("open_long".to_string());
    payload["strategy_ir"]["data_requirements"] = serde_json::json!([
        {
            "data_id": "binance_btc_quote",
            "venue": "binance",
            "symbol": "BTCUSDT",
            "data_type": "quote",
            "granularity": "1m",
            "lookback": 200,
            "fields": ["bid", "ask", "mid"]
        },
        {
            "data_id": "okx_btc_quote",
            "venue": "okx",
            "symbol": "BTCUSDT",
            "data_type": "quote",
            "granularity": "1m",
            "lookback": 200,
            "fields": ["bid", "ask", "mid"]
        }
    ]);

    let response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/strategy-ir/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    serde_json::from_slice(&body).unwrap()
}

async fn compile_strategy_ir_spread_error_for_test(
    indicator_params: serde_json::Value,
    condition: &str,
    compile_id: &str,
) -> serde_json::Value {
    let graph_app = build_app_router(test_app_state());
    let mut payload = sample_strategy_ir_compile_request_json();
    payload["compile_id"] = serde_json::Value::String(compile_id.to_string());
    payload["strategy_ir"]["signals"][0]["signal_id"] =
        serde_json::Value::String("spread_signal".to_string());
    payload["strategy_ir"]["signals"][0]["name"] =
        serde_json::Value::String("Spread Signal".to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
        serde_json::Value::String("spread".to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["inputs"] =
        serde_json::json!(["binance_btc_quote", "okx_btc_quote"]);
    payload["strategy_ir"]["signals"][0]["indicator"]["params"] = indicator_params;
    payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
        serde_json::Value::String(condition.to_string());
    payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
        serde_json::Value::String("open_long".to_string());
    payload["strategy_ir"]["data_requirements"] = serde_json::json!([
        {
            "data_id": "binance_btc_quote",
            "venue": "binance",
            "symbol": "BTCUSDT",
            "data_type": "quote",
            "granularity": "1m",
            "lookback": 200,
            "fields": ["bid", "ask", "mid"]
        },
        {
            "data_id": "okx_btc_quote",
            "venue": "okx",
            "symbol": "BTCUSDT",
            "data_type": "quote",
            "granularity": "1m",
            "lookback": 200,
            "fields": ["bid", "ask", "mid"]
        }
    ]);

    let response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/strategy-ir/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "{}",
        String::from_utf8_lossy(&body)
    );
    serde_json::from_slice(&body).unwrap()
}

fn formal_compile_golden_view(value: &serde_json::Value) -> serde_json::Value {
    let indicator_kinds = value["core_ir"]["indicators"]
        .as_array()
        .unwrap()
        .iter()
        .map(|indicator| indicator["kind"].clone())
        .collect::<Vec<_>>();
    let agent_policy_kinds = value["core_ir"]["agent_policies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|policy| policy["kind"].clone())
        .collect::<Vec<_>>();

    serde_json::json!({
        "core_source_kind": value["core_ir"]["metadata"]["source_kind"].clone(),
        "data_bindings": value["core_ir"]["data_bindings"].clone(),
        "indicator_kinds": indicator_kinds,
        "signal_rules": value["core_ir"]["signal_rules"].clone(),
        "agent_policy_kinds": agent_policy_kinds,
        "runtime_projection": {
            "data_modules": value["runtime_config"]["data_sources"].as_array().unwrap().iter().map(|node| node["module_key"].clone()).collect::<Vec<_>>(),
            "intent_modules": value["runtime_config"]["intent_generators"].as_array().unwrap().iter().map(|node| node["module_key"].clone()).collect::<Vec<_>>(),
            "agent_modules": value["runtime_config"]["agents"].as_array().unwrap().iter().map(|node| node["module_key"].clone()).collect::<Vec<_>>(),
            "risk_modules": value["runtime_config"]["risk_controls"].as_array().unwrap().iter().map(|node| node["module_key"].clone()).collect::<Vec<_>>(),
            "execution_modules": value["runtime_config"]["executions"].as_array().unwrap().iter().map(|node| node["module_key"].clone()).collect::<Vec<_>>(),
            "runtime_module": value["runtime_config"]["runtime_control"]["module_key"].clone(),
        }
    })
}

fn formal_compile_error_golden_view(value: &serde_json::Value) -> serde_json::Value {
    // v1.1.2: 安全访问替代 unwrap 链，防止 panic
    let details = value["details"].as_array();
    let first_detail = details.and_then(|arr| arr.first());

    let detail_code = first_detail
        .and_then(|d| d.get("code"))
        .cloned()
        .unwrap_or(Value::Null);
    let detail_msg = first_detail
        .and_then(|d| d.get("message"))
        .cloned()
        .unwrap_or(Value::Null);
    let detail_reason = first_detail
        .and_then(|d| d.get("reason"))
        .cloned()
        .unwrap_or(Value::Null);
    let detail_span = first_detail
        .and_then(|d| d.get("span_label"))
        .cloned()
        .unwrap_or(Value::Null);

    serde_json::json!({
        "error": value["error"].clone(),
        "detail": {
            "code": detail_code,
            "message": detail_msg,
            "reason": detail_reason,
            "span_label": detail_span,
        }
    })
}

fn formal_compile_error_details_golden_view(value: &serde_json::Value) -> serde_json::Value {
    let details = value["details"]
        .as_array()
        .unwrap()
        .iter()
        .map(|detail| {
            serde_json::json!({
                "code": detail["code"].clone(),
                "message": detail["message"].clone(),
                "span_label": detail
                    .get("span_label")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "error": value["error"].clone(),
        "details": details,
    })
}

fn expected_formal_spread_rejection_golden_view() -> serde_json::Value {
    serde_json::json!({
        "error": "quantscript_lowering_failed",
        "detail": {
            "code": "QPQSLOW001",
            "message": "QPQSLOW001 不支持的条件下发 Intent 编译: 条件必须映射到支持的指标或价差意图",
            "reason": "将条件下发重写为支持的指标或价差意图，或保留下发为无条件。",
            "span_label": serde_json::Value::Null,
        }
    })
}

fn canonical_condition_for_entry_equivalence(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .iter()
                .map(canonical_condition_for_entry_equivalence)
                .collect(),
        ),
        serde_json::Value::Object(map) => {
            if map.get("kind").and_then(|kind| kind.as_str()) == Some("ref") {
                serde_json::json!({
                    "kind": "ref",
                    "name": "__ref__"
                })
            } else {
                let mut normalized = serde_json::Map::new();
                for (key, child) in map {
                    if key == "data_id" {
                        normalized.insert(
                            key.clone(),
                            serde_json::Value::String("__data__".to_string()),
                        );
                        continue;
                    }
                    normalized.insert(
                        key.clone(),
                        canonical_condition_for_entry_equivalence(child),
                    );
                }
                serde_json::Value::Object(normalized)
            }
        }
        _ => value.clone(),
    }
}

fn core_ir_entry_equivalence_view(core_ir: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "indicator_kind": core_ir["indicators"][0]["kind"].clone(),
        "condition": canonical_condition_for_entry_equivalence(&core_ir["signal_rules"][0]["condition"]),
    })
}

fn core_ir_risk_profile_equivalence_view(core_ir: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "max_position_ratio": core_ir["risk_policies"][0]["max_position_ratio"].clone(),
        "max_total_leverage": core_ir["risk_policies"][0]["max_total_leverage"].clone(),
        "max_exchange_leverage": core_ir["risk_policies"][0]["max_exchange_leverage"].clone(),
        "min_action_interval_ms": core_ir["risk_policies"][0]["min_action_interval_ms"].clone(),
    })
}

fn core_ir_execution_profile_equivalence_view(core_ir: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "venue_kind": core_ir["execution"]["venue_kind"].clone(),
        "taker_fee_bps": core_ir["execution"]["taker_fee_bps"].clone(),
        "slippage_bps": core_ir["execution"]["slippage_bps"].clone(),
    })
}

fn graph_value_from_runtime_config(
    runtime_config: &serde_json::Value,
    formal_source: &str,
) -> serde_json::Value {
    let mut nodes = Vec::<serde_json::Value>::new();
    let mut edges = Vec::<serde_json::Value>::new();

    for node in runtime_config["data_sources"].as_array().unwrap() {
        nodes.push(serde_json::json!({
            "id": node["id"].clone(),
            "type": "data",
            "module_key": node["module_key"].clone(),
            "name": node["name"].clone(),
            "config": node["config"].clone(),
        }));
    }

    for node in runtime_config["intent_generators"].as_array().unwrap() {
        nodes.push(serde_json::json!({
            "id": node["id"].clone(),
            "type": "intent",
            "module_key": node["module_key"].clone(),
            "name": node["name"].clone(),
            "config": node["config"].clone(),
        }));

        for input_ref in node["input_refs"].as_array().unwrap() {
            edges.push(serde_json::json!({
                "source_node_id": input_ref["source_id"].clone(),
                "source_port": input_ref["source_port"].clone(),
                "target_node_id": node["id"].clone(),
                "target_port": input_ref["target_port"].clone(),
            }));
        }
    }

    for node in runtime_config["agents"].as_array().unwrap() {
        nodes.push(serde_json::json!({
            "id": node["id"].clone(),
            "type": "agent",
            "module_key": node["module_key"].clone(),
            "name": node["name"].clone(),
            "config": node["config"].clone(),
        }));

        for intent_ref in node["intent_refs"].as_array().unwrap() {
            edges.push(serde_json::json!({
                "source_node_id": intent_ref.clone(),
                "source_port": "intent_out",
                "target_node_id": node["id"].clone(),
                "target_port": "intent_input",
            }));
        }
    }

    for node in runtime_config["risk_controls"].as_array().unwrap() {
        nodes.push(serde_json::json!({
            "id": node["id"].clone(),
            "type": "risk",
            "module_key": node["module_key"].clone(),
            "name": node["name"].clone(),
            "config": node["config"].clone(),
        }));

        for agent_ref in node["agent_refs"].as_array().unwrap() {
            edges.push(serde_json::json!({
                "source_node_id": agent_ref.clone(),
                "source_port": "agent_out",
                "target_node_id": node["id"].clone(),
                "target_port": "agent_input",
            }));
        }
    }

    for node in runtime_config["executions"].as_array().unwrap() {
        nodes.push(serde_json::json!({
            "id": node["id"].clone(),
            "type": "execution",
            "module_key": node["module_key"].clone(),
            "name": node["name"].clone(),
            "config": node["config"].clone(),
        }));

        if !node["risk_ref"].is_null() {
            edges.push(serde_json::json!({
                "source_node_id": node["risk_ref"].clone(),
                "source_port": "risk_out",
                "target_node_id": node["id"].clone(),
                "target_port": "risk_input",
            }));
        }
    }

    let runtime_node = &runtime_config["runtime_control"];
    nodes.push(serde_json::json!({
        "id": runtime_node["id"].clone(),
        "type": "runtime",
        "module_key": runtime_node["module_key"].clone(),
        "name": runtime_node["name"].clone(),
        "config": runtime_node["config"].clone(),
    }));

    if let Some(execution_node) = runtime_config["executions"].as_array().unwrap().first() {
        edges.push(serde_json::json!({
            "source_node_id": execution_node["id"].clone(),
            "source_port": "execution_out",
            "target_node_id": runtime_node["id"].clone(),
            "target_port": "execution_input",
        }));
    }

    serde_json::json!({
        "metadata": {
            "graph_id": runtime_config["metadata"]["graph_id"].clone(),
            "name": runtime_config["metadata"]["name"].clone(),
            "version": runtime_config["metadata"]["version"].clone(),
            "artifacts": {
                "quantscript": {
                    "formal_source": formal_source,
                }
            }
        },
        "nodes": nodes,
        "edges": edges,
    })
}

fn api_error_detail_by_code<'a>(value: &'a serde_json::Value, code: &str) -> &'a serde_json::Value {
    value["details"]
        .as_array()
        .unwrap()
        .iter()
        .find(|detail| detail["code"] == code)
        .unwrap_or_else(|| panic!("missing api error detail for code {code}"))
}

#[test]
fn attach_quantscript_artifacts_preserves_node_source_targets() {
    let mut graph = serde_json::json!({
        "metadata": {
            "graph_id": "graph_test",
            "name": "Test Graph",
            "version": "1.0.0"
        },
        "nodes": [
            {
                "id": "data_feed",
                "type": "data",
                "module_key": "builtin.data.kline",
                "name": "Price Feed",
                "config": {
                    "window_size": 20,
                    "timeframe": "1d"
                }
            }
        ],
        "edges": []
    });

    attach_quantscript_artifacts(
        &mut graph,
        "strategy_graph graph_test {\n}",
        1,
        std::path::Path::new("storage/graphs/graph_test.qs"),
    );

    let quantscript = &graph["metadata"]["artifacts"]["quantscript"];
    assert!(quantscript["node_sources"]["data_feed"].is_string());
    assert_eq!(
        quantscript["label_targets"]["Price Feed.window_size"]["node_id"],
        "data_feed"
    );
    assert_eq!(
        quantscript["label_targets"]["Price Feed.window_size"]["field"],
        "window_size"
    );
    assert_eq!(
        quantscript["runtime_targets"]["source_to_node"]["data_data_feed"],
        "data_feed"
    );
}

#[test]
fn attach_quantscript_artifacts_preserves_formal_source() {
    let mut graph = serde_json::json!({
        "metadata": {
            "graph_id": "graph_test",
            "name": "Test Graph",
            "version": "1.0.0",
            "artifacts": {
                "quantscript": {
                    "formal_source": "fn strategy() {\n    emit Intent(\"BUY\", instrument=\"BTCUSDT\", quantity=1.0)\n}"
                }
            }
        },
        "nodes": [],
        "edges": []
    });

    attach_quantscript_artifacts(
        &mut graph,
        "strategy_graph graph_test {\n}",
        1,
        std::path::Path::new("storage/graphs/graph_test.qs"),
    );

    assert_eq!(
        graph["metadata"]["artifacts"]["quantscript"]["formal_source"],
        "fn strategy() {\n    emit Intent(\"BUY\", instrument=\"BTCUSDT\", quantity=1.0)\n}"
    );
}

#[tokio::test]
async fn compile_endpoint_accepts_spread_arbitrage_modules_and_lowers_spread_indicator() {
    let graph_app = build_app_router(test_app_state());

    let response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(sample_spread_compile_request_json().to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["indicators"][0]["kind"],
        "spread"
    );
    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["indicators"][0]["spread_spec"]["output"],
        "bps"
    );
    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["agent_policies"][0]["kind"],
        "cross_venue_arbitrage"
    );
}

#[tokio::test]
async fn compile_endpoint_roundtrips_data_request_controls() {
    let graph_app = build_app_router(test_app_state());
    let mut payload = sample_compile_request_json();
    payload["runtime_config"]["data_sources"][0]["config"]["ping_enabled"] =
        serde_json::Value::Bool(true);
    payload["runtime_config"]["data_sources"][0]["config"]["request_interval_ms"] =
        serde_json::Value::from(2_500_u64);

    let response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["data_bindings"][0]["source_hints"]
            ["ping_enabled"],
        "true"
    );
    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["data_bindings"][0]["source_hints"]
            ["request_interval_ms"],
        "2500"
    );
}

#[tokio::test]
async fn compile_endpoint_lowers_graph_spread_bps_to_structured_threshold_condition() {
    let value = compile_runtime_spread_graph_for_test(
        serde_json::json!({
            "max_time_diff_ms": 5000,
            "field_code": 0,
            "align_direction_code": 0,
            "resample_period_ms": 0,
            "resample_agg_code": 0,
            "window_size": 1,
            "window_agg_code": 1,
            "spread_output_code": 1,
            "comparison_shape_code": 1,
            "comparison_op_code": 2,
            "comparison_threshold": 5.0
        }),
        "compile_graph_spread_threshold",
    )
    .await;

    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["indicators"][0]["kind"],
        "spread"
    );
    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["signal_rules"][0]["condition"],
        serde_json::json!({
            "kind": "compare",
            "left": {
                "kind": "ref",
                "name": "intent_spread_1"
            },
            "op": "gt",
            "right": {
                "kind": "number",
                "value": 5.0
            }
        })
    );
}

#[tokio::test]
async fn compile_endpoint_rejects_graph_spread_threshold_with_non_bps_output() {
    let value = compile_runtime_spread_graph_error_for_test(
        serde_json::json!({
            "max_time_diff_ms": 5000,
            "field_code": 0,
            "align_direction_code": 0,
            "resample_period_ms": 0,
            "resample_agg_code": 0,
            "window_size": 1,
            "window_agg_code": 1,
            "spread_output_code": 0,
            "comparison_shape_code": 1,
            "comparison_op_code": 2,
            "comparison_threshold": 5.0
        }),
        "compile_graph_spread_non_bps_reject",
    )
    .await;

    assert_eq!(value["error"], "runtime_compile_failed");
    assert_eq!(value["details"][0]["code"], "QPSPREAD001");
}

#[tokio::test]
async fn compile_endpoint_rejects_graph_spread_threshold_with_non_positive_tolerance() {
    let value = compile_runtime_spread_graph_error_for_test(
        serde_json::json!({
            "max_time_diff_ms": 0,
            "field_code": 0,
            "align_direction_code": 0,
            "resample_period_ms": 0,
            "resample_agg_code": 0,
            "window_size": 1,
            "window_agg_code": 1,
            "spread_output_code": 1,
            "comparison_shape_code": 1,
            "comparison_op_code": 2,
            "comparison_threshold": 5.0
        }),
        "compile_graph_spread_bad_tolerance_reject",
    )
    .await;

    assert_eq!(value["error"], "runtime_compile_failed");
    assert_eq!(value["details"][0]["code"], "QPSPREAD002");
}

#[tokio::test]
async fn compile_endpoint_rejects_graph_spread_threshold_with_non_one_sided_shape() {
    let value = compile_runtime_spread_graph_error_for_test(
        serde_json::json!({
            "max_time_diff_ms": 5000,
            "field_code": 0,
            "align_direction_code": 0,
            "resample_period_ms": 0,
            "resample_agg_code": 0,
            "window_size": 1,
            "window_agg_code": 1,
            "spread_output_code": 1,
            "comparison_shape_code": 2,
            "comparison_op_code": 0,
            "comparison_threshold": 5.0
        }),
        "compile_graph_spread_bad_shape_reject",
    )
    .await;

    assert_eq!(value["error"], "runtime_compile_failed");
    assert_eq!(value["details"][0]["code"], "QPSPREAD003");
}

#[tokio::test]
async fn compile_endpoint_rejects_unsupported_runtime_mode_with_structured_error() {
    let app = build_app_router(test_app_state());
    let mut payload = sample_compile_request_json();
    payload["runtime_config"]["metadata"]["mode"] = serde_json::Value::String("live".to_string());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "capability_gated");
    assert_eq!(value["details"][0]["code"], "unsupported_runtime_mode");
    assert_eq!(value["details"][0]["target"], "metadata.mode");
}

#[tokio::test]
async fn compile_endpoint_returns_warmup_diagnostics() {
    let app = build_app_router(test_app_state());
    let mut payload = sample_compile_request_json();
    payload["runtime_config"]["data_sources"][0]["config"]["window_size"] =
        serde_json::Value::from(20);
    payload["runtime_config"]["intent_generators"][0]["config"]["slow_period"] =
        serde_json::Value::from(50);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["diagnostics"][0]["code"], "QPWARM001");
    assert_eq!(value["diagnostics"][0]["severity"], "warning");
    assert_eq!(value["diagnostics"][0]["target"]["scope"], "node");
    assert_eq!(value["diagnostics"][0]["target"]["node_id"], "data_data_1");
    assert_eq!(value["diagnostics"][0]["target"]["field"], "window_size");
}

#[tokio::test]
async fn compile_endpoint_returns_artifact_bundle() {
    let app = build_app_router(test_app_state());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(sample_compile_request_json().to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        value["artifacts"]["strategy"]["schema_version"],
        "quantpilot/strategy-artifact/v1"
    );
    assert_eq!(
        value["artifacts"]["compile"]["schema_version"],
        "quantpilot/compile-artifact/v1"
    );
    assert_eq!(
        value["artifacts"]["core_ir"]["schema_version"],
        "quantpilot/core-ir-artifact/v1"
    );
    assert_eq!(
        value["artifacts"]["compile"]["strategy_artifact_id"],
        value["artifacts"]["strategy"]["artifact_id"]
    );
    assert_eq!(
        value["artifacts"]["compile"]["core_ir_artifact_id"],
        value["artifacts"]["core_ir"]["artifact_id"]
    );
    assert_eq!(
        value["artifacts"]["compile"]["config_hash"],
        value["config_hash"]
    );
}

#[tokio::test]
async fn compile_endpoint_lowers_graph_momentum_to_structured_threshold_condition() {
    let app = build_app_router(test_app_state());
    let mut payload = sample_compile_request_json();
    payload["runtime_config"]["intent_generators"][0]["module_key"] =
        serde_json::Value::String("builtin.intent.momentum".to_string());
    payload["runtime_config"]["intent_generators"][0]["config"] = serde_json::json!({
        "lookback": 20,
        "threshold_ratio": 0.03
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["indicators"][0]["kind"],
        "momentum"
    );
    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["signal_rules"][0]["condition"],
        serde_json::json!({
            "kind": "compare",
            "left": {
                "kind": "ref",
                "name": "intent_intent_1"
            },
            "op": "gt",
            "right": {
                "kind": "number",
                "value": 0.03
            }
        })
    );
}

#[tokio::test]
async fn compile_endpoint_lowers_graph_rsi_to_structured_threshold_condition() {
    let app = build_app_router(test_app_state());
    let mut payload = sample_compile_request_json();
    payload["runtime_config"]["intent_generators"][0]["module_key"] =
        serde_json::Value::String("builtin.intent.rsi".to_string());
    payload["runtime_config"]["intent_generators"][0]["config"] = serde_json::json!({
        "period": 14,
        "oversold_threshold": 25.0,
        "overbought_threshold": 70.0
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["indicators"][0]["kind"],
        "rsi"
    );
    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["signal_rules"][0]["condition"],
        serde_json::json!({
            "kind": "compare",
            "left": {
                "kind": "ref",
                "name": "intent_intent_1"
            },
            "op": "lt",
            "right": {
                "kind": "number",
                "value": 25.0
            }
        })
    );
}

#[tokio::test]
async fn compile_endpoint_lowers_graph_zscore_to_structured_threshold_condition() {
    let app = build_app_router(test_app_state());
    let mut payload = sample_compile_request_json();
    payload["runtime_config"]["intent_generators"][0]["module_key"] =
        serde_json::Value::String("builtin.intent.zscore".to_string());
    payload["runtime_config"]["intent_generators"][0]["config"] = serde_json::json!({
        "window": 20,
        "entry_z": 2.0
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["indicators"][0]["kind"],
        "z_score"
    );
    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["signal_rules"][0]["condition"],
        serde_json::json!({
            "kind": "compare",
            "left": {
                "kind": "ref",
                "name": "intent_intent_1"
            },
            "op": "lt",
            "right": {
                "kind": "number",
                "value": -2.0
            }
        })
    );
}

#[tokio::test]
async fn strategy_ir_compile_endpoint_accepts_restricted_custom_and_lowers_to_core_ir() {
    let app = build_app_router(test_app_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/strategy-ir/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    sample_strategy_ir_compile_request_json().to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["graph_id"], "strategy_graph_test");
    assert_eq!(value["compile_id"], "strategy_compile_test");
    assert_eq!(value["compilable"], true);
    assert_eq!(value["core_ir"]["indicators"][0]["kind"], "custom");
    assert_eq!(
        value["core_ir"]["indicators"][0]["custom_expr"]["schema_version"],
        "quantpilot/custom-expr/v1"
    );
    assert_eq!(value["diagnostics"], serde_json::json!([]));
}

#[tokio::test]
async fn strategy_ir_compile_endpoint_lowers_rsi_to_structured_threshold_condition() {
    let app = build_app_router(test_app_state());
    let mut payload = sample_strategy_ir_compile_request_json();
    payload["strategy_ir"]["signals"][0]["signal_id"] =
        serde_json::Value::String("rsi_signal".to_string());
    payload["strategy_ir"]["signals"][0]["name"] =
        serde_json::Value::String("RSI Signal".to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
        serde_json::Value::String("rsi".to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["params"] = serde_json::json!({
        "period": 14
    });
    payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
        serde_json::Value::String("rsi_signal < 25".to_string());
    payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
        serde_json::Value::String("open_long".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/strategy-ir/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["core_ir"]["indicators"][0]["kind"], "rsi");
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"],
        serde_json::json!({
            "kind": "compare",
            "left": {
                "kind": "ref",
                "name": "rsi_signal"
            },
            "op": "lt",
            "right": {
                "kind": "number",
                "value": 25.0
            }
        })
    );
}

#[tokio::test]
async fn strategy_ir_compile_endpoint_lowers_ma_cross_to_structured_series_compare() {
    let app = build_app_router(test_app_state());
    let mut payload = sample_strategy_ir_compile_request_json();
    payload["strategy_ir"]["signals"][0]["signal_id"] =
        serde_json::Value::String("ma_cross".to_string());
    payload["strategy_ir"]["signals"][0]["name"] =
        serde_json::Value::String("MA Cross".to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
        serde_json::Value::String("ma_cross".to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["params"] = serde_json::json!({
        "fast": 20,
        "slow": 50
    });
    payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
        serde_json::Value::String("ma_cross > 0".to_string());
    payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
        serde_json::Value::String("open_long".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/strategy-ir/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["core_ir"]["indicators"][0]["kind"], "ma_cross");
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"],
        serde_json::json!({
            "kind": "compare",
            "left": {
                "kind": "series",
                "expr": {
                    "kind": "window_agg",
                    "input": {
                        "kind": "data_field",
                        "data_id": "btc_1d",
                        "field": "close"
                    },
                    "window_size": 20,
                    "agg": "mean"
                }
            },
            "op": "gt",
            "right": {
                "kind": "series",
                "expr": {
                    "kind": "window_agg",
                    "input": {
                        "kind": "data_field",
                        "data_id": "btc_1d",
                        "field": "close"
                    },
                    "window_size": 50,
                    "agg": "mean"
                }
            }
        })
    );
}

#[tokio::test]
async fn strategy_ir_compile_endpoint_lowers_momentum_to_structured_threshold_condition() {
    let app = build_app_router(test_app_state());
    let mut payload = sample_strategy_ir_compile_request_json();
    payload["strategy_ir"]["signals"][0]["signal_id"] =
        serde_json::Value::String("momentum_signal".to_string());
    payload["strategy_ir"]["signals"][0]["name"] =
        serde_json::Value::String("Momentum Signal".to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
        serde_json::Value::String("momentum".to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["params"] = serde_json::json!({
        "lookback": 20
    });
    payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
        serde_json::Value::String("momentum_signal > 0.03".to_string());
    payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
        serde_json::Value::String("open_long".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/strategy-ir/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["core_ir"]["indicators"][0]["kind"], "momentum");
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"],
        serde_json::json!({
            "kind": "compare",
            "left": {
                "kind": "ref",
                "name": "momentum_signal"
            },
            "op": "gt",
            "right": {
                "kind": "number",
                "value": 0.03
            }
        })
    );
}

#[tokio::test]
async fn strategy_ir_compile_endpoint_lowers_zscore_to_structured_threshold_condition() {
    let app = build_app_router(test_app_state());
    let mut payload = sample_strategy_ir_compile_request_json();
    payload["strategy_ir"]["signals"][0]["signal_id"] =
        serde_json::Value::String("zscore_signal".to_string());
    payload["strategy_ir"]["signals"][0]["name"] =
        serde_json::Value::String("ZScore Signal".to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
        serde_json::Value::String("z_score".to_string());
    payload["strategy_ir"]["signals"][0]["indicator"]["params"] = serde_json::json!({
        "window": 20
    });
    payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
        serde_json::Value::String("zscore_signal < -2".to_string());
    payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
        serde_json::Value::String("open_long".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/strategy-ir/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["core_ir"]["indicators"][0]["kind"], "z_score");
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"],
        serde_json::json!({
            "kind": "compare",
            "left": {
                "kind": "ref",
                "name": "zscore_signal"
            },
            "op": "lt",
            "right": {
                "kind": "number",
                "value": -2.0
            }
        })
    );
}

#[tokio::test]
async fn strategy_ir_compile_endpoint_lowers_spread_to_structured_threshold_condition() {
    let value = compile_strategy_ir_spread_for_test(
        serde_json::json!({
            "align_direction_code": 0,
            "max_time_diff_ms": 5000,
            "spread_output_code": 1
        }),
        "spread_signal > 5",
        "compile_strategy_ir_spread_threshold",
    )
    .await;

    assert_eq!(value["core_ir"]["indicators"][0]["kind"], "spread");
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"],
        serde_json::json!({
            "kind": "compare",
            "left": {
                "kind": "ref",
                "name": "spread_signal"
            },
            "op": "gt",
            "right": {
                "kind": "number",
                "value": 5.0
            }
        })
    );
}

#[tokio::test]
async fn strategy_ir_compile_endpoint_rejects_spread_with_non_bps_output() {
    let value = compile_strategy_ir_spread_error_for_test(
        serde_json::json!({
            "align_direction_code": 0,
            "max_time_diff_ms": 5000,
            "spread_output_code": 0
        }),
        "spread_signal > 5",
        "compile_strategy_ir_spread_non_bps_reject",
    )
    .await;

    assert_eq!(value["error"], "strategy_ir_compile_failed");
    assert_eq!(value["details"][0]["code"], "QPSTRATSPREAD001");
}

#[tokio::test]
async fn strategy_ir_compile_endpoint_rejects_spread_with_non_positive_tolerance() {
    let value = compile_strategy_ir_spread_error_for_test(
        serde_json::json!({
            "align_direction_code": 0,
            "max_time_diff_ms": 0,
            "spread_output_code": 1
        }),
        "spread_signal > 5",
        "compile_strategy_ir_spread_bad_tolerance_reject",
    )
    .await;

    assert_eq!(value["error"], "strategy_ir_compile_failed");
    assert_eq!(value["details"][0]["code"], "QPSTRATSPREAD002");
}

#[tokio::test]
async fn strategy_ir_compile_endpoint_rejects_spread_with_non_one_sided_shape() {
    let value = compile_strategy_ir_spread_error_for_test(
        serde_json::json!({
            "align_direction_code": 0,
            "max_time_diff_ms": 5000,
            "spread_output_code": 1
        }),
        "spread_signal < -5",
        "compile_strategy_ir_spread_bad_shape_reject",
    )
    .await;

    assert_eq!(value["error"], "strategy_ir_compile_failed");
    assert_eq!(value["details"][0]["code"], "QPSTRATSPREAD003");
}

#[tokio::test]
async fn spread_bps_condition_lowers_equivalently_across_graph_and_strategy_ir() {
    let graph = compile_runtime_spread_graph_for_test(
        serde_json::json!({
            "max_time_diff_ms": 5000,
            "field_code": 0,
            "align_direction_code": 0,
            "resample_period_ms": 0,
            "resample_agg_code": 0,
            "window_size": 1,
            "window_agg_code": 1,
            "spread_output_code": 1,
            "comparison_shape_code": 1,
            "comparison_op_code": 2,
            "comparison_threshold": 5.0
        }),
        "compile_graph_spread_equivalence",
    )
    .await;
    let strategy = compile_strategy_ir_spread_for_test(
        serde_json::json!({
            "align_direction_code": 0,
            "max_time_diff_ms": 5000,
            "spread_output_code": 1
        }),
        "spread_signal > 5",
        "compile_strategy_ir_spread_equivalence",
    )
    .await;

    let graph_view = core_ir_entry_equivalence_view(&graph["artifacts"]["core_ir"]["core_ir"]);
    let strategy_view = core_ir_entry_equivalence_view(&strategy["core_ir"]);

    assert_eq!(graph_view, strategy_view);
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_lowers_admitted_spread_to_structured_threshold_condition(
) {
    let value = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_threshold",
        )
        .await;

    assert_eq!(value["core_ir"]["indicators"][0]["kind"], "spread");
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"],
        serde_json::json!({
            "kind": "compare",
            "left": {
                "kind": "ref",
                "name": "intent_btcusdt_spread"
            },
            "op": "gt",
            "right": {
                "kind": "number",
                "value": 5.0
            }
        })
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_rejects_spread_with_non_bps_output() {
    let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="ratio")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_non_bps",
        )
        .await;

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(
        api_error_detail_by_code(&value, "QPQSLOW001")["code"],
        "QPQSLOW001"
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_rejects_spread_without_explicit_align_asof() {
    let value = compile_formal_quantscript_error_for_test(
        r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let spread_signal = spread(field(left, name="bid"), field(right, name="ask"), output="bps")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_spread_missing_align_asof",
    )
    .await;

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(
        api_error_detail_by_code(&value, "QPQSLOW001")["code"],
        "QPQSLOW001"
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_rejects_spread_with_non_positive_tolerance() {
    let value = compile_formal_quantscript_error_for_test(
        r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=0)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=0)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_spread_non_positive_tolerance",
    )
    .await;

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(
        api_error_detail_by_code(&value, "QPQSLOW001")["code"],
        "QPQSLOW001"
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_rejects_spread_with_non_one_sided_shape() {
    let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")
    if spread_signal < 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_non_one_sided",
        )
        .await;

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(
        api_error_detail_by_code(&value, "QPQSLOW001")["code"],
        "QPQSLOW001"
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_spread_non_bps_diagnostic_golden_view() {
    let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="ratio")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_non_bps_golden",
        )
        .await;

    assert_eq!(
        formal_compile_error_golden_view(&value),
        expected_formal_spread_rejection_golden_view()
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_spread_missing_align_diagnostic_golden_view() {
    let value = compile_formal_quantscript_error_for_test(
        r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let spread_signal = spread(field(left, name="bid"), field(right, name="ask"), output="bps")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_spread_missing_align_golden",
    )
    .await;

    assert_eq!(
        formal_compile_error_golden_view(&value),
        expected_formal_spread_rejection_golden_view()
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_spread_non_positive_tolerance_diagnostic_golden_view(
) {
    let value = compile_formal_quantscript_error_for_test(
        r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=0)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=0)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_spread_non_positive_tolerance_golden",
    )
    .await;

    assert_eq!(
        formal_compile_error_golden_view(&value),
        expected_formal_spread_rejection_golden_view()
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_spread_non_one_sided_diagnostic_golden_view() {
    let value = compile_formal_quantscript_error_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")
    if spread_signal < 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_non_one_sided_golden",
        )
        .await;

    assert_eq!(
        formal_compile_error_golden_view(&value),
        expected_formal_spread_rejection_golden_view()
    );
}

#[tokio::test]
async fn spread_bps_condition_lowers_equivalently_across_formal_graph_and_strategy_ir() {
    let formal = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")
    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_spread_equivalence",
        )
        .await;
    let graph = compile_runtime_spread_graph_for_test(
        serde_json::json!({
            "max_time_diff_ms": 5000,
            "field_code": 0,
            "align_direction_code": 0,
            "resample_period_ms": 0,
            "resample_agg_code": 0,
            "window_size": 1,
            "window_agg_code": 1,
            "spread_output_code": 1,
            "comparison_shape_code": 1,
            "comparison_op_code": 2,
            "comparison_threshold": 5.0
        }),
        "compile_graph_spread_formal_equivalence",
    )
    .await;
    let strategy = compile_strategy_ir_spread_for_test(
        serde_json::json!({
            "align_direction_code": 0,
            "max_time_diff_ms": 5000,
            "spread_output_code": 1
        }),
        "spread_signal > 5",
        "compile_strategy_ir_spread_formal_equivalence",
    )
    .await;

    let formal_view = core_ir_entry_equivalence_view(&formal["core_ir"]);
    let graph_view = core_ir_entry_equivalence_view(&graph["artifacts"]["core_ir"]["core_ir"]);
    let strategy_view = core_ir_entry_equivalence_view(&strategy["core_ir"]);

    assert_eq!(formal_view, graph_view);
    assert_eq!(formal_view, strategy_view);
}

#[tokio::test]
async fn one_sided_rsi_condition_lowers_equivalently_across_formal_graph_and_strategy_ir() {
    let formal = compile_formal_quantscript_for_test(
        r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let r = rsi(data_feed, 14)
    if r < 25 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_rsi_equivalence",
    )
    .await;
    let graph = compile_runtime_graph_for_test(
        "builtin.intent.rsi",
        serde_json::json!({
            "period": 14,
            "oversold_threshold": 25.0,
            "overbought_threshold": 70.0
        }),
        "compile_graph_rsi_equivalence",
    )
    .await;
    let strategy = compile_strategy_ir_for_test(
        "rsi_signal",
        "RSI Signal",
        "rsi",
        serde_json::json!({ "period": 14 }),
        "rsi_signal < 25",
        "compile_strategy_ir_rsi_equivalence",
    )
    .await;

    let formal_view = core_ir_entry_equivalence_view(&formal["core_ir"]);
    let graph_view = core_ir_entry_equivalence_view(&graph["artifacts"]["core_ir"]["core_ir"]);
    let strategy_view = core_ir_entry_equivalence_view(&strategy["core_ir"]);

    assert_eq!(formal_view, graph_view);
    assert_eq!(formal_view, strategy_view);
}

#[tokio::test]
async fn one_sided_momentum_condition_lowers_equivalently_across_formal_graph_and_strategy_ir() {
    let formal = compile_formal_quantscript_for_test(
        r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_momentum_equivalence",
    )
    .await;
    let graph = compile_runtime_graph_for_test(
        "builtin.intent.momentum",
        serde_json::json!({
            "lookback": 20,
            "threshold_ratio": 0.03
        }),
        "compile_graph_momentum_equivalence",
    )
    .await;
    let strategy = compile_strategy_ir_for_test(
        "momentum_signal",
        "Momentum Signal",
        "momentum",
        serde_json::json!({ "lookback": 20 }),
        "momentum_signal > 0.03",
        "compile_strategy_ir_momentum_equivalence",
    )
    .await;

    let formal_view = core_ir_entry_equivalence_view(&formal["core_ir"]);
    let graph_view = core_ir_entry_equivalence_view(&graph["artifacts"]["core_ir"]["core_ir"]);
    let strategy_view = core_ir_entry_equivalence_view(&strategy["core_ir"]);

    assert_eq!(formal_view, graph_view);
    assert_eq!(formal_view, strategy_view);
}

#[tokio::test]
async fn one_sided_zscore_condition_lowers_equivalently_across_formal_graph_and_strategy_ir() {
    let formal = compile_formal_quantscript_for_test(
        r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let z = zscore(data_feed, 20)
    if z < -2 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_zscore_equivalence",
    )
    .await;
    let graph = compile_runtime_graph_for_test(
        "builtin.intent.zscore",
        serde_json::json!({
            "window": 20,
            "entry_z": 2.0
        }),
        "compile_graph_zscore_equivalence",
    )
    .await;
    let strategy = compile_strategy_ir_for_test(
        "zscore_signal",
        "ZScore Signal",
        "z_score",
        serde_json::json!({ "window": 20 }),
        "zscore_signal < -2",
        "compile_strategy_ir_zscore_equivalence",
    )
    .await;

    let formal_view = core_ir_entry_equivalence_view(&formal["core_ir"]);
    let graph_view = core_ir_entry_equivalence_view(&graph["artifacts"]["core_ir"]["core_ir"]);
    let strategy_view = core_ir_entry_equivalence_view(&strategy["core_ir"]);

    assert_eq!(formal_view, graph_view);
    assert_eq!(formal_view, strategy_view);
}

#[tokio::test]
async fn direct_ma_condition_lowers_equivalently_across_formal_graph_and_strategy_ir() {
    let formal = compile_formal_quantscript_for_test(
        r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(data_feed, 20)
    let slow = sma(data_feed, 100)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_direct_ma_equivalence",
    )
    .await;
    let graph = compile_runtime_graph_for_test(
        "builtin.intent.double_ma",
        serde_json::json!({
            "fast_period": 20,
            "slow_period": 100,
            "entry_ratio": 1.0
        }),
        "compile_graph_direct_ma_equivalence",
    )
    .await;
    let strategy = compile_strategy_ir_for_test(
        "ma_cross",
        "MA Cross",
        "ma_cross",
        serde_json::json!({
            "fast": 20,
            "slow": 100
        }),
        "ma_cross > 0",
        "compile_strategy_ir_direct_ma_equivalence",
    )
    .await;

    let formal_view = core_ir_entry_equivalence_view(&formal["core_ir"]);
    let graph_view = core_ir_entry_equivalence_view(&graph["artifacts"]["core_ir"]["core_ir"]);
    let strategy_view = core_ir_entry_equivalence_view(&strategy["core_ir"]);

    assert_eq!(formal_view["indicator_kind"], graph_view["indicator_kind"]);
    assert_eq!(
        formal_view["indicator_kind"],
        strategy_view["indicator_kind"]
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_lowers_global_risk_profile_to_runtime_global_risk_node(
) {
    let value = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    risk.profile("global", max_position=0.35, max_total_leverage=4.0, max_exchange_leverage=5.0, min_action_interval_ms=250)
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let r = rsi(data_feed, 14)
    if r < 25 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_risk_profile_global",
        )
        .await;

    assert_eq!(
        value["runtime_config"]["risk_controls"][0]["module_key"],
        "builtin.risk.global"
    );
    assert_eq!(
        value["runtime_config"]["risk_controls"][0]["config"]["profile_id"],
        "global"
    );
    assert_eq!(
        value["core_ir"]["risk_policies"][0]["max_position_ratio"],
        serde_json::json!(0.35)
    );
    assert_eq!(
        value["core_ir"]["risk_policies"][0]["max_total_leverage"],
        serde_json::json!(4.0)
    );
    assert_eq!(
        value["core_ir"]["risk_policies"][0]["max_exchange_leverage"],
        serde_json::json!(5.0)
    );
    assert_eq!(
        value["core_ir"]["risk_policies"][0]["min_action_interval_ms"],
        serde_json::json!(250)
    );
}

#[tokio::test]
async fn global_risk_profile_lowers_equivalently_across_formal_graph_and_strategy_ir() {
    let formal = compile_formal_quantscript_for_test(
            r#"
fn strategy() {
    risk.profile("global", max_position=0.35, max_total_leverage=4.0, max_exchange_leverage=5.0, min_action_interval_ms=250)
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
            "compile_formal_risk_profile_equivalence",
        )
        .await;

    let graph_app = build_app_router(test_app_state());
    let mut graph_payload = sample_compile_request_json();
    graph_payload["compile_id"] =
        serde_json::Value::String("compile_graph_risk_profile_equivalence".to_string());
    graph_payload["runtime_config"]["intent_generators"][0]["module_key"] =
        serde_json::Value::String("builtin.intent.momentum".to_string());
    graph_payload["runtime_config"]["intent_generators"][0]["config"] = serde_json::json!({
        "lookback": 20,
        "threshold": 0.03
    });
    graph_payload["runtime_config"]["risk_controls"][0]["config"] = serde_json::json!({
        "profile_id": "global",
        "max_position": 0.35,
        "max_total_leverage": 4.0,
        "max_exchange_leverage": 5.0,
        "min_action_interval_ms": 250
    });

    let graph_response = graph_app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(graph_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(graph_response.status(), StatusCode::OK);
    let graph_body = to_bytes(graph_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let graph: serde_json::Value = serde_json::from_slice(&graph_body).unwrap();

    let mut strategy_payload = sample_strategy_ir_compile_request_json();
    strategy_payload["compile_id"] =
        serde_json::Value::String("compile_strategy_ir_risk_profile_equivalence".to_string());
    strategy_payload["strategy_ir"]["signals"][0]["signal_id"] =
        serde_json::Value::String("momentum_signal".to_string());
    strategy_payload["strategy_ir"]["signals"][0]["name"] =
        serde_json::Value::String("Momentum Signal".to_string());
    strategy_payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
        serde_json::Value::String("momentum".to_string());
    strategy_payload["strategy_ir"]["signals"][0]["indicator"]["params"] = serde_json::json!({
        "lookback": 20
    });
    strategy_payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
        serde_json::Value::String("momentum_signal > 0.03".to_string());
    strategy_payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
        serde_json::Value::String("open_long".to_string());
    strategy_payload["strategy_ir"]["risk_profile"] = serde_json::json!({
        "profile_id": "global",
        "max_position": 0.35,
        "max_total_leverage": 4.0,
        "max_exchange_leverage": 5.0,
        "min_action_interval_ms": 250
    });

    let strategy_app = build_app_router(test_app_state());
    let strategy_response = strategy_app
        .oneshot(
            Request::builder()
                .uri("/api/strategy-ir/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(strategy_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(strategy_response.status(), StatusCode::OK);
    let strategy_body = to_bytes(strategy_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let strategy: serde_json::Value = serde_json::from_slice(&strategy_body).unwrap();

    let formal_view = core_ir_risk_profile_equivalence_view(&formal["core_ir"]);
    let graph_view = core_ir_risk_profile_equivalence_view(&graph["core_ir"]);
    let strategy_view = core_ir_risk_profile_equivalence_view(&strategy["core_ir"]);

    assert_eq!(
        formal["runtime_config"]["risk_controls"][0]["config"]["profile_id"],
        "global"
    );
    assert_eq!(
        graph["runtime_config"]["risk_controls"][0]["config"]["profile_id"],
        "global"
    );
    assert_eq!(formal_view, graph_view);
    assert_eq!(formal_view, strategy_view);
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_lowers_paper_execution_profile_to_runtime_execution_node(
) {
    let value = compile_formal_quantscript_for_test(
        r#"
fn strategy() {
    execution.profile("paper", fee_bps=12.5, slippage_bps=7.5)
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_execution_profile_paper",
    )
    .await;

    assert_eq!(
        value["runtime_config"]["executions"][0]["module_key"],
        "builtin.execution.paper"
    );
    assert_eq!(
        value["runtime_config"]["executions"][0]["config"]["profile_id"],
        "paper"
    );
    assert_eq!(
        value["runtime_config"]["executions"][0]["config"]["fee_bps"],
        serde_json::json!(12.5)
    );
    assert_eq!(
        value["runtime_config"]["executions"][0]["config"]["slippage_bps"],
        serde_json::json!(7.5)
    );
    assert_eq!(value["core_ir"]["execution"]["venue_kind"], "paper");
    assert_eq!(
        value["core_ir"]["execution"]["taker_fee_bps"],
        serde_json::json!(12.5)
    );
    assert_eq!(
        value["core_ir"]["execution"]["slippage_bps"],
        serde_json::json!(7.5)
    );
}

#[tokio::test]
async fn paper_execution_profile_lowers_equivalently_across_formal_graph_and_strategy_ir() {
    let formal = compile_formal_quantscript_for_test(
        r#"
fn strategy() {
    execution.profile("paper", fee_bps=12.5, slippage_bps=7.5)
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_execution_profile_equivalence",
    )
    .await;

    let graph_app = build_app_router(test_app_state());
    let mut graph_payload = sample_compile_request_json();
    graph_payload["compile_id"] =
        serde_json::Value::String("compile_graph_execution_profile_equivalence".to_string());
    graph_payload["runtime_config"]["intent_generators"][0]["module_key"] =
        serde_json::Value::String("builtin.intent.momentum".to_string());
    graph_payload["runtime_config"]["intent_generators"][0]["config"] = serde_json::json!({
        "lookback": 20,
        "threshold": 0.03
    });
    graph_payload["runtime_config"]["executions"][0]["config"] = serde_json::json!({
        "profile_id": "paper",
        "mode": "paper",
        "fee_bps": 12.5,
        "slippage_bps": 7.5
    });

    let graph_response = graph_app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(graph_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(graph_response.status(), StatusCode::OK);
    let graph_body = to_bytes(graph_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let graph: serde_json::Value = serde_json::from_slice(&graph_body).unwrap();

    let mut strategy_payload = sample_strategy_ir_compile_request_json();
    strategy_payload["compile_id"] =
        serde_json::Value::String("compile_strategy_ir_execution_profile_equivalence".to_string());
    strategy_payload["strategy_ir"]["signals"][0]["signal_id"] =
        serde_json::Value::String("momentum_signal".to_string());
    strategy_payload["strategy_ir"]["signals"][0]["name"] =
        serde_json::Value::String("Momentum Signal".to_string());
    strategy_payload["strategy_ir"]["signals"][0]["indicator"]["kind"] =
        serde_json::Value::String("momentum".to_string());
    strategy_payload["strategy_ir"]["signals"][0]["indicator"]["params"] =
        serde_json::json!({ "lookback": 20 });
    strategy_payload["strategy_ir"]["logic"]["entry_rules"][0]["condition"] =
        serde_json::Value::String("momentum_signal > 0.03".to_string());
    strategy_payload["strategy_ir"]["logic"]["entry_rules"][0]["action"] =
        serde_json::Value::String("open_long".to_string());
    strategy_payload["strategy_ir"]["execution_profile"] = serde_json::json!({
        "profile_id": "paper",
        "fee_bps": 12.5,
        "slippage_bps": 7.5
    });

    let strategy_response = graph_app
        .oneshot(
            Request::builder()
                .uri("/api/strategy-ir/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(strategy_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(strategy_response.status(), StatusCode::OK);
    let strategy_body = to_bytes(strategy_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let strategy: serde_json::Value = serde_json::from_slice(&strategy_body).unwrap();

    let formal_view = core_ir_execution_profile_equivalence_view(&formal["core_ir"]);
    let graph_view = core_ir_execution_profile_equivalence_view(&graph["core_ir"]);
    let strategy_view = core_ir_execution_profile_equivalence_view(&strategy["core_ir"]);

    assert_eq!(
        formal["runtime_config"]["executions"][0]["config"]["profile_id"],
        "paper"
    );
    assert_eq!(
        graph["runtime_config"]["executions"][0]["config"]["profile_id"],
        "paper"
    );
    assert_eq!(formal_view, graph_view);
    assert_eq!(formal_view, strategy_view);
}

#[tokio::test]
async fn strategy_ir_compile_endpoint_returns_structured_custom_diagnostics() {
    let app = build_app_router(test_app_state());
    let mut payload = sample_strategy_ir_compile_request_json();
    payload["strategy_ir"]["signals"][0]["indicator"]["params"]["custom_expr"]["predicate"]
        ["left"]["data_id"] = serde_json::Value::String("other_data".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/strategy-ir/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "strategy_ir_compile_failed");
    assert_eq!(value["details"][0]["code"], "CUSTOM006");
    assert_eq!(value["details"][0]["target"], "params.custom_expr");
    assert!(value["details"][0]["message"]
        .as_str()
        .unwrap()
        .contains("未声明的输入"));
    assert!(value["details"][0]["reason"]
        .as_str()
        .unwrap()
        .contains("自定义指标受限"));
}

#[tokio::test]
async fn backtest_endpoint_returns_output_artifacts() {
    let app = build_app_router(test_app_state());
    let mut payload = sample_compile_request_json();
    payload["backtest_options"] = serde_json::json!({
        "replay_source": "deterministic_mock"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        value["backtest_artifacts"]["manifest"]["backtest_spec"]["schema_version"],
        "quantpilot/backtest-spec/v1"
    );
    assert_eq!(
        value["backtest_artifacts"]["manifest"]["backtest_spec"]["replay_source"],
        "deterministic_mock"
    );
    assert_eq!(
        value["backtest_artifacts"]["manifest"]["backtest_spec"]["run_spec"]["config_hash"],
        value["config_hash"]
    );
    assert_eq!(
        value["backtest_artifacts"]["manifest"]["compile_artifacts"]["compile"]["config_hash"],
        value["config_hash"]
    );
    assert_eq!(
        value["backtest_artifacts"]["manifest"]["compile_artifacts"]["core_ir"]["digest"]["value"],
        value["backtest_artifacts"]["manifest"]["backtest_spec"]["run_spec"]["core_ir_digest"]
            ["value"]
    );
    assert!(value["backtest_artifacts"]["metrics"]["summary"].is_object());
    assert!(value["backtest_artifacts"]["manifest"]["output_artifacts"].is_array());
}

#[tokio::test]
async fn runtime_run_is_persisted_only_after_save() {
    let base = test_storage_base("run-save-gate");
    let graph_dir = base.join("graphs");
    let run_dir = base.join("runs");
    let backtest_dir = base.join("backtests");
    std::fs::create_dir_all(&graph_dir).unwrap();
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&backtest_dir).unwrap();
    let app = build_app_router(test_app_state_from_dirs(
        base,
        graph_dir,
        run_dir.clone(),
        backtest_dir,
    ));
    let payload = sample_compile_request_json();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/test-run")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let run_id = created["run_id"].as_str().unwrap().to_string();
    let run_path = run_dir.join(format!("{run_id}.json"));

    assert!(!run_path.exists());

    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}/save"))
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(save_response.status(), StatusCode::OK);
    assert!(run_path.exists());
}

#[tokio::test]
async fn runtime_run_can_be_discarded_only_before_save() {
    let base = test_storage_base("run-discard");
    let graph_dir = base.join("graphs");
    let run_dir = base.join("runs");
    let backtest_dir = base.join("backtests");
    std::fs::create_dir_all(&graph_dir).unwrap();
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&backtest_dir).unwrap();
    let app = build_app_router(test_app_state_from_dirs(
        base,
        graph_dir,
        run_dir.clone(),
        backtest_dir,
    ));
    let payload = sample_compile_request_json();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/test-run")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let started: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let run_id = started["run_id"].as_str().unwrap().to_string();

    let discard_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}"))
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(discard_response.status(), StatusCode::OK);

    let detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{run_id}"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail_response.status(), StatusCode::NOT_FOUND);

    let saved_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/test-run")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let saved_body = to_bytes(saved_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&saved_body).unwrap();
    let saved_run_id = saved["run_id"].as_str().unwrap().to_string();
    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{saved_run_id}/save"))
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_response.status(), StatusCode::OK);

    let discard_saved_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/runs/{saved_run_id}"))
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(discard_saved_response.status(), StatusCode::CONFLICT);
    assert!(run_dir.join(format!("{saved_run_id}.json")).exists());
}

#[tokio::test]
async fn backtest_detail_can_be_reloaded_from_artifact_directory() {
    let base = test_storage_base("backtest-artifacts");
    let graph_dir = base.join("graphs");
    let run_dir = base.join("runs");
    let backtest_dir = base.join("backtests");
    std::fs::create_dir_all(&graph_dir).unwrap();
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&backtest_dir).unwrap();

    let app = build_app_router(test_app_state_from_dirs(
        base,
        graph_dir.clone(),
        run_dir.clone(),
        backtest_dir.clone(),
    ));
    let mut payload = sample_compile_request_json();
    payload["backtest_options"] = serde_json::json!({
        "replay_source": "deterministic_mock"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let backtest_id = created["backtest_id"].as_str().unwrap().to_string();

    assert!(!backtest_dir.join(&backtest_id).exists());

    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/backtests/{backtest_id}/save"))
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(save_response.status(), StatusCode::OK);

    assert!(backtest_dir
        .join(&backtest_id)
        .join("manifest.json")
        .exists());
    assert!(backtest_dir
        .join(&backtest_id)
        .join("trade_ledger.json")
        .exists());
    assert!(backtest_dir
        .join(&backtest_id)
        .join("equity_curve.json")
        .exists());
    assert!(std::fs::read_dir(&backtest_dir)
        .unwrap()
        .all(|entry| !is_backtest_promotion_work_dir(&entry.unwrap().path())));

    let second_save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/backtests/{backtest_id}/save"))
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(second_save_response.status(), StatusCode::OK);
    assert!(backtest_dir
        .join(&backtest_id)
        .join("manifest.json")
        .exists());
    assert!(std::fs::read_dir(&backtest_dir)
        .unwrap()
        .all(|entry| !is_backtest_promotion_work_dir(&entry.unwrap().path())));

    let fresh_app = build_app_router(new_app_state(graph_dir, run_dir, backtest_dir));
    let detail_response = fresh_app
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/backtests/{backtest_id}"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_body = to_bytes(detail_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let detail: serde_json::Value = serde_json::from_slice(&detail_body).unwrap();

    assert_eq!(detail["backtest_id"], backtest_id);
    assert!(detail["backtest_artifacts"]["trade_ledger"]["trades"].is_array());
    assert!(detail["backtest_artifacts"]["equity_curve"]["points"].is_array());
    assert_eq!(
        detail["backtest_artifacts"]["metrics"]["summary"]["final_equity"],
        created["backtest_artifacts"]["metrics"]["summary"]["final_equity"]
    );
}

#[tokio::test]
async fn large_transient_backtest_spills_to_temp_until_save() {
    let base = test_storage_base("backtest-transient-spill");
    let graph_dir = base.join("graphs");
    let run_dir = base.join("runs");
    let backtest_dir = base.join("backtests");
    std::fs::create_dir_all(&graph_dir).unwrap();
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&backtest_dir).unwrap();

    let mut state = test_app_state_from_dirs(base, graph_dir, run_dir, backtest_dir.clone());
    state.transient_backtest_spill_threshold_bytes = 1;
    let transient_dir = state.transient_backtest_store_dir.as_ref().clone();
    let app = build_app_router(state);
    let mut payload = sample_compile_request_json();
    payload["backtest_options"] = serde_json::json!({
        "replay_source": "deterministic_mock"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let backtest_id = created["backtest_id"].as_str().unwrap().to_string();

    assert!(!backtest_dir.join(&backtest_id).exists());
    assert!(std::fs::read_dir(&transient_dir)
        .unwrap()
        .any(|entry| entry.unwrap().path().is_dir()));

    let detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/backtests/{backtest_id}"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_body = to_bytes(detail_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let detail: serde_json::Value = serde_json::from_slice(&detail_body).unwrap();
    let mut expected_loaded_governance =
        created["backtest_artifacts"]["manifest"]["governance"].clone();
    expected_loaded_governance["governance_source"] =
        serde_json::Value::String("loaded_manifest".to_string());
    assert_eq!(detail["governance"], expected_loaded_governance);
    assert_eq!(
        detail["backtest_artifacts"]["manifest"]["governance"],
        detail["governance"]
    );

    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/backtests/{backtest_id}/save"))
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(save_response.status(), StatusCode::OK);
    let save_body = to_bytes(save_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&save_body).unwrap();
    assert_eq!(saved["governance"], detail["governance"]);
    assert_eq!(
        saved["backtest_artifacts"]["manifest"]["governance"],
        detail["governance"]
    );
    assert!(backtest_dir
        .join(&backtest_id)
        .join("manifest.json")
        .exists());
    assert!(std::fs::read_dir(&transient_dir)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(true));
}

#[tokio::test]
async fn backtest_can_be_discarded_only_before_save() {
    let base = test_storage_base("backtest-discard");
    let graph_dir = base.join("graphs");
    let run_dir = base.join("runs");
    let backtest_dir = base.join("backtests");
    std::fs::create_dir_all(&graph_dir).unwrap();
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&backtest_dir).unwrap();

    let mut state = test_app_state_from_dirs(base, graph_dir, run_dir, backtest_dir.clone());
    state.transient_backtest_spill_threshold_bytes = 1;
    let transient_dir = state.transient_backtest_store_dir.as_ref().clone();
    let app = build_app_router(state);
    let mut payload = sample_compile_request_json();
    payload["backtest_options"] = serde_json::json!({
        "replay_source": "deterministic_mock"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let backtest_id = created["backtest_id"].as_str().unwrap().to_string();

    let discard_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/backtests/{backtest_id}"))
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(discard_response.status(), StatusCode::OK);
    assert!(std::fs::read_dir(&transient_dir)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(true));

    let detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/backtests/{backtest_id}"))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail_response.status(), StatusCode::NOT_FOUND);

    let saved_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runtime/backtest")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let saved_body = to_bytes(saved_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&saved_body).unwrap();
    let saved_backtest_id = saved["backtest_id"].as_str().unwrap().to_string();
    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/backtests/{saved_backtest_id}/save"))
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_response.status(), StatusCode::OK);

    let discard_saved_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/runtime/backtests/{saved_backtest_id}"))
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(discard_saved_response.status(), StatusCode::CONFLICT);
    assert!(backtest_dir
        .join(&saved_backtest_id)
        .join("manifest.json")
        .exists());
}

#[tokio::test]
async fn graphs_endpoint_lists_saved_graph_files_only() {
    let base = test_storage_base("graph-index");
    let graph_dir = base.join("graphs");
    let run_dir = base.join("runs");
    let backtest_dir = base.join("backtests");
    std::fs::create_dir_all(&graph_dir).unwrap();
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&backtest_dir).unwrap();

    std::fs::write(
            graph_dir.join("alpha_strategy.json"),
            serde_json::json!({
                "metadata": {
                    "graph_id": "alpha_strategy",
                    "name": "Alpha strategy",
                    "updated_at": 1710000000000u64,
                    "artifacts": {
                        "quantscript": {
                            "saved_path": graph_dir.join("alpha_strategy.qs").to_string_lossy().to_string()
                        }
                    }
                }
            })
            .to_string(),
        )
        .unwrap();
    std::fs::write(
        graph_dir.join("alpha_strategy.qs"),
        "strategy_graph alpha_strategy {}",
    )
    .unwrap();
    std::fs::write(
            graph_dir.join("beta_strategy.json"),
            serde_json::json!({
                "metadata": {
                    "graph_id": "beta_strategy",
                    "name": "Beta strategy",
                    "updated_at": 1710000200000u64,
                    "artifacts": {
                        "quantscript": {
                            "saved_path": graph_dir.join("beta_strategy.qs").to_string_lossy().to_string()
                        }
                    }
                }
            })
            .to_string(),
        )
        .unwrap();
    std::fs::write(
        graph_dir.join("beta_strategy.qs"),
        "strategy_graph beta_strategy {}",
    )
    .unwrap();
    std::fs::write(
        graph_dir.join("latest.json"),
        serde_json::json!({
            "metadata": {
                "graph_id": "latest_shadow",
                "name": "Latest shadow",
                "updated_at": 1710000300000u64
            }
        })
        .to_string(),
    )
    .unwrap();

    let app = build_app_router(test_app_state_from_dirs(
        base,
        graph_dir,
        run_dir,
        backtest_dir,
    ));
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/graphs")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let items = value["data"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["graph_id"], "beta_strategy");
    assert_eq!(items[1]["graph_id"], "alpha_strategy");
    assert!(items[0]["path"]
        .as_str()
        .unwrap()
        .ends_with("beta_strategy.qs"));
    assert!(items[1]["path"]
        .as_str()
        .unwrap()
        .ends_with("alpha_strategy.qs"));
    assert!(items
        .iter()
        .all(|entry| entry["graph_id"] != "latest_shadow"));
}

#[tokio::test]
async fn delete_graph_endpoint_removes_strategy_files_and_refreshes_latest() {
    let base = test_storage_base("graph-delete");
    let graph_dir = base.join("graphs");
    let run_dir = base.join("runs");
    let backtest_dir = base.join("backtests");
    std::fs::create_dir_all(&graph_dir).unwrap();
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&backtest_dir).unwrap();
    std::fs::create_dir_all(graph_dir.join("versions").join("alpha_strategy")).unwrap();

    let alpha = serde_json::json!({
        "metadata": {
            "graph_id": "alpha_strategy",
            "name": "Alpha strategy",
            "updated_at": 1710000000000u64,
            "collaboration": {
                "owner": {
                    "actor_id": "previous_operator",
                    "display_name": "Previous operator"
                }
            }
        },
        "nodes": [],
        "edges": []
    });
    let beta = serde_json::json!({
        "metadata": {
            "graph_id": "beta_strategy",
            "name": "Beta strategy",
            "updated_at": 1710000200000u64
        },
        "nodes": [],
        "edges": []
    });
    std::fs::write(graph_dir.join("alpha_strategy.json"), alpha.to_string()).unwrap();
    std::fs::write(graph_dir.join("alpha_strategy.qs"), "strategy alpha() {}").unwrap();
    std::fs::write(
        graph_dir
            .join("versions")
            .join("alpha_strategy")
            .join("1710000000000.json"),
        alpha.to_string(),
    )
    .unwrap();
    std::fs::write(graph_dir.join("beta_strategy.json"), beta.to_string()).unwrap();
    std::fs::write(graph_dir.join("beta_strategy.qs"), "strategy beta() {}").unwrap();
    std::fs::write(graph_dir.join("latest.json"), alpha.to_string()).unwrap();

    let app = build_app_router(test_app_state_from_dirs(
        base,
        graph_dir.clone(),
        run_dir,
        backtest_dir,
    ));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/graphs/alpha_strategy")
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(!graph_dir.join("alpha_strategy.json").exists());
    assert!(!graph_dir.join("alpha_strategy.qs").exists());
    assert!(!graph_dir.join("versions").join("alpha_strategy").exists());
    assert!(graph_dir.join("beta_strategy.json").exists());

    let latest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(graph_dir.join("latest.json")).unwrap())
            .unwrap();
    assert_eq!(latest["metadata"]["graph_id"], "beta_strategy");

    let list_response = app
        .oneshot(
            Request::builder()
                .uri("/api/graphs")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let items = value["data"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["graph_id"], "beta_strategy");
}

#[tokio::test]
async fn delete_graph_endpoint_returns_not_found_for_missing_graph() {
    let base = test_storage_base("graph-delete-missing");
    let graph_dir = base.join("graphs");
    let run_dir = base.join("runs");
    let backtest_dir = base.join("backtests");
    std::fs::create_dir_all(&graph_dir).unwrap();
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&backtest_dir).unwrap();

    let app = build_app_router(test_app_state_from_dirs(
        base,
        graph_dir,
        run_dir,
        backtest_dir,
    ));
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/graphs/missing_strategy")
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["deleted"], false);
}

#[tokio::test]
async fn graph_version_endpoints_list_load_and_restore_versions() {
    let base = test_storage_base("graph-versions");
    let graph_dir = base.join("graphs");
    let run_dir = base.join("runs");
    let backtest_dir = base.join("backtests");
    std::fs::create_dir_all(&graph_dir).unwrap();
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&backtest_dir).unwrap();

    let app = build_app_router(test_app_state_from_dirs(
        base,
        graph_dir.clone(),
        run_dir,
        backtest_dir,
    ));
    let graph_v1 = serde_json::json!({
        "metadata": {
            "graph_id": "versioned_strategy",
            "name": "Versioned Strategy V1",
            "version": "1.0.0"
        },
        "nodes": [
            {
                "id": "data_feed",
                "type": "data",
                "module_key": "builtin.data.kline",
                "name": "Price Feed",
                "config": {
                    "window_size": 20
                }
            }
        ],
        "edges": []
    });
    let graph_v2 = serde_json::json!({
        "metadata": {
            "graph_id": "versioned_strategy",
            "name": "Versioned Strategy V2",
            "version": "1.0.0"
        },
        "nodes": [
            {
                "id": "data_feed",
                "type": "data",
                "module_key": "builtin.data.kline",
                "name": "Price Feed",
                "config": {
                    "window_size": 55
                }
            }
        ],
        "edges": []
    });

    let save_v1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/graphs/save")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "graph": graph_v1 }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_v1.status(), StatusCode::OK);
    let save_v1_body = to_bytes(save_v1.into_body(), usize::MAX).await.unwrap();
    let save_v1_value: serde_json::Value = serde_json::from_slice(&save_v1_body).unwrap();
    let version_v1 = save_v1_value["version_id"].as_str().unwrap().to_string();

    sleep(Duration::from_millis(5)).await;

    let save_v2 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/graphs/save")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "graph": graph_v2 }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_v2.status(), StatusCode::OK);
    let save_v2_body = to_bytes(save_v2.into_body(), usize::MAX).await.unwrap();
    let save_v2_value: serde_json::Value = serde_json::from_slice(&save_v2_body).unwrap();
    let version_v2 = save_v2_value["version_id"].as_str().unwrap().to_string();
    assert_ne!(version_v1, version_v2);

    let versions_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/graphs/versioned_strategy/versions")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(versions_response.status(), StatusCode::OK);
    let versions_body = to_bytes(versions_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let versions: serde_json::Value = serde_json::from_slice(&versions_body).unwrap();
    assert_eq!(versions.as_array().unwrap().len(), 2);
    assert_eq!(versions[0]["version_id"], version_v2);
    assert_eq!(versions[0]["is_latest"], true);
    assert_eq!(versions[1]["version_id"], version_v1);
    assert!(versions[0]["path"]
        .as_str()
        .unwrap()
        .contains("\\versions\\"));

    let old_version_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/graphs/versioned_strategy/versions/{version_v1}"
                ))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(old_version_response.status(), StatusCode::OK);
    let old_version_body = to_bytes(old_version_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let old_version: serde_json::Value = serde_json::from_slice(&old_version_body).unwrap();
    assert_eq!(old_version["metadata"]["name"], "Versioned Strategy V1");
    assert_eq!(old_version["nodes"][0]["config"]["window_size"], 20);

    let compare_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/graphs/versioned_strategy/versions/compare/{version_v1}/{version_v2}"
                ))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(compare_response.status(), StatusCode::OK);
    let compare_body = to_bytes(compare_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let compare: serde_json::Value = serde_json::from_slice(&compare_body).unwrap();
    assert_eq!(
        compare["strategy_config_diff"]["schema_version"],
        "quantpilot/v4-strategy-config-diff/v1"
    );
    assert_eq!(
        compare["strategy_config_diff"]["source_digest_changes"][0]["field"],
        "graph_digest"
    );
    assert!(compare["strategy_config_diff"]["domain_changes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|change| change["domain_id"] == "market"));
    assert_eq!(
        compare["strategy_config_evidence_diff"]["schema_version"],
        "quantpilot/v4-strategy-config-evidence-diff/v1"
    );
    assert_eq!(
        compare["strategy_config_evidence_diff"]["status"],
        "missing"
    );

    sleep(Duration::from_millis(5)).await;

    let restore_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/graphs/versioned_strategy/versions/{version_v1}/restore"
                ))
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(restore_response.status(), StatusCode::OK);

    let latest_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/graphs/versioned_strategy")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(latest_response.status(), StatusCode::OK);
    let latest_body = to_bytes(latest_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let latest: serde_json::Value = serde_json::from_slice(&latest_body).unwrap();
    assert_eq!(latest["metadata"]["name"], "Versioned Strategy V1");
    assert_eq!(latest["nodes"][0]["config"]["window_size"], 20);

    let versions_after_restore_response = app
        .oneshot(
            Request::builder()
                .uri("/api/graphs/versioned_strategy/versions")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(versions_after_restore_response.status(), StatusCode::OK);
    let versions_after_restore_body =
        to_bytes(versions_after_restore_response.into_body(), usize::MAX)
            .await
            .unwrap();
    let versions_after_restore: serde_json::Value =
        serde_json::from_slice(&versions_after_restore_body).unwrap();
    assert_eq!(versions_after_restore.as_array().unwrap().len(), 3);
    assert_eq!(versions_after_restore[0]["is_latest"], true);
}

#[tokio::test]
async fn reveal_graph_endpoint_returns_not_found_for_missing_graph() {
    let base = test_storage_base("graph-reveal");
    let graph_dir = base.join("graphs");
    let run_dir = base.join("runs");
    let backtest_dir = base.join("backtests");
    std::fs::create_dir_all(&graph_dir).unwrap();
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&backtest_dir).unwrap();

    let app = build_app_router(test_app_state_from_dirs(
        base,
        graph_dir,
        run_dir,
        backtest_dir,
    ));
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/graphs/missing_strategy/reveal")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn reveal_graph_path_prefers_existing_quantscript_and_returns_absolute_path() {
    let base = test_storage_base("graph-reveal-path");
    let graph_dir = base.join("graphs");
    std::fs::create_dir_all(&graph_dir).unwrap();
    let graph_path = graph_dir.join("alpha_strategy.json");
    let quantscript_path = graph_dir.join("alpha_strategy.qs");
    std::fs::write(&graph_path, "{}").unwrap();
    std::fs::write(&quantscript_path, "strategy alpha() {}").unwrap();

    let graph = serde_json::json!({
        "metadata": {
            "artifacts": {
                "quantscript": {
                    "saved_path": quantscript_path.to_string_lossy()
                }
            }
        }
    });
    let reveal_path = graph_api::resolve_graph_reveal_path_from_value(&graph, &graph_path)
        .await
        .unwrap();

    assert!(reveal_path.is_absolute());
    assert_eq!(
        reveal_path,
        std::fs::canonicalize(&quantscript_path).unwrap()
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_success() {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_test",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "runtime_targets": {
            "source_to_node": {
                "data_data_feed": "data_feed",
                "intent_intent_rsi": "intent_rsi",
                "agent_script_main": "agent_main",
                "risk_script_global": "risk_main"
            },
            "runtime_node_id": "runtime_main",
            "execution_node_id": "execution_main"
        },
        "source": r#"
fn strategy() {
    let data_data_feed_series = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let intent_intent_rsi_signal = rsi(data_data_feed_series, 14)
    if intent_intent_rsi_signal < 30 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if intent_intent_rsi_signal > 70 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["graph_id"], "graph_test");
    assert_eq!(value["compile_id"], "compile_formal_test");
    assert_eq!(value["compilable"], true);
    assert_eq!(value["counts"]["data_sources"], 1);
    assert_eq!(
        value["core_ir"]["metadata"]["source_kind"],
        "formal_quant_script"
    );
    assert_eq!(value["core_ir"]["metadata"]["strategy_id"], "graph_test");
    assert_eq!(value["core_ir"]["metadata"]["name"], "Test Graph");
    assert_eq!(
        value["runtime_config"]["metadata"]["compile_id"],
        "compile_formal_test"
    );
    assert_eq!(
        value["runtime_config"]["data_sources"][0]["id"],
        "data_feed"
    );
    assert_eq!(
        value["runtime_config"]["intent_generators"][0]["id"],
        "intent_rsi"
    );
    assert_eq!(
        value["runtime_targets"]["source_to_node"]["data_data_feed"],
        "data_feed"
    );
    assert_eq!(
        value["runtime_targets"]["source_to_node"]["intent_intent_rsi"],
        "intent_rsi"
    );
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["kind"],
        "raw_text"
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_direct_ma_compare_golden_view() {
    let value = compile_formal_quantscript_for_test(
        r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(data_feed, 20)
    let slow = sma(data_feed, 100)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_ma_golden",
    )
    .await;

    assert_eq!(
        formal_compile_golden_view(&value),
        serde_json::json!({
            "core_source_kind": "formal_quant_script",
            "data_bindings": [
                {
                    "data_id": "script_okx_btcusdt_1d",
                    "kind": "kline_series",
                    "source_hints": {
                        "exchange": "okx",
                        "symbol": "BTCUSDT",
                        "timeframe": "1d"
                    }
                }
            ],
            "indicator_kinds": ["ma_cross"],
            "signal_rules": [
                {
                    "signal_id": "intent_btcusdt_ma_entry_signal",
                    "signal_kind": "long",
                    "indicator_id": "intent_btcusdt_ma_entry",
                    "condition": {
                        "kind": "raw_text",
                        "source": "ma_cross(fast=20, slow=100, entry_ratio=0.2)"
                    }
                }
            ],
            "agent_policy_kinds": ["weighted_signals"],
            "runtime_projection": {
                "data_modules": ["builtin.data.kline"],
                "intent_modules": ["builtin.intent.double_ma"],
                "agent_modules": ["builtin.agent.weighted"],
                "risk_modules": ["builtin.risk.global"],
                "execution_modules": ["builtin.execution.paper"],
                "runtime_module": "builtin.runtime.control"
            }
        })
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_one_sided_rsi_golden_view() {
    let value = compile_formal_quantscript_for_test(
        r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let r = rsi(data_feed, 14)
    if r < 25 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_rsi_golden",
    )
    .await;

    assert_eq!(
        formal_compile_golden_view(&value),
        serde_json::json!({
            "core_source_kind": "formal_quant_script",
            "data_bindings": [
                {
                    "data_id": "script_okx_btcusdt_1d",
                    "kind": "kline_series",
                    "source_hints": {
                        "exchange": "okx",
                        "symbol": "BTCUSDT",
                        "timeframe": "1d"
                    }
                }
            ],
            "indicator_kinds": ["rsi"],
            "signal_rules": [
                {
                    "signal_id": "intent_btcusdt_rsi_signal",
                    "signal_kind": "long",
                    "indicator_id": "intent_btcusdt_rsi",
                    "condition": {
                        "kind": "compare",
                        "left": {
                            "kind": "ref",
                            "name": "intent_btcusdt_rsi"
                        },
                        "op": "lt",
                        "right": {
                            "kind": "number",
                            "value": 25.0
                        }
                    }
                }
            ],
            "agent_policy_kinds": ["weighted_signals"],
            "runtime_projection": {
                "data_modules": ["builtin.data.kline"],
                "intent_modules": ["builtin.intent.rsi"],
                "agent_modules": ["builtin.agent.weighted"],
                "risk_modules": ["builtin.risk.global"],
                "execution_modules": ["builtin.execution.paper"],
                "runtime_module": "builtin.runtime.control"
            }
        })
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_one_sided_momentum_golden_view() {
    let value = compile_formal_quantscript_for_test(
        r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_momentum_golden",
    )
    .await;

    assert_eq!(
        formal_compile_golden_view(&value),
        serde_json::json!({
            "core_source_kind": "formal_quant_script",
            "data_bindings": [
                {
                    "data_id": "script_okx_btcusdt_1d",
                    "kind": "kline_series",
                    "source_hints": {
                        "exchange": "okx",
                        "symbol": "BTCUSDT",
                        "timeframe": "1d"
                    }
                }
            ],
            "indicator_kinds": ["momentum"],
            "signal_rules": [
                {
                    "signal_id": "intent_btcusdt_momentum_signal",
                    "signal_kind": "long",
                    "indicator_id": "intent_btcusdt_momentum",
                    "condition": {
                        "kind": "compare",
                        "left": {
                            "kind": "ref",
                            "name": "intent_btcusdt_momentum"
                        },
                        "op": "gt",
                        "right": {
                            "kind": "number",
                            "value": 0.03
                        }
                    }
                }
            ],
            "agent_policy_kinds": ["weighted_signals"],
            "runtime_projection": {
                "data_modules": ["builtin.data.kline"],
                "intent_modules": ["builtin.intent.momentum"],
                "agent_modules": ["builtin.agent.weighted"],
                "risk_modules": ["builtin.risk.global"],
                "execution_modules": ["builtin.execution.paper"],
                "runtime_module": "builtin.runtime.control"
            }
        })
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view() {
    let value = compile_formal_quantscript_for_test(
        r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let z = zscore(data_feed, 20)
    if z < -2 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_zscore_golden",
    )
    .await;

    assert_eq!(
        formal_compile_golden_view(&value),
        serde_json::json!({
            "core_source_kind": "formal_quant_script",
            "data_bindings": [
                {
                    "data_id": "script_okx_btcusdt_1d",
                    "kind": "kline_series",
                    "source_hints": {
                        "exchange": "okx",
                        "symbol": "BTCUSDT",
                        "timeframe": "1d"
                    }
                }
            ],
            "indicator_kinds": ["z_score"],
            "signal_rules": [
                {
                    "signal_id": "intent_btcusdt_zscore_signal",
                    "signal_kind": "long",
                    "indicator_id": "intent_btcusdt_zscore",
                    "condition": {
                        "kind": "compare",
                        "left": {
                            "kind": "ref",
                            "name": "intent_btcusdt_zscore"
                        },
                        "op": "lt",
                        "right": {
                            "kind": "number",
                            "value": -2.0
                        }
                    }
                }
            ],
            "agent_policy_kinds": ["weighted_signals"],
            "runtime_projection": {
                "data_modules": ["builtin.data.kline"],
                "intent_modules": ["builtin.intent.zscore"],
                "agent_modules": ["builtin.agent.weighted"],
                "risk_modules": ["builtin.risk.global"],
                "execution_modules": ["builtin.execution.paper"],
                "runtime_module": "builtin.runtime.control"
            }
        })
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_non_trunk_import_alias_diagnostic_golden_view()
{
    let value = compile_formal_quantscript_error_for_test(
        r#"
import data as market_data

fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=50)?
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#,
        "compile_formal_import_alias_golden",
    )
    .await;

    assert_eq!(
        formal_compile_error_golden_view(&value),
        serde_json::json!({
            "error": "quantscript_compile_failed",
            "detail": {
                "code": "QS0608",
                "message": "QuantScript 不支持简单的 `import foo as bar`；请使用 `from module import name as alias`",
                "reason": serde_json::Value::Null,
                "span_label": "data as market_data",
            }
        })
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_universe_helper_input_diagnostic_golden_view()
{
    let value = compile_formal_quantscript_error_for_test(
        r#"
fn strategy() {
    let selected = top(1, 2)
    rebalance(equal_weight(selected), every="1d")
}
"#,
        "compile_formal_universe_input_golden",
    )
    .await;

    assert_eq!(
        formal_compile_error_golden_view(&value),
        serde_json::json!({
            "error": "quantscript_compile_failed",
            "detail": {
                "code": "QS0610",
                "message": "策略函数必须包含至少一个 fetch() 调用来获取市场数据",
                "reason": serde_json::Value::Null,
                "span_label": "strategy",
            }
        })
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_multi_detail_non_trunk_diagnostic_golden_view()
{
    let value = compile_formal_quantscript_error_for_test(
        r#"
import data as market_data

fn helper(series) {
    return helper(series)
}

async fn strategy() {
    let closes = await fetch("BTCUSDT", interval="1d", lookback=50)?
    let unsafe_try = sma(closes, 20)?
    let mut out = []
    out.push(1)
    if fetch("BTCUSDT", interval="1d", lookback=20).retryable() {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    for value in closes[20..] {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    while closes[0] > 0 {
        match closes[0] {
            _ => emit Intent("BUY", instrument="BTCUSDT", quantity=1.0),
        }
    }
}
"#,
        "compile_formal_non_trunk_golden",
    )
    .await;

    assert_eq!(
        formal_compile_error_details_golden_view(&value),
        serde_json::json!({
            "error": "quantscript_compile_failed",
            "details": [
                {
                    "code": "QS0608",
                    "message": "QuantScript 不支持简单的 `import foo as bar`；请使用 `from module import name as alias`",
                    "span_label": "data as market_data",
                },
                {
                    "code": "QS0605",
                    "message": "QuantScript 不支持strategy() 函数体中的递归辅助调用",
                    "span_label": "helper",
                },
                {
                    "code": "QS0601",
                    "message": "QuantScript 不支持strategy() 函数体中的异步函数",
                    "span_label": "strategy",
                },
                {
                    "code": "QS0602",
                    "message": "QuantScript 不支持strategy() 函数体中的 await 表达式",
                    "span_label": "await",
                },
                {
                    "code": "QS0607",
                    "message": "QuantScript 在strategy() 函数体中仅支持对 fetch 类数据源表达式使用后缀 `?`",
                    "span_label": "?",
                },
                {
                    "code": "QS0609",
                    "message": "QuantScript 不支持strategy() 函数体中使用 `.push(...)` 构建可变列表",
                    "span_label": ".push",
                },
                {
                    "code": "QS0610",
                    "message": "QuantScript 不支持strategy() 函数体中的 `.ok()` / `.retryable()` 辅助方法",
                    "span_label": "retryable",
                },
                {
                    "code": "QS0603",
                    "message": "QuantScript 不支持 strategy() 函数体中的 while 循环。请改用 for ... in ... 或在数据源上使用窗口聚合",
                    "span_label": "strategy",
                },
                {
                    "code": "QS0604",
                    "message": "QuantScript 不支持strategy() 函数体中的 match 语句",
                    "span_label": "strategy",
                },
                {
                    "code": "QS0606",
                    "message": "QuantScript 在strategy() 函数体中仅支持对 Universe 的 for 循环",
                    "span_label": "for",
                }
            ],
        })
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_multi_detail_lowering_diagnostic_golden_view()
{
    let value = compile_formal_quantscript_error_for_test(
        r#"
fn strategy() {
    emit Intent("HOLD", instrument="BTCUSDT", quantity=1.0)
    emit Intent("", instrument="BTCUSDT", quantity=1.0)
}
"#,
        "compile_formal_lowering_multi_golden",
    )
    .await;

    assert_eq!(
        formal_compile_error_details_golden_view(&value),
        serde_json::json!({
            "error": "quantscript_compile_failed",
            "details": [
                {
                    "code": "QS0610",
                    "message": "策略函数必须包含至少一个 fetch() 调用来获取市场数据",
                    "span_label": "strategy",
                }
            ],
        })
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_matches_mixed_boundary_diagnostic_golden_view() {
    let value = compile_formal_quantscript_error_for_test(
        r#"
import data as market_data

fn strategy() {
    emit Intent("HOLD", instrument="BTCUSDT", quantity=1.0)
    emit Intent("", instrument="BTCUSDT", quantity=1.0)
}
"#,
        "compile_formal_mixed_boundary_golden",
    )
    .await;

    assert_eq!(
        formal_compile_error_details_golden_view(&value),
        serde_json::json!({
            "error": "quantscript_compile_failed",
            "details": [
                {
                    "code": "QS0608",
                    "message": "QuantScript 不支持简单的 `import foo as bar`；请使用 `from module import name as alias`",
                    "span_label": "data as market_data",
                },
                {
                    "code": "QS0610",
                    "message": "策略函数必须包含至少一个 fetch() 调用来获取市场数据",
                    "span_label": "strategy",
                }
            ],
        })
    );
}

#[tokio::test]
async fn formal_quantscript_text_to_core_ir_to_graph_round_trip_sample() {
    let source = r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(data_feed, 20)
    let slow = sma(data_feed, 100)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;
    let value =
        compile_formal_quantscript_for_test(source, "compile_formal_round_trip_sample").await;

    let graph = graph_value_from_runtime_config(&value["runtime_config"], source);
    let generated = generate_quantscript_from_graph_value(&graph).unwrap();
    let reparsed = parse_graph_quantscript_source(&generated).unwrap();
    let regenerated = generate_quantscript_from_graph_value(&reparsed).unwrap();
    let module_keys: Vec<&str> = reparsed["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|node| node["module_key"].as_str())
        .collect();

    assert_eq!(
        value["core_ir"]["metadata"]["source_kind"],
        "formal_quant_script"
    );
    assert!(generated.starts_with("strategy_graph graph_test {"));
    assert!(regenerated.starts_with("strategy_graph graph_test {"));
    assert_eq!(reparsed["metadata"]["graph_id"], "graph_test");
    assert_eq!(reparsed["nodes"].as_array().unwrap().len(), 6);
    assert_eq!(reparsed["edges"].as_array().unwrap().len(), 5);
    assert!(module_keys.contains(&"builtin.data.kline"));
    assert!(module_keys.contains(&"builtin.intent.double_ma"));
    assert!(module_keys.contains(&"builtin.agent.weighted"));
    assert!(module_keys.contains(&"builtin.risk.global"));
    assert!(module_keys.contains(&"builtin.execution.paper"));
    assert!(module_keys.contains(&"builtin.runtime.control"));
}

#[tokio::test]
async fn formal_quantscript_text_to_core_ir_to_graph_round_trip_momentum_sample() {
    let source = r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;
    let value =
        compile_formal_quantscript_for_test(source, "compile_formal_round_trip_momentum_sample")
            .await;

    let graph = graph_value_from_runtime_config(&value["runtime_config"], source);
    let generated = generate_quantscript_from_graph_value(&graph).unwrap();
    let reparsed = parse_graph_quantscript_source(&generated).unwrap();
    let regenerated = generate_quantscript_from_graph_value(&reparsed).unwrap();
    let module_keys: Vec<&str> = reparsed["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|node| node["module_key"].as_str())
        .collect();

    assert_eq!(
        value["core_ir"]["metadata"]["source_kind"],
        "formal_quant_script"
    );
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["left"]["name"],
        "intent_btcusdt_momentum"
    );
    assert!(generated.starts_with("strategy_graph graph_test {"));
    assert!(regenerated.starts_with("strategy_graph graph_test {"));
    assert_eq!(reparsed["metadata"]["graph_id"], "graph_test");
    assert_eq!(reparsed["nodes"].as_array().unwrap().len(), 6);
    assert_eq!(reparsed["edges"].as_array().unwrap().len(), 5);
    assert!(module_keys.contains(&"builtin.data.kline"));
    assert!(module_keys.contains(&"builtin.intent.momentum"));
    assert!(module_keys.contains(&"builtin.agent.weighted"));
    assert!(module_keys.contains(&"builtin.risk.global"));
    assert!(module_keys.contains(&"builtin.execution.paper"));
    assert!(module_keys.contains(&"builtin.runtime.control"));
}

#[tokio::test]
async fn formal_quantscript_text_to_core_ir_to_graph_round_trip_rsi_sample() {
    let source = r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let r = rsi(data_feed, 14)
    if r < 25 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;
    let value =
        compile_formal_quantscript_for_test(source, "compile_formal_round_trip_rsi_sample").await;

    let graph = graph_value_from_runtime_config(&value["runtime_config"], source);
    let generated = generate_quantscript_from_graph_value(&graph).unwrap();
    let reparsed = parse_graph_quantscript_source(&generated).unwrap();
    let regenerated = generate_quantscript_from_graph_value(&reparsed).unwrap();
    let module_keys: Vec<&str> = reparsed["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|node| node["module_key"].as_str())
        .collect();

    assert_eq!(
        value["core_ir"]["metadata"]["source_kind"],
        "formal_quant_script"
    );
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["left"]["name"],
        "intent_btcusdt_rsi"
    );
    assert!(generated.starts_with("strategy_graph graph_test {"));
    assert!(regenerated.starts_with("strategy_graph graph_test {"));
    assert_eq!(reparsed["metadata"]["graph_id"], "graph_test");
    assert_eq!(reparsed["nodes"].as_array().unwrap().len(), 6);
    assert_eq!(reparsed["edges"].as_array().unwrap().len(), 5);
    assert!(module_keys.contains(&"builtin.data.kline"));
    assert!(module_keys.contains(&"builtin.intent.rsi"));
    assert!(module_keys.contains(&"builtin.agent.weighted"));
    assert!(module_keys.contains(&"builtin.risk.global"));
    assert!(module_keys.contains(&"builtin.execution.paper"));
    assert!(module_keys.contains(&"builtin.runtime.control"));
}

#[tokio::test]
async fn formal_quantscript_text_to_core_ir_to_graph_round_trip_zscore_sample() {
    let source = r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let z = zscore(data_feed, 20)
    if z < -2 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;
    let value =
        compile_formal_quantscript_for_test(source, "compile_formal_round_trip_zscore_sample")
            .await;

    let graph = graph_value_from_runtime_config(&value["runtime_config"], source);
    let generated = generate_quantscript_from_graph_value(&graph).unwrap();
    let reparsed = parse_graph_quantscript_source(&generated).unwrap();
    let regenerated = generate_quantscript_from_graph_value(&reparsed).unwrap();
    let module_keys: Vec<&str> = reparsed["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|node| node["module_key"].as_str())
        .collect();

    assert_eq!(
        value["core_ir"]["metadata"]["source_kind"],
        "formal_quant_script"
    );
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["left"]["name"],
        "intent_btcusdt_zscore"
    );
    assert!(generated.starts_with("strategy_graph graph_test {"));
    assert!(regenerated.starts_with("strategy_graph graph_test {"));
    assert_eq!(reparsed["metadata"]["graph_id"], "graph_test");
    assert_eq!(reparsed["nodes"].as_array().unwrap().len(), 6);
    assert_eq!(reparsed["edges"].as_array().unwrap().len(), 5);
    assert!(module_keys.contains(&"builtin.data.kline"));
    assert!(module_keys.contains(&"builtin.intent.zscore"));
    assert!(module_keys.contains(&"builtin.agent.weighted"));
    assert!(module_keys.contains(&"builtin.risk.global"));
    assert!(module_keys.contains(&"builtin.execution.paper"));
    assert!(module_keys.contains(&"builtin.runtime.control"));
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_lowers_direct_ma_compare_to_structured_core_ir_condition(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_ma_compare",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(data_feed, 20)
    let slow = sma(data_feed, 100)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["kind"],
        "raw_text"
    );
    assert!(value["core_ir"]["signal_rules"][0]["condition"]["source"]
        .as_str()
        .unwrap_or("")
        .contains("ma_cross"));
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_lowers_one_sided_rsi_compare_to_structured_core_ir_condition(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_rsi_compare",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let r = rsi(data_feed, 14)
    if r < 25 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["kind"],
        "compare"
    );
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["left"]["kind"],
        "ref"
    );
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["left"]["name"],
        "intent_btcusdt_rsi"
    );
    assert_eq!(value["core_ir"]["signal_rules"][0]["condition"]["op"], "lt");
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["right"]["kind"],
        "number"
    );
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["right"]["value"],
        25.0
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_lowers_one_sided_momentum_compare_to_structured_core_ir_condition(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_momentum_compare",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["kind"],
        "compare"
    );
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["left"]["name"],
        "intent_btcusdt_momentum"
    );
    assert_eq!(value["core_ir"]["signal_rules"][0]["condition"]["op"], "gt");
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["right"]["value"],
        0.03
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_keeps_dual_sided_momentum_compare_on_raw_text_path() {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_momentum_dual",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(data_feed, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if m < -0.03 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["kind"],
        "raw_text"
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_lowers_one_sided_zscore_compare_to_structured_core_ir_condition(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_zscore_compare",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let z = zscore(data_feed, 20)
    if z < -2 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["kind"],
        "compare"
    );
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["left"]["name"],
        "intent_btcusdt_zscore"
    );
    assert_eq!(value["core_ir"]["signal_rules"][0]["condition"]["op"], "lt");
    assert_eq!(
        value["core_ir"]["signal_rules"][0]["condition"]["right"]["value"],
        -2.0
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_lowers_equal_weight_rebalance_helper() {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_rebalance",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="1d")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)
        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["compilable"], true);
    assert_eq!(
        value["artifacts"]["core_ir"]["core_ir"]["agent_policies"][0]["kind"],
        "portfolio_rebalance"
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_emits_authoring_view_for_headered_sections() {
    let value = compile_formal_quantscript_for_test(
        r#"fn strategy() {
    # risk
    risk.profile("global", max_position=0.35)
    # execution
    execution.profile("paper", fee_bps=12.5, slippage_bps=7.5)
    # data
    let closes = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 100)
    # intent
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    # agent
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="1d")
}
"#,
        "compile_formal_authoring_view_headers",
    )
    .await;

    let authoring_view = formal_compile_authoring_view(&value);
    assert_eq!(authoring_view["kind"], "quantscript_authoring_view");
    assert_eq!(
        authoring_view["source_order"],
        serde_json::json!(["risk", "execution", "data", "intent", "agent"])
    );
    assert_eq!(
        authoring_view["pipeline_order"],
        serde_json::json!(["data", "intent", "agent", "risk", "execution"])
    );
    assert_eq!(
        authoring_view["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|section| section["effective_kind"].clone())
            .collect::<Vec<_>>(),
        vec![
            serde_json::json!("risk"),
            serde_json::json!("execution"),
            serde_json::json!("data"),
            serde_json::json!("intent"),
            serde_json::json!("agent"),
        ]
    );
    assert_eq!(
        authoring_view["edges"]
            .as_array()
            .unwrap()
            .iter()
            .map(|edge| edge["reason"].clone())
            .collect::<Vec<_>>(),
        vec![
            serde_json::json!("intent_reads_data"),
            serde_json::json!("agent_uses_intent"),
            serde_json::json!("risk_governs_agent"),
            serde_json::json!("execution_applies_to_agent"),
        ]
    );
    assert_eq!(
        authoring_view["sections"][0]["start_line"],
        serde_json::json!(2)
    );
    assert_eq!(
        authoring_view["sections"][0]["end_line"],
        serde_json::json!(3)
    );
    assert!(authoring_view["sections"][0]["snippet"]
        .as_str()
        .unwrap()
        .contains("# risk"));
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_emits_authoring_view_without_explicit_headers() {
    let value = compile_formal_quantscript_for_test(
        r#"fn strategy() {
    execution.profile("paper", fee_bps=12.5, slippage_bps=7.5)
    let closes = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let m = momentum(closes, 20)
    if m > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_authoring_view_inferred",
    )
    .await;

    let authoring_view = formal_compile_authoring_view(&value);
    assert_eq!(
        authoring_view["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|section| {
                serde_json::json!({
                    "declared": section["declared_kind"].clone(),
                    "effective": section["effective_kind"].clone(),
                    "origin": section["origin"].clone(),
                })
            })
            .collect::<Vec<_>>(),
        vec![
            serde_json::json!({"declared": "execution", "effective": "execution", "origin": "hybrid"}),
            serde_json::json!({"declared": "data", "effective": "data", "origin": "hybrid"}),
            serde_json::json!({"declared": "intent", "effective": "intent", "origin": "hybrid"}),
        ]
    );
    assert_eq!(
        authoring_view["edges"]
            .as_array()
            .unwrap()
            .iter()
            .map(|edge| edge["relation"].clone())
            .collect::<Vec<_>>(),
        vec![serde_json::json!("dataflow")]
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_emits_partial_authoring_view_on_semantic_failure() {
    let value = compile_formal_quantscript_error_for_test(
        r#"fn strategy() {
    # risk
    risk.profile("global", max_position=0.35)
    # data
    let closes = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    # intent
    if fast > threshold {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        "compile_formal_partial_authoring_semantic_failure",
    )
    .await;

    let authoring_view = formal_compile_partial_authoring_view(&value);
    assert_eq!(authoring_view["kind"], "quantscript_authoring_view");
    assert_eq!(
        authoring_view["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|section| section["effective_kind"].clone())
            .collect::<Vec<_>>(),
        vec![
            serde_json::json!("risk"),
            serde_json::json!("data"),
            serde_json::json!("intent"),
        ]
    );
    assert_eq!(authoring_view["pool_pipeline"], serde_json::Value::Null);
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_emits_pool_pipeline_in_authoring_view() {
    let value = compile_formal_quantscript_for_test_with_universe_snapshot(
        r#"fn strategy() {
    # data
    let closes = fetch("BTCUSDT", exchange="binance", interval="1d", lookback=30)?
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let liquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=180)
    let leaders = top(sort_by(liquid, key="market_cap", order="desc"), 2)

    # intent
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)

    # agent
    rebalance(rank_weight(leaders, method="linear"), every="weekly")
}
"#,
        "compile_formal_authoring_view_pool_pipeline",
        Some(sample_formal_universe_snapshot_json()),
    )
    .await;

    let authoring_view = formal_compile_authoring_view(&value);
    assert_eq!(
        authoring_view["pool_pipeline"]["order"],
        serde_json::json!([
            "source",
            "eligibility",
            "features",
            "selection",
            "weighting",
            "rebalance"
        ])
    );
    assert_eq!(
        authoring_view["pool_pipeline"]["stages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|stage| stage["kind"].clone())
            .collect::<Vec<_>>(),
        vec![
            serde_json::json!("source"),
            serde_json::json!("eligibility"),
            serde_json::json!("features"),
            serde_json::json!("selection"),
            serde_json::json!("weighting"),
            serde_json::json!("rebalance"),
        ]
    );
    assert_eq!(
        authoring_view["pool_pipeline"]["stages"][0]["summary"],
        serde_json::json!("universe(exchange=binance, market=spot, quote=USDT)")
    );
    assert_eq!(
        authoring_view["pool_pipeline"]["stages"][1]["details"],
        serde_json::json!(["volume_24h >= 1000000000", "listing_age_days >= 180"])
    );
    assert_eq!(
        authoring_view["pool_pipeline"]["stages"][2]["status"],
        serde_json::json!("empty")
    );
    assert_eq!(
        authoring_view["pool_pipeline"]["stages"][3]["summary"],
        serde_json::json!("ordered_top_n by metadata.market_cap desc top 2")
    );
    assert_eq!(
        authoring_view["pool_pipeline"]["stages"][4]["summary"],
        serde_json::json!("rank_weight (linear)")
    );
    assert_eq!(
        authoring_view["pool_pipeline"]["stages"][5]["summary"],
        serde_json::json!("rebalance weekly")
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_emits_partial_pool_pipeline_on_lowering_failure() {
    let value = compile_formal_quantscript_error_for_test_with_universe_snapshot(
        r#"fn strategy() {
    # data
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let liquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=180)
    let ranked = top(sort_by(liquid, key="factor_score", order="desc"), 3)

    # intent
    for s in ranked {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=30)?
        let signal = momentum(closes, 20)
        if signal > 0.03 {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }

    # agent
    rebalance(rank_weight(ranked, method="linear"), every="weekly")
}
"#,
        "compile_formal_partial_authoring_lowering_failure",
        Some(sample_formal_universe_snapshot_json()),
    )
    .await;

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(
        api_error_detail_by_code(&value, "QPQSLOW011")["code"],
        "QPQSLOW011"
    );

    let authoring_view = formal_compile_partial_authoring_view(&value);
    assert_eq!(authoring_view["kind"], "quantscript_authoring_view");
    assert_eq!(
        authoring_view["pool_pipeline"]["stages"][3]["summary"],
        serde_json::json!("ordered_top_n by feature.factor_score desc top 3")
    );
    assert!(
        !authoring_view["pool_pipeline"]["stages"][3]["related_section_ids"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let effective_kinds = authoring_view["sections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|section| section["effective_kind"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert!(effective_kinds.len() >= 2);
    assert!(
        effective_kinds.iter().any(|kind| kind == "agent")
            || effective_kinds.iter().any(|kind| kind == "mixed")
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_diagnostics() {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_test",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let slow = closes[50..].sum() / 50
    if closes.last() > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_compile_failed");
    assert_eq!(value["details"][0]["code"], "QS0501");
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_rejects_non_trunk_control_flow_constructs_early() {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_non_trunk",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
import data as market_data

fn helper(series) {
    return helper(series)
}

async fn strategy() {
    let closes = await fetch("BTCUSDT", interval="1d", lookback=50)?
    let unsafe_try = sma(closes, 20)?
    let mut out = []
    out.push(1)
    if fetch("BTCUSDT", interval="1d", lookback=20).retryable() {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    for value in closes[20..] {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    while closes[0] > 0 {
        match closes[0] {
            _ => emit Intent("BUY", instrument="BTCUSDT", quantity=1.0),
        }
    }
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_compile_failed");
    assert!(value["details"]
        .as_array()
        .unwrap()
        .iter()
        .any(|detail| detail["code"] == "QS0601"));
    assert!(value["details"]
        .as_array()
        .unwrap()
        .iter()
        .any(|detail| detail["code"] == "QS0602"));
    assert!(value["details"]
        .as_array()
        .unwrap()
        .iter()
        .any(|detail| detail["code"] == "QS0603"));
    assert!(value["details"]
        .as_array()
        .unwrap()
        .iter()
        .any(|detail| detail["code"] == "QS0604"));
    assert!(value["details"]
        .as_array()
        .unwrap()
        .iter()
        .any(|detail| detail["code"] == "QS0605"));
    assert!(value["details"]
        .as_array()
        .unwrap()
        .iter()
        .any(|detail| detail["code"] == "QS0606"));
    assert!(value["details"]
        .as_array()
        .unwrap()
        .iter()
        .any(|detail| detail["code"] == "QS0607"));
    assert!(value["details"]
        .as_array()
        .unwrap()
        .iter()
        .any(|detail| detail["code"] == "QS0608"));
    assert!(value["details"]
        .as_array()
        .unwrap()
        .iter()
        .any(|detail| detail["code"] == "QS0609"));
    assert!(value["details"]
        .as_array()
        .unwrap()
        .iter()
        .any(|detail| detail["code"] == "QS0610"));
    assert!(api_error_detail_by_code(&value, "QS0610")["message"]
        .as_str()
        .unwrap()
        .contains(".ok()"));
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_fetch(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_missing_fetch",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_compile_failed");
    assert_eq!(value["details"][0]["code"], "QS0610");
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_unsupported_emit_action(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_bad_action",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    emit Intent("HOLD", instrument="BTCUSDT", quantity=1.0)
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW004");
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_malformed_spread_helper(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_bad_spread_helper",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let left = fetch("BTCUSDT", interval="1d", lookback=20)?
    let right = fetch("ETHUSDT", interval="1d", lookback=20)?
    if spread(left) > 0.0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW001");
    assert_eq!(
        value["details"][0]["reason"],
        "将条件下发重写为支持的指标或价差意图，或保留下发为无条件。"
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_indicator_source(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_missing_indicator_source",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let r = rsi(1, 14)
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_compile_failed");
    assert_eq!(value["details"][0]["code"], "QS0007");
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_macd_source(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_missing_macd_source",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let m = macd()
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW022");
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_momentum_source(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_missing_momentum_source",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let m = momentum()
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW022");
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_zscore_source(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_missing_zscore_source",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let z = zscore()
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW022");
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_non_positive_indicator_window(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_non_positive_indicator_window",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let fast = sma(closes, 0)
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW023");
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_indicator_window(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_missing_indicator_window",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let fast = sma(closes)
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW023");
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_invalid_moving_average_source(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_invalid_moving_average_source",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let fast = sma(1, 20)
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_compile_failed");
    assert_eq!(value["details"][0]["code"], "QS0007");
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_moving_average_source(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_missing_moving_average_source",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=20)?
    let fast = sma()
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_lowering_failed");
    assert_eq!(value["details"][0]["code"], "QPQSLOW024");
    assert_eq!(
        value["details"][0]["reason"],
        "将 fetch/get_data 序列传入移动平均辅助函数，或对 ema(...) 传入可识别的 MACD 线。"
    );
}

#[tokio::test]
async fn formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_invalid_universe_helper_input(
) {
    let app = build_app_router(test_app_state());
    let payload = serde_json::json!({
        "graph_id": "graph_test",
        "compile_id": "compile_formal_invalid_universe_input",
        "runtime_template": sample_compile_request_json()["runtime_config"].clone(),
        "source": r#"
fn strategy() {
    let selected = top(1, 2)
    rebalance(equal_weight(selected), every="1d")
}
"#
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/quantscript/formal/compile")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["error"], "quantscript_compile_failed");
    assert_eq!(value["details"][0]["code"], "QS0610");
}
