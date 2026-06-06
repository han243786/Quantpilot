use crate::{
    agent_module::AgentModuleProvider, config_tracker, data_module::DataModuleProvider,
    execution_module::ExecutionModuleProvider, intent_module::IntentModuleProvider,
    merge_coordinator, risk_checker::RiskCheckerProvider, risk_monitor, runtime_state,
};
use qrpc_core::{
    CoreStrategyIr, Exchange, ExchangeExposure, NormalizedMarketData, PortfolioState,
    RiskDecisionMode, RuntimeEvent, RuntimeEventType, Symbol,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

mod config_generation;
mod constructor_provider_wiring;
mod execution_market_entrypoints;
mod provider_delegation_helpers;
mod session_cycle_orchestration;
mod state_config_accessors;

/// 配置代际记录
#[derive(Debug, Clone)]
pub struct ConfigGenerationEntry {
    pub generation: u64,
    pub activated_at_ms: u64,
    pub deployment_revision: String,
    pub parameter_version: String,
}

#[derive(Clone)]
pub struct RuntimeCoordinator {
    // v2.2.0: 提取为子结构体的可变状态
    pub(crate) state: runtime_state::RuntimeState,
    pub(crate) config: config_tracker::ConfigTracker,
    pub(crate) merge: merge_coordinator::MergeCoordinator,
    // 核心编译产物 (不可变引用)
    core_ir: CoreStrategyIr,
    // 5 个模块提供者 (trait objects, 依赖注入)
    data_module: Arc<dyn DataModuleProvider>,
    intent_module: Arc<dyn IntentModuleProvider>,
    agent_module: Arc<dyn AgentModuleProvider>,
    execution_module: Arc<Mutex<dyn ExecutionModuleProvider>>,
    risk_checker: Arc<dyn RiskCheckerProvider>,
    // 杂项状态
    risk_mode: RiskDecisionMode,
    pending_module_configs: BTreeMap<String, serde_json::Value>,
    /// v1.2.0: 独立实时风控监控器（可选）
    risk_monitor: Option<risk_monitor::RiskMonitor>,
    /// v1.2.0: RiskMonitor 是否已触发停止
    risk_stopped: bool,
}

impl std::fmt::Debug for RuntimeCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeCoordinator")
            .field("core_ir", &self.core_ir)
            .field("portfolio", &self.state.portfolio)
            .field("data_fetch_counts", &self.state.data_fetch_counts)
            .field("last_action_at_ms", &self.state.last_action_at_ms)
            .field("last_rebalance_at_ms", &self.state.last_rebalance_at_ms)
            .field("data_provider_key", &self.data_module.provider_key())
            .field("intent_provider_key", &self.intent_module.provider_key())
            .field("agent_provider_key", &self.agent_module.provider_key())
            .field(
                "execution_provider_key",
                &self.execution_module_provider_key(),
            )
            .field("risk_provider_key", &self.risk_checker.provider_key())
            .field("risk_stopped", &self.risk_stopped)
            .field("risk_monitor_enabled", &self.risk_monitor.is_some())
            .finish()
    }
}

impl RuntimeCoordinator {
    pub fn portfolio_update_event(
        &self,
        source_id: &str,
        trace_id: &str,
        now_ms: u64,
    ) -> RuntimeEvent {
        let equity_estimate = portfolio_equity_estimate(&self.state.portfolio);
        let open_orders = self
            .state
            .portfolio
            .open_orders
            .iter()
            .map(|order| {
                json!({
                    "order_id": order.order_id,
                    "side": format!("{:?}", order.side),
                    "remaining_qty": order.remaining_qty,
                    "limit_price": order.limit_price,
                    "reserved_cash": order.reserved_cash,
                    "reserved_qty": order.reserved_qty,
                })
            })
            .collect::<Vec<_>>();
        RuntimeEvent {
            event_id: format!("evt-portfolio-{source_id}-{now_ms}"),
            event_type: RuntimeEventType::PortfolioUpdated,
            trace_id: trace_id.to_string(),
            source_id: source_id.to_string(),
            ts_ms: now_ms,
            payload: json!({
                "cash_balance": self.state.portfolio.cash_balance,
                "available_cash_balance": self.state.portfolio.available_cash_balance,
                "frozen_cash_balance": self.state.portfolio.frozen_cash_balance,
                "total_gross_notional": self.state.portfolio.total_gross_notional,
                "total_net_notional": self.state.portfolio.total_net_notional,
                "total_leverage": self.state.portfolio.total_leverage,
                "equity_estimate": equity_estimate,
                "positions": self.state.portfolio.positions.len(),
                "open_order_count": self.state.portfolio.open_orders.len(),
                "open_orders": open_orders,
            }),
        }
    }

    fn refresh_portfolio_state(&mut self, normalized_data: &[NormalizedMarketData], now_ms: u64) {
        let quotes = quote_price_map(normalized_data);
        let mut exposures: BTreeMap<Exchange, (f64, f64)> = BTreeMap::new();

        for position in &mut self.state.portfolio.positions {
            // v2.3.0: 若无新行情, 保持上次市价并记录陈旧状态
            position.mark_price = quotes
                .get(&(position.exchange.clone(), position.symbol.clone()))
                .copied()
                .unwrap_or(position.mark_price);
            // 无行情时标记: 使用 avg_entry_price 作为保守估计 (不做多, 也不做空)
            if position.mark_price <= 0.0 {
                position.mark_price = position.avg_entry_price.max(0.01);
            }
            position.unrealized_pnl =
                (position.mark_price - position.avg_entry_price) * position.net_qty;
            let gross = position.net_qty.abs() * position.mark_price;
            let net = position.net_qty * position.mark_price;
            let entry = exposures.entry(position.exchange.clone()).or_default();
            entry.0 += gross;
            entry.1 += net;
        }

        self.state.portfolio.exchange_exposures = exposures
            .into_iter()
            .map(
                |(exchange, (gross_notional, net_notional))| ExchangeExposure {
                    leverage: 0.0,
                    exchange,
                    gross_notional,
                    net_notional,
                },
            )
            .collect();

        self.state.portfolio.total_gross_notional = self
            .state
            .portfolio
            .exchange_exposures
            .iter()
            .map(|item| item.gross_notional)
            .sum();
        self.state.portfolio.total_net_notional = self
            .state
            .portfolio
            .exchange_exposures
            .iter()
            .map(|item| item.net_notional)
            .sum();
        let equity_estimate = portfolio_equity_estimate(&self.state.portfolio);
        // v2.3.0: 使用比例下限替代固定 floor(1.0), 避免小额权益时杠杆被低估
        let initial_equity = 100_000.0;
        let equity_floor = 1_000.0_f64.max(initial_equity * 0.001);
        self.state.portfolio.total_leverage = if equity_estimate.abs() > f64::EPSILON {
            self.state.portfolio.total_gross_notional / equity_estimate.abs().max(equity_floor)
        } else {
            0.0
        };
        for exposure in &mut self.state.portfolio.exchange_exposures {
            exposure.leverage = if equity_estimate.abs() > f64::EPSILON {
                exposure.gross_notional / equity_estimate.abs().max(equity_floor)
            } else {
                0.0
            };
        }
        self.state.portfolio.updated_at_ms = now_ms;
    }
}

fn portfolio_equity_estimate(portfolio: &PortfolioState) -> f64 {
    portfolio.cash_balance + portfolio.total_net_notional
}

fn quote_price_map(normalized_data: &[NormalizedMarketData]) -> BTreeMap<(Exchange, Symbol), f64> {
    normalized_data
        .iter()
        .filter_map(|item| match item {
            NormalizedMarketData::Quote(quote) => Some((
                (quote.exchange.clone(), quote.symbol.clone()),
                quote.mid_price,
            )),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        agent_module::AgentEvaluationRequest,
        data_module::{DataCollectionOutput, DataCollectionRequest},
        execution_module::ExecutionPlanningRequest,
        intent_module::IntentEvaluationRequest,
        risk_checker::{RiskCheckOutput, RiskCheckRequest},
        AgentEvaluationOutput, ExecutionPlanner, ExecutionPlanningOutput, ExecutionSubmitter,
        IntentEvaluationOutput,
    };
    use anyhow::Result;
    use qrpc_compiler::compile_runtime_protocol_config;
    use qrpc_core::{
        AgentConfig, DataKind, DataSourceConfig, DecisionStatus, ExecutionPlan, IntentConfig,
        IntentKind, MarketType, RiskConfig, RiskDecision, RuntimeProtocolCoreConfig,
    };
    use std::collections::BTreeSet;

    #[derive(Default)]
    struct RejectAllRiskChecker;

    #[derive(Default)]
    struct NoopExecutionModule;

    #[derive(Default)]
    struct NoopAgentModule;

    #[derive(Default)]
    struct NoopIntentModule;

    #[derive(Default)]
    struct NoopDataModule;

    impl DataModuleProvider for NoopDataModule {
        fn provider_key(&self) -> &'static str {
            "test.data.noop"
        }

        fn collect(&self, _request: DataCollectionRequest<'_>) -> Result<DataCollectionOutput> {
            Ok(DataCollectionOutput {
                normalized_data: Vec::new(),
                events: Vec::new(),
            })
        }
    }

    impl IntentModuleProvider for NoopIntentModule {
        fn provider_key(&self) -> &'static str {
            "test.intent.noop"
        }

        fn evaluate_intents(
            &self,
            _request: IntentEvaluationRequest<'_>,
        ) -> IntentEvaluationOutput {
            IntentEvaluationOutput {
                signals: Vec::new(),
                events: Vec::new(),
            }
        }
    }

    impl AgentModuleProvider for NoopAgentModule {
        fn provider_key(&self) -> &'static str {
            "test.agent.noop"
        }

        fn evaluate_agents(&self, _request: AgentEvaluationRequest<'_>) -> AgentEvaluationOutput {
            AgentEvaluationOutput {
                decisions: Vec::new(),
                events: Vec::new(),
                evaluated_rebalance_agent_ids: BTreeSet::new(),
            }
        }
    }

    impl ExecutionPlanner for NoopExecutionModule {
        fn provider_key(&self) -> &'static str {
            "test.execution.noop"
        }

        fn plan_execution(
            &self,
            _request: ExecutionPlanningRequest<'_>,
        ) -> ExecutionPlanningOutput {
            ExecutionPlanningOutput {
                plans: Vec::new(),
                events: Vec::new(),
            }
        }
    }

    impl ExecutionSubmitter for NoopExecutionModule {
        fn submit_plan(
            &mut self,
            plan: &ExecutionPlan,
            _normalized_data: &[NormalizedMarketData],
            _portfolio: &mut PortfolioState,
            _now_ms: u64,
            _trace_id: &str,
        ) -> qrpc_core::FillResult {
            qrpc_core::FillResult {
                plan_id: plan.plan_id.clone(),
                status: qrpc_core::ExecutionStatus::Rejected,
                fills: Vec::new(),
                open_orders: Vec::new(),
                events: Vec::new(),
            }
        }

        fn on_market_update(
            &mut self,
            _normalized_data: &[NormalizedMarketData],
            _portfolio: &mut PortfolioState,
            now_ms: u64,
            _trace_id: &str,
        ) -> qrpc_core::FillResult {
            qrpc_core::FillResult {
                plan_id: format!("noop-market-{now_ms}"),
                status: qrpc_core::ExecutionStatus::Open,
                fills: Vec::new(),
                open_orders: Vec::new(),
                events: Vec::new(),
            }
        }
    }

    impl RiskCheckerProvider for RejectAllRiskChecker {
        fn provider_key(&self) -> &'static str {
            "test.risk.reject_all"
        }

        fn evaluate(&self, request: RiskCheckRequest<'_>) -> Result<RiskCheckOutput> {
            Ok(RiskCheckOutput {
                decisions: request
                    .decisions
                    .iter()
                    .map(|decision| RiskDecision {
                        risk_decision_id: format!(
                            "reject-{}-{}",
                            decision.decision_id, request.now_ms
                        ),
                        risk_id: request
                            .core_ir
                            .risk_policies
                            .first()
                            .map(|risk| risk.policy_id.clone())
                            .unwrap_or_else(|| "risk_override".into()),
                        agent_decision_id: decision.decision_id.clone(),
                        symbol: decision.symbol.clone(),
                        status: DecisionStatus::Reject,
                        mode: request.mode,
                        adjusted_portfolio_target_decision: None,
                        adjusted_actions: Vec::new(),
                        reason_codes: vec![qrpc_core::RiskReasonCode::InvalidAction],
                        reason_text: "rejected by test provider".into(),
                        produced_at_ms: request.now_ms,
                        trace_id: request.trace_id.to_string(),
                    })
                    .collect(),
                events: Vec::new(),
                approved_agent_ids: std::collections::BTreeSet::new(),
            })
        }
    }

    #[test]
    fn session_runs_both_slow_and_fast_cycles() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::new(compiled);
        let output = runtime
            .run_session(1_700_000_000_000, 1_700_000_005_000)
            .unwrap();

        assert!(!output.slow_cycle.intent_signals.is_empty());
        assert!(!output.fast_cycle.intent_signals.is_empty());
        assert!(!output.slow_cycle.execution_plans.is_empty());
        assert!(
            output
                .data_fetch_counts
                .get("binance_btc_quote")
                .copied()
                .unwrap_or_default()
                >= 1
        );
    }

    #[test]
    fn risk_clamps_when_leverage_limit_is_tight() {
        let mut config = sample_config();
        config.risks[0].max_total_leverage = 0.05;
        config.risks[0].max_exchange_leverage = 0.05;
        let compiled = compile_runtime_protocol_config(&config).unwrap();
        let mut runtime = RuntimeCoordinator::new(compiled);
        let output = runtime.run_slow_cycle(1_700_000_005_000).unwrap();

        assert!(output
            .risk_decisions
            .iter()
            .any(|item| matches!(item.status, DecisionStatus::Clamp)));
    }

    #[test]
    fn custom_data_module_provider_can_override_runtime_inputs() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::with_data_module(compiled, NoopDataModule);
        let output = runtime.run_fast_cycle(1_700_000_005_000).unwrap();

        assert!(output.normalized_data.is_empty());
        assert!(output.intent_signals.is_empty());
        assert!(output.agent_decisions.is_empty());
        assert_eq!(runtime.data_module().provider_key(), "test.data.noop");
    }

    #[test]
    fn custom_intent_module_provider_can_override_runtime_signals() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::with_intent_module(compiled, NoopIntentModule);
        let output = runtime.run_fast_cycle(1_700_000_005_000).unwrap();

        assert!(output
            .normalized_data
            .iter()
            .any(|item| matches!(item, NormalizedMarketData::Quote(_))));
        assert!(output.intent_signals.is_empty());
        assert!(output.agent_decisions.is_empty());
        assert_eq!(runtime.intent_module().provider_key(), "test.intent.noop");
    }

    #[test]
    fn custom_agent_module_provider_can_override_runtime_decisions() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::with_agent_module(compiled, NoopAgentModule);
        let output = runtime.run_fast_cycle(1_700_000_005_000).unwrap();

        assert!(!output.intent_signals.is_empty());
        assert!(output.agent_decisions.is_empty());
        assert!(output.risk_decisions.is_empty());
        assert_eq!(runtime.agent_module().provider_key(), "test.agent.noop");
    }

    #[test]
    fn custom_risk_checker_provider_can_override_runtime_decisions() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::with_risk_checker(compiled, RejectAllRiskChecker);
        let output = runtime.run_slow_cycle(1_700_000_005_000).unwrap();

        assert!(!output.agent_decisions.is_empty());
        assert!(output
            .risk_decisions
            .iter()
            .all(|item| matches!(item.status, DecisionStatus::Reject)));
        assert!(output.execution_plans.is_empty());
        assert_eq!(
            runtime.risk_checker().provider_key(),
            "test.risk.reject_all"
        );
    }

    #[test]
    fn custom_execution_module_provider_can_override_runtime_plans() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::with_execution_module(compiled, NoopExecutionModule);
        let output = runtime.run_slow_cycle(1_700_000_005_000).unwrap();

        assert!(!output.risk_decisions.is_empty());
        assert!(output.execution_plans.is_empty());
        assert!(output.fill_reports.is_empty());
        assert_eq!(
            runtime.execution_module_provider_key(),
            "test.execution.noop"
        );
    }

    #[test]
    fn fills_are_traceable_back_to_runtime_chain() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let mut runtime = RuntimeCoordinator::new(compiled);
        let output = runtime
            .run_session(1_700_000_000_000, 1_700_000_005_000)
            .unwrap();

        let trace_id = output.slow_cycle.trace_id.clone();
        assert!(!output.slow_cycle.fill_reports.is_empty());
        assert!(output
            .slow_cycle
            .fill_reports
            .iter()
            .all(|fill| fill.trace_id == trace_id));
        assert!(output
            .slow_cycle
            .runtime_events
            .iter()
            .any(|evt| evt.trace_id == trace_id
                && evt.event_type == RuntimeEventType::ExecutionFilled));
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
                    data_id: "okx_btc_150d_1d".into(),
                    exchange: Exchange::Okx,
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
