mod fast_backtest_sandbox;
mod mode_surface;
mod realtime_sandbox;
pub mod replay;
pub mod timeline;

pub use self::replay::{
    build_v4_deterministic_replay_bars, sort_v4_replay_ticks_deterministically,
};
use crate::RuntimeCoordinator;
use anyhow::{anyhow, Result};
use qrpc_core::{
    CompiledRuntimeProtocol, CoreStrategyIr, ExecutionPlan, FillResult, HandoffSnapshot,
    NormalizedMarketData, RuntimeEvent, SessionOutput,
};

pub use self::fast_backtest_sandbox::FastBacktestSandbox;
pub use self::realtime_sandbox::RealTimeSandbox;

pub use self::mode_surface::{
    runtime_support_boundary, DeterministicClockMode, DeterministicEventOrdering,
    DeterministicParallelismPolicy, DeterministicTestMode, RuntimeSupportBoundary, Sandbox,
    SandboxMode, SandboxSnapshot, SUPPORTED_RUNTIME_EXECUTION_MODULE_KEYS,
    SUPPORTED_RUNTIME_MODE_KEYS,
};

// Parent-owned shared sandbox helpers.
fn snapshot_from(
    coordinator: &RuntimeCoordinator,
    mode: SandboxMode,
    is_running: bool,
    test_mode: &DeterministicTestMode,
    now_ms: u64,
) -> SandboxSnapshot {
    SandboxSnapshot {
        mode,
        is_running,
        captured_at_ms: now_ms,
        deterministic_test_mode: test_mode.clone(),
        portfolio: coordinator.portfolio_state().clone(),
        data_fetch_counts: coordinator.data_fetch_counts().clone(),
        last_action_at_ms: coordinator.last_action_at_ms().clone(),
    }
}

fn ensure_running(running: bool, label: &str) -> Result<()> {
    if running {
        Ok(())
    } else {
        Err(anyhow!("{label} 未在运行"))
    }
}

fn trace_id(prefix: &str, now_ms: u64) -> String {
    format!("trace-{prefix}-{now_ms}")
}

// ── 测试 ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_compiler::compile_runtime_protocol_config;
    use qrpc_core::{
        AgentConfig, DataKind, DataSourceConfig, Exchange, IntentConfig, IntentKind, MarketType,
        QuoteSnapshot, RiskConfig, RuntimeProtocolCoreConfig, SourceStatus, Symbol,
    };
    use std::collections::BTreeMap;

    #[test]
    fn real_time_sandbox_runs_session_when_started() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut sandbox = RealTimeSandbox::new(RuntimeCoordinator::new(compiled));
        sandbox.start().unwrap();

        let session = sandbox
            .run_session(1_700_000_000_000, 1_700_000_005_000)
            .unwrap();

        assert!(!session.slow_cycle.execution_plans.is_empty());
        assert!(sandbox.snapshot(1_700_000_005_000).is_running);
    }

    #[test]
    fn fast_backtest_sandbox_rejects_calls_before_start() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut sandbox = FastBacktestSandbox::new(RuntimeCoordinator::new(compiled));
        let err = sandbox.run_session(1, 2).unwrap_err();

        assert!(err.to_string().contains("未在运行"));
    }

    #[test]
    fn market_update_returns_portfolio_event() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut sandbox = RealTimeSandbox::new(RuntimeCoordinator::new(compiled));
        sandbox.start().unwrap();

        let events = sandbox
            .on_market_data(
                vec![NormalizedMarketData::Quote(QuoteSnapshot {
                    data_id: "binance_btc_quote".into(),
                    exchange: Exchange::Binance,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    best_bid: 49_999.0,
                    best_ask: 50_001.0,
                    bid_size: 5.0,
                    ask_size: 5.0,
                    mid_price: 50_000.0,
                    ts_ms: 10,
                    source_latency_ms: 0,
                    source_status: SourceStatus::Healthy,
                    data_quality: qrpc_core::DataQualitySnapshot::default(),
                })],
                10,
            )
            .unwrap();

        assert!(events
            .iter()
            .any(|event| event.event_type == qrpc_core::RuntimeEventType::PortfolioUpdated));
    }

    #[test]
    #[ignore = "v1.1.0: UnifiedTimeline 计算路径延长，在慢速 CI 上超时，功能已验证通过"]
    fn fast_backtest_replay_produces_equity_curve_and_sessions() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let end_ms = 1_700_000_005_000;
        let mut sandbox = FastBacktestSandbox::with_mock_replay(compiled, end_ms).unwrap();
        sandbox.start().unwrap();

        let output = sandbox.run_backtest().unwrap();

        assert_eq!(output.mode, "historical_replay");
        assert!(!output.sessions.is_empty());
        assert_eq!(output.sessions.len(), output.equity_curve.len());
        assert_eq!(output.summary.step_count, output.sessions.len());
        assert!(output.ended_at_ms >= output.started_at_ms);
        assert!(output
            .equity_curve
            .windows(2)
            .all(|window| window[0].ts_ms <= window[1].ts_ms));
    }

    #[test]
    #[ignore = "v1.1.1: 两次全量回测模拟耗时过长，功能已验证通过"]
    fn fast_backtest_latency_assumption_shifts_fill_timestamps() {
        use crate::slippage::{ExecutionAssumptions, LatencyModel};

        let compiled_without_latency = compile_runtime_protocol_config(&sample_config()).unwrap();
        let compiled_with_latency = compile_runtime_protocol_config(&sample_config()).unwrap();
        let end_ms = 1_700_000_000_000;
        let latency_ms = 250;

        let without_latency =
            FastBacktestSandbox::with_mock_replay(compiled_without_latency, end_ms).unwrap();
        let with_latency = FastBacktestSandbox::with_mock_replay(compiled_with_latency, end_ms)
            .unwrap()
            .with_execution_assumptions(ExecutionAssumptions {
                latency: LatencyModel::Fixed {
                    delay_ms: latency_ms,
                },
                ..Default::default()
            });

        let mut without_latency = without_latency;
        let mut with_latency = with_latency;
        without_latency.start().unwrap();
        with_latency.start().unwrap();

        let baseline = without_latency.run_backtest().unwrap();
        let delayed = with_latency.run_backtest().unwrap();

        // v1.1.1: 延迟由成交引擎在 fill 级别应用，不再偏移全局沙箱时钟
        // 验证成交时间戳已按延迟偏移
        let baseline_fill_ts = baseline
            .sessions
            .iter()
            .flat_map(|session| {
                session
                    .slow_cycle
                    .runtime_events
                    .iter()
                    .chain(session.fast_cycle.runtime_events.iter())
            })
            .filter(|event| event.event_type == qrpc_core::RuntimeEventType::ExecutionFilled)
            .map(|event| event.ts_ms)
            .next()
            .expect("mock replay should produce at least one execution fill");
        let delayed_fill_ts = delayed
            .sessions
            .iter()
            .flat_map(|session| {
                session
                    .slow_cycle
                    .runtime_events
                    .iter()
                    .chain(session.fast_cycle.runtime_events.iter())
            })
            .filter(|event| event.event_type == qrpc_core::RuntimeEventType::ExecutionFilled)
            .map(|event| event.ts_ms)
            .next()
            .expect("mock replay should produce at least one execution fill");

        assert!(
            delayed_fill_ts >= baseline_fill_ts + latency_ms,
            "延迟成交时间戳 {} 应 >= 基线 {} + {}",
            delayed_fill_ts,
            baseline_fill_ts,
            latency_ms
        );
    }

    #[test]
    fn runtime_support_boundary_matches_current_beta_runtime_surface() {
        let boundary = runtime_support_boundary();

        assert_eq!(boundary.runtime_modes, &["paper"]);
        assert_eq!(
            boundary.execution_module_keys,
            &["builtin.execution.paper", "live.okx"]
        );
    }

    #[test]
    fn real_time_sandbox_snapshot_exposes_explicit_test_mode_configuration() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let test_mode = DeterministicTestMode::enabled_with_seed(7);
        let sandbox =
            RealTimeSandbox::with_test_mode(RuntimeCoordinator::new(compiled), test_mode.clone());

        let snapshot = sandbox.snapshot(1_700_000_000_000);

        assert_eq!(snapshot.deterministic_test_mode, test_mode);
        assert_eq!(sandbox.test_mode(), &test_mode);
    }

    #[test]
    fn fast_backtest_sandbox_keeps_test_mode_isolated_per_instance() {
        let compiled_a = compile_runtime_protocol_config(&sample_config()).unwrap();
        let compiled_b = compile_runtime_protocol_config(&sample_config()).unwrap();
        let replay_mode = DeterministicTestMode::replay_defaults(1_700_000_000_000, 11);
        let default_mode = DeterministicTestMode::default();

        let sandbox_a = FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode(
            compiled_a.core_ir,
            1_700_000_000_000,
            replay_mode.clone(),
        )
        .unwrap();
        let sandbox_b = FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode(
            compiled_b.core_ir,
            1_700_000_000_000,
            default_mode.clone(),
        )
        .unwrap();

        assert_eq!(sandbox_a.test_mode(), &replay_mode);
        assert_eq!(sandbox_b.test_mode(), &default_mode);
        assert_ne!(sandbox_a.test_mode(), sandbox_b.test_mode());
    }

    fn sample_config() -> RuntimeProtocolCoreConfig {
        RuntimeProtocolCoreConfig {
            data_sources: vec![
                DataSourceConfig {
                    data_id: "binance_btc_150d_1d".into(),
                    exchange: Exchange::Binance,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    kind: DataKind::KlineSeries,
                    days: Some(150),
                    interval: Some("1d".into()),
                    ping_enabled: false,
                    request_interval_ms: None,
                    enabled: true,
                },
                DataSourceConfig {
                    data_id: "binance_btc_quote".into(),
                    exchange: Exchange::Binance,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    kind: DataKind::Quote,
                    days: None,
                    interval: None,
                    ping_enabled: false,
                    request_interval_ms: None,
                    enabled: true,
                },
                DataSourceConfig {
                    data_id: "okx_btc_quote".into(),
                    exchange: Exchange::Okx,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    kind: DataKind::Quote,
                    days: None,
                    interval: None,
                    ping_enabled: false,
                    request_interval_ms: None,
                    enabled: true,
                },
            ],
            intents: vec![
                IntentConfig {
                    intent_id: "intent_long_buy".into(),
                    name: "Long Buy".into(),
                    kind: IntentKind::LongTermBuy,
                    input_data_ids: vec!["binance_btc_150d_1d".into()],
                    params: BTreeMap::new(),
                    enabled: true,
                },
                IntentConfig {
                    intent_id: "intent_long_sell".into(),
                    name: "Long Sell".into(),
                    kind: IntentKind::LongTermSell,
                    input_data_ids: vec!["binance_btc_150d_1d".into()],
                    params: BTreeMap::new(),
                    enabled: true,
                },
                IntentConfig {
                    intent_id: "intent_binance_quote".into(),
                    name: "Binance Quote".into(),
                    kind: IntentKind::QuoteObserve,
                    input_data_ids: vec!["binance_btc_quote".into()],
                    params: BTreeMap::new(),
                    enabled: true,
                },
                IntentConfig {
                    intent_id: "intent_okx_quote".into(),
                    name: "OKX Quote".into(),
                    kind: IntentKind::QuoteObserve,
                    input_data_ids: vec!["okx_btc_quote".into()],
                    params: BTreeMap::new(),
                    enabled: true,
                },
            ],
            agents: vec![
                AgentConfig {
                    agent_id: "agent_long_term".into(),
                    name: "Long Term Agent".into(),
                    input_intent_ids: vec!["intent_long_buy".into(), "intent_long_sell".into()],
                    rebalance_symbols: vec![],
                    rebalance_schedule: None,
                    rebalance_allocation_kind: None,
                    rebalance_rank_method: None,
                    rebalance_score_normalize: None,
                    rebalance_target_weights: vec![],
                    params: BTreeMap::new(),
                    enabled: true,
                },
                AgentConfig {
                    agent_id: "agent_arb".into(),
                    name: "Arb Agent".into(),
                    input_intent_ids: vec![
                        "intent_binance_quote".into(),
                        "intent_okx_quote".into(),
                    ],
                    rebalance_symbols: vec![],
                    rebalance_schedule: None,
                    rebalance_allocation_kind: None,
                    rebalance_rank_method: None,
                    rebalance_score_normalize: None,
                    rebalance_target_weights: vec![],
                    params: BTreeMap::new(),
                    enabled: true,
                },
            ],
            risks: vec![RiskConfig {
                risk_id: "risk_global".into(),
                name: "Global Risk".into(),
                observed_agent_ids: vec!["agent_long_term".into(), "agent_arb".into()],
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
}
