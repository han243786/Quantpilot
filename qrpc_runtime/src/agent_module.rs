use qrpc_core::{
    AgentDecision, CoreStrategyIr, Exchange, IntentSignal, PortfolioState, RuntimeEvent,
    RuntimeEventType, SignalSide, Symbol,
};
use qrpc_core_ir::{AgentPolicyKind, SignalKind};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

mod cross_venue_arbitrage;
mod portfolio_rebalance;
mod weighted_signal_decisions;

// v2.1.1: 提取魔法数字为命名常量
const MIN_QUANTITY_RATIO: f64 = 0.01;
const DEFAULT_DECISION_THRESHOLD: f64 = 0.05;
const SPREAD_MULTIPLIER: f64 = 20.0;
#[cfg(test)]
const DEFAULT_COST_BUFFER_BPS: f64 = 20.0;

#[derive(Debug, Clone)]
pub struct AgentEvaluationRequest<'a> {
    pub cycle_name: &'a str,
    pub signals: &'a [IntentSignal],
    pub core_ir: &'a CoreStrategyIr,
    pub portfolio: &'a PortfolioState,
    pub last_rebalance_at_ms: &'a BTreeMap<String, u64>,
    pub now_ms: u64,
    pub trace_id: &'a str,
}

#[derive(Debug, Clone)]
pub struct AgentEvaluationOutput {
    pub decisions: Vec<AgentDecision>,
    pub events: Vec<RuntimeEvent>,
    pub evaluated_rebalance_agent_ids: BTreeSet<String>,
}

pub trait AgentModuleProvider: Send + Sync {
    fn provider_key(&self) -> &'static str {
        "builtin.agent.default"
    }

    fn evaluate_agents(&self, request: AgentEvaluationRequest<'_>) -> AgentEvaluationOutput;
}

#[derive(Debug, Clone, Default)]
pub struct BuiltinAgentModule;

impl AgentModuleProvider for BuiltinAgentModule {
    fn evaluate_agents(&self, request: AgentEvaluationRequest<'_>) -> AgentEvaluationOutput {
        let agent_policies = &request.core_ir.agent_policies;
        let mut decisions = Vec::with_capacity(agent_policies.len());
        let mut events = Vec::with_capacity(agent_policies.len());
        let mut evaluated_rebalance_agent_ids = BTreeSet::new();

        for agent in agent_policies.iter().filter(|item| item.enabled) {
            let related = request
                .signals
                .iter()
                .filter(|signal| agent.input_signal_ids.contains(&signal.intent_id))
                .cloned()
                .collect::<Vec<_>>();

            let agent_decisions = match (&agent.kind, request.cycle_name) {
                (AgentPolicyKind::WeightedSignals, "slow") => {
                    weighted_signal_decisions::build_weighted_agent_decisions(
                        agent,
                        &related,
                        request.core_ir,
                        request.portfolio,
                        request.now_ms,
                        request.trace_id,
                    )
                }
                (AgentPolicyKind::PortfolioRebalance, "slow") => {
                    if !portfolio_rebalance::portfolio_rebalance_due(
                        agent,
                        request.last_rebalance_at_ms,
                        request.now_ms,
                    ) {
                        Vec::new()
                    } else {
                        evaluated_rebalance_agent_ids.insert(agent.agent_id.clone());
                        portfolio_rebalance::build_portfolio_rebalance_decision(
                            agent,
                            &related,
                            request.core_ir,
                            request.portfolio,
                            request.now_ms,
                            request.trace_id,
                        )
                        .into_iter()
                        .collect()
                    }
                }
                (AgentPolicyKind::CrossVenueArbitrage, "fast") => {
                    cross_venue_arbitrage::build_arb_agent_decision(
                        agent,
                        &related,
                        request.core_ir,
                        request.portfolio,
                        request.now_ms,
                        request.trace_id,
                    )
                    .into_iter()
                    .collect()
                }
                _ => Vec::new(),
            };

            for decision in agent_decisions {
                events.push(RuntimeEvent {
                    event_id: format!("evt-agent-{}-{}", decision.decision_id, request.now_ms),
                    event_type: RuntimeEventType::AgentDecisionProduced,
                    trace_id: request.trace_id.to_string(),
                    source_id: decision.agent_id.clone(),
                    ts_ms: request.now_ms,
                    payload: json!({
                        "provider_key": self.provider_key(),
                        "net_side": format!("{:?}", decision.net_side),
                        "net_strength": decision.net_strength,
                        "actions": decision.proposed_actions.len(),
                        "portfolio_targets": decision
                            .portfolio_target_decision
                            .as_ref()
                            .map(|item| item.target.target_weights.len())
                            .unwrap_or(0),
                    }),
                });
                decisions.push(decision);
            }
        }

        AgentEvaluationOutput {
            decisions,
            events,
            evaluated_rebalance_agent_ids,
        }
    }
}

#[allow(dead_code)]
fn signal_kind_for_intent(core_ir: &CoreStrategyIr, intent_id: &str) -> Option<SignalKind> {
    // v2.4.0 P2-J3: 高频调用路径, 调用方应预先构建 HashMap 索引
    // 单次调用 O(N_rules) 可接受, 但循环中重复调用应为 O(1)
    core_ir
        .signal_rules
        .iter()
        .find(|rule| rule.indicator_id == intent_id)
        .map(|rule| rule.signal_kind)
}

fn build_signal_kind_index(
    core_ir: &CoreStrategyIr,
) -> std::collections::HashMap<String, SignalKind> {
    core_ir
        .signal_rules
        .iter()
        .map(|rule| (rule.indicator_id.clone(), rule.signal_kind))
        .collect()
}

fn signal_score(signal: &IntentSignal) -> f64 {
    let magnitude = signal.strength.abs();
    match signal.side {
        SignalSide::Long => magnitude,
        SignalSide::Short => -magnitude,
        SignalSide::Neutral => 0.0,
    }
}

fn available_position_ratio(
    portfolio: &PortfolioState,
    exchange: &Exchange,
    symbol: &Symbol,
    reference_price: f64,
) -> f64 {
    if !reference_price.is_finite() || reference_price <= 0.0 {
        return 0.0;
    }
    let equity = portfolio_equity(portfolio).abs().max(1.0);
    let available_qty = portfolio
        .positions
        .iter()
        .find(|position| &position.exchange == exchange && &position.symbol == symbol)
        .map(|position| (position.net_qty.max(0.0) - position.frozen_qty).max(0.0))
        .unwrap_or(0.0);
    (available_qty * reference_price / equity).max(0.0)
}

fn current_position_ratio(
    portfolio: &PortfolioState,
    exchange: &Exchange,
    symbol: &Symbol,
    reference_price: f64,
) -> f64 {
    if !reference_price.is_finite() || reference_price <= 0.0 {
        return 0.0;
    }
    let equity = portfolio_equity(portfolio).abs().max(1.0);
    let current_qty = portfolio
        .positions
        .iter()
        .find(|position| &position.exchange == exchange && &position.symbol == symbol)
        .map(|position| position.net_qty.max(0.0))
        .unwrap_or(0.0);
    (current_qty * reference_price / equity).max(0.0)
}

fn portfolio_equity(portfolio: &PortfolioState) -> f64 {
    portfolio.cash_balance + portfolio.total_net_notional
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::{IntentKind, IntentSignal, PortfolioState, Position};
    use qrpc_core_ir::{
        AgentPolicy, AgentPolicyKind, CoreMetadata, CoreSourceKind, CoreStrategyIr,
        CoreTimeInForce, ExecutionRule, ExecutionSizingKind, RebalanceSchedule, SignalKind,
        SignalRule,
    };
    use std::collections::BTreeMap;

    fn sample_execution_rule() -> ExecutionRule {
        ExecutionRule {
            execution_id: "exec".into(),
            venue_kind: "paper".into(),
            sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
            slippage_bps: 5.0,
            taker_fee_bps: 10.0,
            total_cost_buffer_bps: DEFAULT_COST_BUFFER_BPS,
            time_in_force: CoreTimeInForce::Gtc,
            params: BTreeMap::new(),
        }
    }

    fn sample_core_ir_with_agent_policy(
        agent_id: &str,
        kind: AgentPolicyKind,
        input_signal_ids: Vec<&str>,
        signal_rules: Vec<SignalRule>,
        decision_threshold: Option<f64>,
        max_quantity_ratio: f64,
        spread_trigger_bps: Option<f64>,
    ) -> CoreStrategyIr {
        CoreStrategyIr {
            ir_version: qrpc_core::CORE_IR_V1_VERSION.to_string(),
            metadata: CoreMetadata {
                strategy_id: "agent_test".into(),
                name: "Agent Test".into(),
                source_kind: CoreSourceKind::RuntimeProtocol,
            },
            data_bindings: vec![],
            indicators: vec![],
            signal_rules,
            agent_policies: vec![AgentPolicy {
                agent_id: agent_id.into(),
                name: agent_id.into(),
                kind,
                input_signal_ids: input_signal_ids.into_iter().map(str::to_string).collect(),
                rebalance_symbols: vec![],
                rebalance_schedule: None,
                rebalance_allocation_kind: None,
                rebalance_rank_method: None,
                rebalance_score_normalize: None,
                rebalance_target_weights: vec![],
                decision_threshold,
                max_quantity_ratio,
                spread_trigger_bps,
                enabled: true,
            }],
            risk_policies: vec![],
            execution: sample_execution_rule(),
            edges: vec![],
        }
    }

    fn sample_portfolio_with_symbol_position(
        exchange: Exchange,
        symbol: Symbol,
        qty: f64,
        mark_price: f64,
    ) -> PortfolioState {
        let mut portfolio = PortfolioState::new(100_000.0, 0);
        portfolio.positions.push(Position {
            exchange,
            symbol,
            net_qty: qty,
            frozen_qty: 0.0,
            avg_entry_price: mark_price,
            mark_price,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        });
        portfolio.total_net_notional = qty * mark_price;
        portfolio.total_gross_notional = qty.abs() * mark_price;
        portfolio.total_leverage = portfolio.total_gross_notional
            / (portfolio.cash_balance + portfolio.total_net_notional);
        portfolio
    }

    fn sample_portfolio_with_position(
        exchange: Exchange,
        qty: f64,
        mark_price: f64,
    ) -> PortfolioState {
        sample_portfolio_with_symbol_position(exchange, Symbol::BtcUsdt, qty, mark_price)
    }

    #[test]
    fn builtin_agent_module_emits_decision_for_fast_cycle_arb() {
        let module = BuiltinAgentModule;
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_arb",
            AgentPolicyKind::CrossVenueArbitrage,
            vec!["intent_binance_quote", "intent_okx_quote"],
            vec![
                SignalRule {
                    signal_id: "binance_observe".into(),
                    indicator_id: "intent_binance_quote".into(),
                    signal_kind: SignalKind::Observe,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "observe".into(),
                    },
                },
                SignalRule {
                    signal_id: "okx_observe".into(),
                    indicator_id: "intent_okx_quote".into(),
                    signal_kind: SignalKind::Observe,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "observe".into(),
                    },
                },
            ],
            None,
            0.4,
            Some(30.0),
        );
        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "fast",
            signals: &[
                IntentSignal {
                    signal_id: "s1".into(),
                    intent_id: "intent_binance_quote".into(),
                    kind: IntentKind::QuoteObserve,
                    exchange_scope: vec![Exchange::Binance],
                    symbol_scope: vec![Symbol::BtcUsdt],
                    side: SignalSide::Neutral,
                    strength: 0.0,
                    confidence: 1.0,
                    reference_price: Some(50_000.0),
                    derived_metrics: BTreeMap::new(),
                    reason: "binance".into(),
                    triggered_at_ms: 10,
                    ttl_ms: 1000,
                    trace_id: "trace".into(),
                },
                IntentSignal {
                    signal_id: "s2".into(),
                    intent_id: "intent_okx_quote".into(),
                    kind: IntentKind::QuoteObserve,
                    exchange_scope: vec![Exchange::Okx],
                    symbol_scope: vec![Symbol::BtcUsdt],
                    side: SignalSide::Neutral,
                    strength: 0.0,
                    confidence: 1.0,
                    reference_price: Some(50_350.0),
                    derived_metrics: BTreeMap::new(),
                    reason: "okx".into(),
                    triggered_at_ms: 10,
                    ttl_ms: 1000,
                    trace_id: "trace".into(),
                },
            ],
            core_ir: &core_ir,
            portfolio: &sample_portfolio_with_position(Exchange::Okx, 1.0, 50_350.0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.decisions.len(), 1);
        assert_eq!(output.events.len(), 1);
        assert_eq!(
            output.events[0].payload["provider_key"],
            "builtin.agent.default"
        );
    }

    #[test]
    fn builtin_agent_module_emits_decision_from_spread_signal() {
        let module = BuiltinAgentModule;
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_arb",
            AgentPolicyKind::CrossVenueArbitrage,
            vec!["intent_spread"],
            vec![SignalRule {
                signal_id: "spread_observe".into(),
                indicator_id: "intent_spread".into(),
                signal_kind: SignalKind::Observe,
                condition: qrpc_core_ir::ScalarExpr::RawText {
                    source: "spread".into(),
                },
            }],
            None,
            0.4,
            Some(30.0),
        );
        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "fast",
            signals: &[IntentSignal {
                signal_id: "s1".into(),
                intent_id: "intent_spread".into(),
                kind: IntentKind::QuoteObserve,
                exchange_scope: vec![Exchange::Binance, Exchange::Okx],
                symbol_scope: vec![Symbol::BtcUsdt],
                side: SignalSide::Neutral,
                strength: 0.007,
                confidence: 0.95,
                reference_price: Some(50_000.0),
                derived_metrics: BTreeMap::from([
                    ("buy_mid_price".into(), 50_000.0),
                    ("sell_mid_price".into(), 50_350.0),
                    ("spread_ratio".into(), 0.007),
                ]),
                reason: "spread observe Binance->Okx 70bps".into(),
                triggered_at_ms: 10,
                ttl_ms: 1000,
                trace_id: "trace".into(),
            }],
            core_ir: &core_ir,
            portfolio: &sample_portfolio_with_position(Exchange::Okx, 1.0, 50_350.0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.decisions.len(), 1);
        assert_eq!(
            output.decisions[0].exchange_targets,
            vec![Exchange::Binance, Exchange::Okx]
        );
        assert_eq!(output.events.len(), 1);
    }

    #[test]
    fn long_term_agent_inherits_exchange_from_signal_scope() {
        let module = BuiltinAgentModule;
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_long_term",
            AgentPolicyKind::WeightedSignals,
            vec!["intent_long_buy", "intent_long_sell"],
            vec![
                SignalRule {
                    signal_id: "long_buy".into(),
                    indicator_id: "intent_long_buy".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
                SignalRule {
                    signal_id: "long_sell".into(),
                    indicator_id: "intent_long_sell".into(),
                    signal_kind: SignalKind::Short,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "short".into(),
                    },
                },
            ],
            Some(0.05),
            0.5,
            None,
        );
        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[IntentSignal {
                signal_id: "s1".into(),
                intent_id: "intent_long_buy".into(),
                kind: IntentKind::LongTermBuy,
                exchange_scope: vec![Exchange::Okx],
                symbol_scope: vec![Symbol::BtcUsdt],
                side: SignalSide::Long,
                strength: 0.8,
                confidence: 1.0,
                reference_price: Some(70_000.0),
                derived_metrics: BTreeMap::new(),
                reason: "trend".into(),
                triggered_at_ms: 10,
                ttl_ms: 1000,
                trace_id: "trace".into(),
            }],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.decisions.len(), 1);
        assert_eq!(output.decisions[0].exchange_targets, vec![Exchange::Okx]);
        assert_eq!(
            output.decisions[0].proposed_actions[0].exchange,
            Exchange::Okx
        );
    }

    #[test]
    fn long_term_agent_inherits_symbol_from_signal_scope() {
        let module = BuiltinAgentModule;
        let eth = Symbol::parse("ETHUSDT");
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_long_term",
            AgentPolicyKind::WeightedSignals,
            vec!["intent_long_buy"],
            vec![SignalRule {
                signal_id: "long_buy".into(),
                indicator_id: "intent_long_buy".into(),
                signal_kind: SignalKind::Long,
                condition: qrpc_core_ir::ScalarExpr::RawText {
                    source: "long".into(),
                },
            }],
            Some(0.05),
            0.5,
            None,
        );
        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[IntentSignal {
                signal_id: "s1".into(),
                intent_id: "intent_long_buy".into(),
                kind: IntentKind::LongTermBuy,
                exchange_scope: vec![Exchange::Binance],
                symbol_scope: vec![eth.clone()],
                side: SignalSide::Long,
                strength: 0.8,
                confidence: 1.0,
                reference_price: Some(4_000.0),
                derived_metrics: BTreeMap::new(),
                reason: "trend".into(),
                triggered_at_ms: 10,
                ttl_ms: 1000,
                trace_id: "trace".into(),
            }],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.decisions.len(), 1);
        assert_eq!(output.decisions[0].symbol, eth);
    }

    #[test]
    fn portfolio_rebalance_agent_emits_equal_weight_portfolio_target() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc", "intent_eth"],
            vec![
                SignalRule {
                    signal_id: "btc_signal".into(),
                    indicator_id: "intent_btc".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
                SignalRule {
                    signal_id: "eth_signal".into(),
                    indicator_id: "intent_eth".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
            ],
            Some(0.05),
            0.8,
            None,
        );
        let mut portfolio =
            sample_portfolio_with_symbol_position(Exchange::Binance, btc.clone(), 1.4, 50_000.0);
        portfolio.cash_balance = 30_000.0;
        portfolio.available_cash_balance = 30_000.0;
        portfolio.total_net_notional = 70_000.0;
        portfolio.total_gross_notional = 70_000.0;
        portfolio.total_leverage = 70_000.0 / 100_000.0;

        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[
                IntentSignal {
                    signal_id: "s_btc".into(),
                    intent_id: "intent_btc".into(),
                    kind: IntentKind::LongTermBuy,
                    exchange_scope: vec![Exchange::Binance],
                    symbol_scope: vec![btc.clone()],
                    side: SignalSide::Long,
                    strength: 0.9,
                    confidence: 1.0,
                    reference_price: Some(50_000.0),
                    derived_metrics: BTreeMap::new(),
                    reason: "btc selected".into(),
                    triggered_at_ms: 10,
                    ttl_ms: 1000,
                    trace_id: "trace".into(),
                },
                IntentSignal {
                    signal_id: "s_eth".into(),
                    intent_id: "intent_eth".into(),
                    kind: IntentKind::LongTermBuy,
                    exchange_scope: vec![Exchange::Binance],
                    symbol_scope: vec![eth.clone()],
                    side: SignalSide::Long,
                    strength: 0.8,
                    confidence: 1.0,
                    reference_price: Some(4_000.0),
                    derived_metrics: BTreeMap::new(),
                    reason: "eth selected".into(),
                    triggered_at_ms: 10,
                    ttl_ms: 1000,
                    trace_id: "trace".into(),
                },
            ],
            core_ir: &core_ir,
            portfolio: &portfolio,
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.decisions.len(), 1);
        let target = output.decisions[0]
            .portfolio_target_decision
            .as_ref()
            .expect("portfolio target decision");
        assert!(output.decisions[0].proposed_actions.is_empty());
        assert_eq!(target.target.target_weights.len(), 2);
        let btc_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == btc)
            .expect("btc target");
        let eth_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == eth)
            .expect("eth target");
        assert!((btc_weight.current_weight - 0.7).abs() < 0.001);
        assert!((btc_weight.target_weight - 0.5).abs() < 0.001);
        assert!((eth_weight.current_weight - 0.0).abs() < 0.001);
        assert!((eth_weight.target_weight - 0.5).abs() < 0.001);
    }

    #[test]
    fn portfolio_rebalance_agent_emits_fixed_weight_portfolio_target() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc", "intent_eth"],
            vec![
                SignalRule {
                    signal_id: "btc_signal".into(),
                    indicator_id: "intent_btc".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
                SignalRule {
                    signal_id: "eth_signal".into(),
                    indicator_id: "intent_eth".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
            ],
            Some(0.05),
            1.0,
            None,
        );
        let mut core_ir = core_ir;
        core_ir.agent_policies[0].rebalance_symbols =
            vec![btc.as_str().to_string(), eth.as_str().to_string()];
        core_ir.agent_policies[0].rebalance_allocation_kind = Some("fixed_weights".into());
        core_ir.agent_policies[0].rebalance_target_weights = vec![0.7, 0.3];

        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[
                IntentSignal {
                    signal_id: "s_btc".into(),
                    intent_id: "intent_btc".into(),
                    kind: IntentKind::LongTermBuy,
                    exchange_scope: vec![Exchange::Binance],
                    symbol_scope: vec![btc.clone()],
                    side: SignalSide::Long,
                    strength: 0.9,
                    confidence: 1.0,
                    reference_price: Some(50_000.0),
                    derived_metrics: BTreeMap::new(),
                    reason: "btc selected".into(),
                    triggered_at_ms: 10,
                    ttl_ms: 1000,
                    trace_id: "trace".into(),
                },
                IntentSignal {
                    signal_id: "s_eth".into(),
                    intent_id: "intent_eth".into(),
                    kind: IntentKind::LongTermBuy,
                    exchange_scope: vec![Exchange::Binance],
                    symbol_scope: vec![eth.clone()],
                    side: SignalSide::Long,
                    strength: 0.8,
                    confidence: 1.0,
                    reference_price: Some(4_000.0),
                    derived_metrics: BTreeMap::new(),
                    reason: "eth selected".into(),
                    triggered_at_ms: 10,
                    ttl_ms: 1000,
                    trace_id: "trace".into(),
                },
            ],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        let target = output.decisions[0]
            .portfolio_target_decision
            .as_ref()
            .expect("portfolio target");
        let btc_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == btc)
            .expect("btc target");
        let eth_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == eth)
            .expect("eth target");
        assert!((btc_weight.target_weight - 0.7).abs() < 1e-9);
        assert!((eth_weight.target_weight - 0.3).abs() < 1e-9);
    }

    #[test]
    fn portfolio_rebalance_agent_emits_rank_weight_portfolio_target() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let sol = Symbol::parse("SOLUSDT");
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc", "intent_eth", "intent_sol"],
            vec![
                SignalRule {
                    signal_id: "btc_signal".into(),
                    indicator_id: "intent_btc".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
                SignalRule {
                    signal_id: "eth_signal".into(),
                    indicator_id: "intent_eth".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
                SignalRule {
                    signal_id: "sol_signal".into(),
                    indicator_id: "intent_sol".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
            ],
            Some(0.05),
            1.0,
            None,
        );
        let mut core_ir = core_ir;
        core_ir.agent_policies[0].rebalance_symbols = vec![
            btc.as_str().to_string(),
            eth.as_str().to_string(),
            sol.as_str().to_string(),
        ];
        core_ir.agent_policies[0].rebalance_allocation_kind = Some("rank_weight".into());
        core_ir.agent_policies[0].rebalance_rank_method = Some("inverse_rank".into());

        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[
                sample_long_signal("intent_btc", btc.clone(), 50_000.0, 0.9),
                sample_long_signal("intent_eth", eth.clone(), 4_000.0, 0.6),
                sample_long_signal("intent_sol", sol.clone(), 150.0, 0.3),
            ],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        let target = output.decisions[0]
            .portfolio_target_decision
            .as_ref()
            .expect("portfolio target");
        let btc_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == btc)
            .expect("btc");
        let eth_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == eth)
            .expect("eth");
        let sol_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == sol)
            .expect("sol");
        assert!((btc_weight.target_weight - (1.0 / 1.8333333333333333)).abs() < 0.001);
        assert!((eth_weight.target_weight - ((1.0 / 2.0) / 1.8333333333333333)).abs() < 0.001);
        assert!((sol_weight.target_weight - ((1.0 / 3.0) / 1.8333333333333333)).abs() < 0.001);
    }

    #[test]
    fn portfolio_rebalance_agent_emits_score_weight_portfolio_target() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc", "intent_eth"],
            vec![
                SignalRule {
                    signal_id: "btc_signal".into(),
                    indicator_id: "intent_btc".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
                SignalRule {
                    signal_id: "eth_signal".into(),
                    indicator_id: "intent_eth".into(),
                    signal_kind: SignalKind::Long,
                    condition: qrpc_core_ir::ScalarExpr::RawText {
                        source: "long".into(),
                    },
                },
            ],
            Some(0.05),
            1.0,
            None,
        );
        let mut core_ir = core_ir;
        core_ir.agent_policies[0].rebalance_symbols =
            vec![btc.as_str().to_string(), eth.as_str().to_string()];
        core_ir.agent_policies[0].rebalance_allocation_kind = Some("score_weight".into());
        core_ir.agent_policies[0].rebalance_score_normalize = Some("sum".into());

        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[
                sample_long_signal("intent_btc", btc.clone(), 50_000.0, 0.9),
                sample_long_signal("intent_eth", eth.clone(), 4_000.0, 0.3),
            ],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        let target = output.decisions[0]
            .portfolio_target_decision
            .as_ref()
            .expect("portfolio target");
        let btc_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == btc)
            .expect("btc");
        let eth_weight = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == eth)
            .expect("eth");
        assert!((btc_weight.target_weight - 0.75).abs() < 0.001);
        assert!((eth_weight.target_weight - 0.25).abs() < 0.001);
    }

    #[test]
    fn portfolio_rebalance_agent_sells_universe_member_without_current_signal() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let eth = Symbol::parse("ETHUSDT");
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc"],
            vec![SignalRule {
                signal_id: "btc_signal".into(),
                indicator_id: "intent_btc".into(),
                signal_kind: SignalKind::Long,
                condition: qrpc_core_ir::ScalarExpr::RawText {
                    source: "long".into(),
                },
            }],
            Some(0.05),
            1.0,
            None,
        );
        let mut core_ir = core_ir;
        core_ir.agent_policies[0].rebalance_symbols =
            vec![btc.as_str().to_string(), eth.as_str().to_string()];

        let mut portfolio =
            sample_portfolio_with_symbol_position(Exchange::Binance, eth.clone(), 5.0, 4_000.0);
        portfolio.cash_balance = 80_000.0;
        portfolio.available_cash_balance = 80_000.0;
        portfolio.total_net_notional = 20_000.0;
        portfolio.total_gross_notional = 20_000.0;
        portfolio.total_leverage = 0.2;

        let output = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &[IntentSignal {
                signal_id: "s_btc".into(),
                intent_id: "intent_btc".into(),
                kind: IntentKind::LongTermBuy,
                exchange_scope: vec![Exchange::Binance],
                symbol_scope: vec![btc],
                side: SignalSide::Long,
                strength: 0.9,
                confidence: 1.0,
                reference_price: Some(50_000.0),
                derived_metrics: BTreeMap::new(),
                reason: "btc selected".into(),
                triggered_at_ms: 10,
                ttl_ms: 1000,
                trace_id: "trace".into(),
            }],
            core_ir: &core_ir,
            portfolio: &portfolio,
            last_rebalance_at_ms: &BTreeMap::new(),
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.decisions.len(), 1);
        let target = output.decisions[0]
            .portfolio_target_decision
            .as_ref()
            .expect("portfolio target decision");
        let eth_target = target
            .target
            .target_weights
            .iter()
            .find(|item| item.symbol == eth)
            .expect("eth rebalance exit target");
        assert!((eth_target.current_weight - 0.2).abs() < 0.001);
        assert!((eth_target.target_weight - 0.0).abs() < 0.001);
    }

    #[test]
    fn portfolio_rebalance_agent_respects_daily_cadence() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc"],
            vec![SignalRule {
                signal_id: "btc_signal".into(),
                indicator_id: "intent_btc".into(),
                signal_kind: SignalKind::Long,
                condition: qrpc_core_ir::ScalarExpr::RawText {
                    source: "long".into(),
                },
            }],
            Some(0.05),
            1.0,
            None,
        );
        let mut core_ir = core_ir;
        core_ir.agent_policies[0].rebalance_symbols = vec![btc.as_str().to_string()];
        core_ir.agent_policies[0].rebalance_schedule =
            Some(qrpc_core_ir::RebalanceSchedule::Every1d);

        let signals = vec![IntentSignal {
            signal_id: "s_btc".into(),
            intent_id: "intent_btc".into(),
            kind: IntentKind::LongTermBuy,
            exchange_scope: vec![Exchange::Binance],
            symbol_scope: vec![btc],
            side: SignalSide::Long,
            strength: 0.9,
            confidence: 1.0,
            reference_price: Some(50_000.0),
            derived_metrics: BTreeMap::new(),
            reason: "btc selected".into(),
            triggered_at_ms: 10,
            ttl_ms: 1000,
            trace_id: "trace".into(),
        }];
        let last_rebalance_at_ms = BTreeMap::from([("agent_rebalance".to_string(), 1_000_u64)]);

        let skipped = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &signals,
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &last_rebalance_at_ms,
            now_ms: 1_000 + 3_600_000,
            trace_id: "trace",
        });
        assert!(skipped.decisions.is_empty());
        assert!(skipped.evaluated_rebalance_agent_ids.is_empty());

        let due = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &signals,
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &last_rebalance_at_ms,
            now_ms: 1_000 + 86_400_000,
            trace_id: "trace",
        });
        assert_eq!(due.decisions.len(), 1);
        assert!(due.decisions[0].portfolio_target_decision.is_some());
        assert!(due
            .evaluated_rebalance_agent_ids
            .contains("agent_rebalance"));
    }

    #[test]
    fn portfolio_rebalance_agent_respects_weekly_cadence() {
        let module = BuiltinAgentModule;
        let btc = Symbol::BtcUsdt;
        let core_ir = sample_core_ir_with_agent_policy(
            "agent_rebalance",
            AgentPolicyKind::PortfolioRebalance,
            vec!["intent_btc"],
            vec![SignalRule {
                signal_id: "btc_signal".into(),
                indicator_id: "intent_btc".into(),
                signal_kind: SignalKind::Long,
                condition: qrpc_core_ir::ScalarExpr::RawText {
                    source: "long".into(),
                },
            }],
            Some(0.05),
            1.0,
            None,
        );
        let mut core_ir = core_ir;
        core_ir.agent_policies[0].rebalance_symbols = vec![btc.as_str().to_string()];
        core_ir.agent_policies[0].rebalance_schedule = Some(RebalanceSchedule::Weekly);

        let signals = vec![IntentSignal {
            signal_id: "s_btc".into(),
            intent_id: "intent_btc".into(),
            kind: IntentKind::LongTermBuy,
            exchange_scope: vec![Exchange::Binance],
            symbol_scope: vec![btc],
            side: SignalSide::Long,
            strength: 0.9,
            confidence: 1.0,
            reference_price: Some(50_000.0),
            derived_metrics: BTreeMap::new(),
            reason: "btc selected".into(),
            triggered_at_ms: 10,
            ttl_ms: 1000,
            trace_id: "trace".into(),
        }];
        let last_rebalance_at_ms = BTreeMap::from([("agent_rebalance".to_string(), 1_000_u64)]);

        let skipped = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &signals,
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &last_rebalance_at_ms,
            now_ms: 1_000 + 6 * 86_400_000,
            trace_id: "trace",
        });
        assert!(skipped.decisions.is_empty());

        let due = module.evaluate_agents(AgentEvaluationRequest {
            cycle_name: "slow",
            signals: &signals,
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_rebalance_at_ms: &last_rebalance_at_ms,
            now_ms: 1_000 + 7 * 86_400_000,
            trace_id: "trace",
        });
        assert_eq!(due.decisions.len(), 1);
        assert!(due
            .evaluated_rebalance_agent_ids
            .contains("agent_rebalance"));
    }

    fn sample_long_signal(
        intent_id: &str,
        symbol: Symbol,
        reference_price: f64,
        strength: f64,
    ) -> IntentSignal {
        IntentSignal {
            signal_id: format!("signal_{intent_id}"),
            intent_id: intent_id.into(),
            kind: IntentKind::LongTermBuy,
            exchange_scope: vec![Exchange::Binance],
            symbol_scope: vec![symbol],
            side: SignalSide::Long,
            strength,
            confidence: 1.0,
            reference_price: Some(reference_price),
            derived_metrics: BTreeMap::new(),
            reason: "selected".into(),
            triggered_at_ms: 10,
            ttl_ms: 1000,
            trace_id: "trace".into(),
        }
    }
}
