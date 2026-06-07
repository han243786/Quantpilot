use super::*;
use qrpc_core::{
    Exchange, PortfolioTarget, PortfolioTargetDecision, Position, ProposedAction, SignalSide,
    Symbol, TargetWeight,
};
use qrpc_core_ir::{
    CoreMetadata, CoreSourceKind, CoreStrategyIr, CoreTimeInForce, ExecutionRule,
    ExecutionSizingKind, RiskPolicy,
};
use std::collections::BTreeMap;

fn sample_core_ir_with_risk(
    observed_agent_ids: Vec<&str>,
    min_action_interval_ms: u64,
) -> CoreStrategyIr {
    CoreStrategyIr {
        ir_version: qrpc_core::CORE_IR_V1_VERSION.to_string(),
        metadata: CoreMetadata {
            strategy_id: "risk_test".into(),
            name: "Risk Test".into(),
            source_kind: CoreSourceKind::RuntimeProtocol,
        },
        data_bindings: vec![],
        indicators: vec![],
        signal_rules: vec![],
        agent_policies: vec![],
        risk_policies: vec![RiskPolicy {
            policy_id: "risk_global".into(),
            name: "Global Risk".into(),
            observed_agent_ids: observed_agent_ids.into_iter().map(str::to_string).collect(),
            max_position_ratio: 0.3,
            max_single_weight: None,
            max_concentration_ratio: None,
            max_symbol_net_exposure_ratio: None,
            max_portfolio_net_exposure_ratio: None,
            max_turnover: None,
            min_trade_weight: None,
            max_new_positions_per_rebalance: None,
            max_total_leverage: 3.0,
            max_exchange_leverage: 3.0,
            min_action_interval_ms,
            enabled: true,
            max_cross_symbol_leverage: None,
        }],
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

#[test]
fn checker_rejects_actions_when_min_interval_not_met() {
    let checker = RiskChecker;
    let core_ir = sample_core_ir_with_risk(vec!["agent_1"], 1_000);
    let decision = AgentDecision {
        decision_id: "decision_1".into(),
        agent_id: "agent_1".into(),
        symbol: Symbol::BtcUsdt,
        exchange_targets: vec![Exchange::Binance],
        net_side: SignalSide::Long,
        net_strength: 0.5,
        portfolio_target_decision: None,
        proposed_actions: vec![ProposedAction {
            exchange: Exchange::Binance,
            side: qrpc_core::OrderSide::Buy,
            quantity_ratio: 0.3,
            reference_price: 50_000.0,
            strategy_tag: "test".into(),
        }],
        reason: "test".into(),
        produced_at_ms: 100,
        trace_id: "trace".into(),
    };
    let mut last_action = BTreeMap::new();
    last_action.insert("agent_1".to_string(), 900);

    let output = checker
        .evaluate(RiskCheckRequest {
            decisions: &[decision],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_action_at_ms: &last_action,
            now_ms: 1_000,
            mode: RiskDecisionMode::Normal,
            trace_id: "trace",
        })
        .unwrap();

    assert!(matches!(output.decisions[0].status, DecisionStatus::Reject));
    assert_eq!(output.events.len(), 1);
    assert_eq!(
        output.events[0].payload["provider_key"],
        "builtin.risk.default"
    );
}

#[test]
fn checker_rejects_spot_sell_without_inventory() {
    let checker = RiskChecker;
    let core_ir = sample_core_ir_with_risk(vec!["agent_1"], 0);
    let decision = AgentDecision {
        decision_id: "decision_1".into(),
        agent_id: "agent_1".into(),
        symbol: Symbol::BtcUsdt,
        exchange_targets: vec![Exchange::Binance],
        net_side: SignalSide::Short,
        net_strength: -0.5,
        portfolio_target_decision: None,
        proposed_actions: vec![ProposedAction {
            exchange: Exchange::Binance,
            side: qrpc_core::OrderSide::Sell,
            quantity_ratio: 0.2,
            reference_price: 50_000.0,
            strategy_tag: "test".into(),
        }],
        reason: "test".into(),
        produced_at_ms: 100,
        trace_id: "trace".into(),
    };
    let mut portfolio = PortfolioState::new(100_000.0, 0);
    portfolio.positions.push(Position {
        exchange: Exchange::Binance,
        symbol: Symbol::BtcUsdt,
        net_qty: 0.0,
        frozen_qty: 0.0,
        avg_entry_price: 0.0,
        mark_price: 50_000.0,
        unrealized_pnl: 0.0,
        realized_pnl: 0.0,
    });

    let output = checker
        .evaluate(RiskCheckRequest {
            decisions: &[decision],
            core_ir: &core_ir,
            portfolio: &portfolio,
            last_action_at_ms: &BTreeMap::new(),
            now_ms: 1_000,
            mode: RiskDecisionMode::Normal,
            trace_id: "trace",
        })
        .unwrap();

    assert!(matches!(output.decisions[0].status, DecisionStatus::Reject));
    assert_eq!(
        output.decisions[0].reason_codes,
        vec![RiskReasonCode::InsufficientInventory]
    );
}

#[test]
fn checker_uses_agent_decision_symbol_for_inventory_checks() {
    let checker = RiskChecker;
    let core_ir = sample_core_ir_with_risk(vec!["agent_1"], 0);
    let eth = Symbol::parse("ETHUSDT");
    let decision = AgentDecision {
        decision_id: "decision_1".into(),
        agent_id: "agent_1".into(),
        symbol: eth.clone(),
        exchange_targets: vec![Exchange::Binance],
        net_side: SignalSide::Short,
        net_strength: -0.5,
        portfolio_target_decision: None,
        proposed_actions: vec![ProposedAction {
            exchange: Exchange::Binance,
            side: qrpc_core::OrderSide::Sell,
            quantity_ratio: 0.2,
            reference_price: 4_000.0,
            strategy_tag: "test".into(),
        }],
        reason: "test".into(),
        produced_at_ms: 100,
        trace_id: "trace".into(),
    };
    let mut portfolio = PortfolioState::new(100_000.0, 0);
    portfolio.positions.push(Position {
        exchange: Exchange::Binance,
        symbol: eth.clone(),
        net_qty: 10.0,
        frozen_qty: 0.0,
        avg_entry_price: 4_000.0,
        mark_price: 4_000.0,
        unrealized_pnl: 0.0,
        realized_pnl: 0.0,
    });

    let output = checker
        .evaluate(RiskCheckRequest {
            decisions: &[decision],
            core_ir: &core_ir,
            portfolio: &portfolio,
            last_action_at_ms: &BTreeMap::new(),
            now_ms: 1_000,
            mode: RiskDecisionMode::Normal,
            trace_id: "trace",
        })
        .unwrap();

    assert!(matches!(
        output.decisions[0].status,
        DecisionStatus::Approve
    ));
    assert_eq!(output.decisions[0].symbol, eth);
}

#[test]
fn checker_preserves_portfolio_target_decisions_for_execution_diff() {
    let checker = RiskChecker;
    let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
    core_ir.risk_policies[0].max_position_ratio = 1.0;
    let btc = Symbol::BtcUsdt;
    let eth = Symbol::parse("ETHUSDT");
    let decision = AgentDecision {
        decision_id: "decision_rebalance".into(),
        agent_id: "agent_rebalance".into(),
        symbol: btc.clone(),
        exchange_targets: vec![Exchange::Binance],
        net_side: SignalSide::Neutral,
        net_strength: 0.0,
        portfolio_target_decision: Some(PortfolioTargetDecision {
            target_id: "target_rebalance".into(),
            target: PortfolioTarget {
                allocation_kind: "equal_weight".into(),
                target_weights: vec![
                    TargetWeight {
                        exchange: Exchange::Binance,
                        symbol: btc.clone(),
                        target_weight: 0.5,
                        current_weight: 0.7,
                        reference_price: 50_000.0,
                        signal_score: Some(0.9),
                    },
                    TargetWeight {
                        exchange: Exchange::Binance,
                        symbol: eth.clone(),
                        target_weight: 0.5,
                        current_weight: 0.0,
                        reference_price: 4_000.0,
                        signal_score: Some(0.8),
                    },
                ],
            },
            reason: "equal weight".into(),
        }),
        proposed_actions: Vec::new(),
        reason: "rebalance".into(),
        produced_at_ms: 100,
        trace_id: "trace".into(),
    };

    let output = checker
        .evaluate(RiskCheckRequest {
            decisions: &[decision],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_action_at_ms: &BTreeMap::new(),
            now_ms: 1_000,
            mode: RiskDecisionMode::Normal,
            trace_id: "trace",
        })
        .unwrap();

    assert!(matches!(
        output.decisions[0].status,
        DecisionStatus::Approve
    ));
    assert!(output.decisions[0].adjusted_actions.is_empty());
    let target = output.decisions[0]
        .adjusted_portfolio_target_decision
        .as_ref()
        .expect("adjusted portfolio target");
    assert_eq!(target.target.target_weights.len(), 2);
    assert_eq!(target.target.target_weights[0].symbol, btc);
    assert_eq!(target.target.target_weights[1].symbol, eth);
}

#[test]
fn checker_clamps_portfolio_target_to_max_single_weight() {
    let checker = RiskChecker;
    let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
    core_ir.risk_policies[0].max_single_weight = Some(0.3);
    let btc = Symbol::BtcUsdt;
    let decision = AgentDecision {
        decision_id: "decision_rebalance".into(),
        agent_id: "agent_rebalance".into(),
        symbol: btc.clone(),
        exchange_targets: vec![Exchange::Binance],
        net_side: SignalSide::Neutral,
        net_strength: 0.0,
        portfolio_target_decision: Some(PortfolioTargetDecision {
            target_id: "target_rebalance".into(),
            target: PortfolioTarget {
                allocation_kind: "fixed".into(),
                target_weights: vec![TargetWeight {
                    exchange: Exchange::Binance,
                    symbol: btc.clone(),
                    target_weight: 0.8,
                    current_weight: 0.0,
                    reference_price: 50_000.0,
                    signal_score: Some(1.0),
                }],
            },
            reason: "fixed weight".into(),
        }),
        proposed_actions: Vec::new(),
        reason: "rebalance".into(),
        produced_at_ms: 100,
        trace_id: "trace".into(),
    };

    let output = checker
        .evaluate(RiskCheckRequest {
            decisions: &[decision],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_action_at_ms: &BTreeMap::new(),
            now_ms: 1_000,
            mode: RiskDecisionMode::Normal,
            trace_id: "trace",
        })
        .unwrap();

    assert!(matches!(output.decisions[0].status, DecisionStatus::Clamp));
    assert_eq!(
        output.decisions[0].reason_codes,
        vec![RiskReasonCode::ExceedSingleWeight]
    );
    let target = output.decisions[0]
        .adjusted_portfolio_target_decision
        .as_ref()
        .expect("adjusted portfolio target");
    assert!((target.target.target_weights[0].target_weight - 0.3).abs() < 1e-9);
}

#[test]
fn checker_scales_portfolio_target_to_max_turnover() {
    let checker = RiskChecker;
    let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
    core_ir.risk_policies[0].max_position_ratio = 1.0;
    core_ir.risk_policies[0].max_turnover = Some(0.35);
    let btc = Symbol::BtcUsdt;
    let eth = Symbol::parse("ETHUSDT");
    let mut portfolio = PortfolioState::new(100_000.0, 0);
    portfolio.positions.push(Position {
        exchange: Exchange::Binance,
        symbol: btc.clone(),
        net_qty: 1.4,
        frozen_qty: 0.0,
        avg_entry_price: 50_000.0,
        mark_price: 50_000.0,
        unrealized_pnl: 0.0,
        realized_pnl: 0.0,
    });
    portfolio.cash_balance = 30_000.0;
    portfolio.available_cash_balance = 30_000.0;
    portfolio.total_net_notional = 70_000.0;
    portfolio.total_gross_notional = 70_000.0;
    portfolio.total_leverage = 0.7;

    let decision = AgentDecision {
        decision_id: "decision_rebalance".into(),
        agent_id: "agent_rebalance".into(),
        symbol: btc.clone(),
        exchange_targets: vec![Exchange::Binance],
        net_side: SignalSide::Neutral,
        net_strength: 0.0,
        portfolio_target_decision: Some(PortfolioTargetDecision {
            target_id: "target_rebalance".into(),
            target: PortfolioTarget {
                allocation_kind: "equal_weight".into(),
                target_weights: vec![
                    TargetWeight {
                        exchange: Exchange::Binance,
                        symbol: btc.clone(),
                        target_weight: 0.5,
                        current_weight: 0.7,
                        reference_price: 50_000.0,
                        signal_score: Some(0.9),
                    },
                    TargetWeight {
                        exchange: Exchange::Binance,
                        symbol: eth.clone(),
                        target_weight: 0.5,
                        current_weight: 0.0,
                        reference_price: 4_000.0,
                        signal_score: Some(0.8),
                    },
                ],
            },
            reason: "equal weight".into(),
        }),
        proposed_actions: Vec::new(),
        reason: "rebalance".into(),
        produced_at_ms: 100,
        trace_id: "trace".into(),
    };

    let output = checker
        .evaluate(RiskCheckRequest {
            decisions: &[decision],
            core_ir: &core_ir,
            portfolio: &portfolio,
            last_action_at_ms: &BTreeMap::new(),
            now_ms: 1_000,
            mode: RiskDecisionMode::Normal,
            trace_id: "trace",
        })
        .unwrap();

    assert!(matches!(output.decisions[0].status, DecisionStatus::Clamp));
    assert_eq!(
        output.decisions[0].reason_codes,
        vec![RiskReasonCode::ExceedTurnover]
    );
    let target = output.decisions[0]
        .adjusted_portfolio_target_decision
        .as_ref()
        .expect("adjusted portfolio target");
    let btc_target = target
        .target
        .target_weights
        .iter()
        .find(|item| item.symbol == btc)
        .expect("btc");
    let eth_target = target
        .target
        .target_weights
        .iter()
        .find(|item| item.symbol == eth)
        .expect("eth");
    assert!((btc_target.target_weight - 0.6).abs() < 1e-9);
    assert!((eth_target.target_weight - 0.25).abs() < 1e-9);
}

#[test]
fn checker_removes_small_portfolio_target_trades() {
    let checker = RiskChecker;
    let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
    core_ir.risk_policies[0].max_position_ratio = 1.0;
    core_ir.risk_policies[0].min_trade_weight = Some(0.02);
    let btc = Symbol::BtcUsdt;
    let decision = AgentDecision {
        decision_id: "decision_rebalance".into(),
        agent_id: "agent_rebalance".into(),
        symbol: btc.clone(),
        exchange_targets: vec![Exchange::Binance],
        net_side: SignalSide::Neutral,
        net_strength: 0.0,
        portfolio_target_decision: Some(PortfolioTargetDecision {
            target_id: "target_rebalance".into(),
            target: PortfolioTarget {
                allocation_kind: "fixed".into(),
                target_weights: vec![TargetWeight {
                    exchange: Exchange::Binance,
                    symbol: btc.clone(),
                    target_weight: 0.515,
                    current_weight: 0.5,
                    reference_price: 50_000.0,
                    signal_score: Some(1.0),
                }],
            },
            reason: "small rebalance".into(),
        }),
        proposed_actions: Vec::new(),
        reason: "rebalance".into(),
        produced_at_ms: 100,
        trace_id: "trace".into(),
    };
    let mut portfolio = PortfolioState::new(100_000.0, 0);
    portfolio.positions.push(Position {
        exchange: Exchange::Binance,
        symbol: btc.clone(),
        net_qty: 1.0,
        frozen_qty: 0.0,
        avg_entry_price: 50_000.0,
        mark_price: 50_000.0,
        unrealized_pnl: 0.0,
        realized_pnl: 0.0,
    });
    portfolio.cash_balance = 50_000.0;
    portfolio.available_cash_balance = 50_000.0;
    portfolio.total_net_notional = 50_000.0;
    portfolio.total_gross_notional = 50_000.0;
    portfolio.total_leverage = 0.5;

    let output = checker
        .evaluate(RiskCheckRequest {
            decisions: &[decision],
            core_ir: &core_ir,
            portfolio: &portfolio,
            last_action_at_ms: &BTreeMap::new(),
            now_ms: 1_000,
            mode: RiskDecisionMode::Normal,
            trace_id: "trace",
        })
        .unwrap();

    assert!(matches!(output.decisions[0].status, DecisionStatus::Clamp));
    assert_eq!(
        output.decisions[0].reason_codes,
        vec![RiskReasonCode::TradeBelowMinimum]
    );
    let target = output.decisions[0]
        .adjusted_portfolio_target_decision
        .as_ref()
        .expect("adjusted portfolio target");
    assert!((target.target.target_weights[0].target_weight - 0.5).abs() < 1e-9);
}

#[test]
fn checker_limits_new_positions_per_rebalance() {
    let checker = RiskChecker;
    let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
    core_ir.risk_policies[0].max_position_ratio = 1.0;
    core_ir.risk_policies[0].max_new_positions_per_rebalance = Some(1);
    let btc = Symbol::BtcUsdt;
    let eth = Symbol::parse("ETHUSDT");
    let sol = Symbol::parse("SOLUSDT");
    let decision = AgentDecision {
        decision_id: "decision_rebalance".into(),
        agent_id: "agent_rebalance".into(),
        symbol: btc.clone(),
        exchange_targets: vec![Exchange::Binance],
        net_side: SignalSide::Neutral,
        net_strength: 0.0,
        portfolio_target_decision: Some(PortfolioTargetDecision {
            target_id: "target_rebalance".into(),
            target: PortfolioTarget {
                allocation_kind: "rank_weight".into(),
                target_weights: vec![
                    TargetWeight {
                        exchange: Exchange::Binance,
                        symbol: btc.clone(),
                        target_weight: 0.3,
                        current_weight: 0.0,
                        reference_price: 50_000.0,
                        signal_score: Some(0.95),
                    },
                    TargetWeight {
                        exchange: Exchange::Binance,
                        symbol: eth.clone(),
                        target_weight: 0.2,
                        current_weight: 0.0,
                        reference_price: 4_000.0,
                        signal_score: Some(0.7),
                    },
                    TargetWeight {
                        exchange: Exchange::Binance,
                        symbol: sol.clone(),
                        target_weight: 0.1,
                        current_weight: 0.0,
                        reference_price: 150.0,
                        signal_score: Some(0.6),
                    },
                ],
            },
            reason: "limit new names".into(),
        }),
        proposed_actions: Vec::new(),
        reason: "rebalance".into(),
        produced_at_ms: 100,
        trace_id: "trace".into(),
    };

    let output = checker
        .evaluate(RiskCheckRequest {
            decisions: &[decision],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_action_at_ms: &BTreeMap::new(),
            now_ms: 1_000,
            mode: RiskDecisionMode::Normal,
            trace_id: "trace",
        })
        .unwrap();

    assert!(matches!(output.decisions[0].status, DecisionStatus::Clamp));
    assert_eq!(
        output.decisions[0].reason_codes,
        vec![RiskReasonCode::ExceedNewPositionsLimit]
    );
    let target = output.decisions[0]
        .adjusted_portfolio_target_decision
        .as_ref()
        .expect("adjusted portfolio target");
    let btc_target = target
        .target
        .target_weights
        .iter()
        .find(|item| item.symbol == btc)
        .expect("btc");
    let eth_target = target
        .target
        .target_weights
        .iter()
        .find(|item| item.symbol == eth)
        .expect("eth");
    let sol_target = target
        .target
        .target_weights
        .iter()
        .find(|item| item.symbol == sol)
        .expect("sol");
    assert!((btc_target.target_weight - 0.3).abs() < 1e-9);
    assert!((eth_target.target_weight - 0.0).abs() < 1e-9);
    assert!((sol_target.target_weight - 0.0).abs() < 1e-9);
}

#[test]
fn checker_limits_portfolio_target_to_portfolio_net_exposure() {
    let checker = RiskChecker;
    let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
    core_ir.risk_policies[0].max_position_ratio = 1.0;
    core_ir.risk_policies[0].max_portfolio_net_exposure_ratio = Some(0.6);
    let decision = AgentDecision {
        decision_id: "decision_rebalance".into(),
        agent_id: "agent_rebalance".into(),
        symbol: Symbol::BtcUsdt,
        exchange_targets: vec![Exchange::Binance],
        net_side: SignalSide::Neutral,
        net_strength: 0.0,
        portfolio_target_decision: Some(PortfolioTargetDecision {
            target_id: "target_rebalance".into(),
            target: PortfolioTarget {
                allocation_kind: "fixed_weights".into(),
                target_weights: vec![
                    TargetWeight {
                        exchange: Exchange::Binance,
                        symbol: Symbol::BtcUsdt,
                        target_weight: 0.4,
                        current_weight: 0.0,
                        reference_price: 50_000.0,
                        signal_score: Some(0.8),
                    },
                    TargetWeight {
                        exchange: Exchange::Binance,
                        symbol: Symbol::parse("ETHUSDT"),
                        target_weight: 0.3,
                        current_weight: 0.0,
                        reference_price: 3_000.0,
                        signal_score: Some(0.6),
                    },
                ],
            },
            reason: "rebalance".into(),
        }),
        proposed_actions: vec![],
        reason: "rebalance".into(),
        produced_at_ms: 0,
        trace_id: "trace".into(),
    };

    let output = checker
        .evaluate(RiskCheckRequest {
            decisions: &[decision],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_action_at_ms: &BTreeMap::new(),
            now_ms: 1_000,
            mode: RiskDecisionMode::Normal,
            trace_id: "trace",
        })
        .expect("risk evaluation");

    assert_eq!(output.decisions[0].status, DecisionStatus::Clamp);
    assert_eq!(
        output.decisions[0].reason_codes,
        vec![RiskReasonCode::ExceedPortfolioNetExposure]
    );
    let target = output.decisions[0]
        .adjusted_portfolio_target_decision
        .as_ref()
        .expect("adjusted portfolio target");
    assert!((target.target.target_weights[0].target_weight - 0.342857142857).abs() < 1e-6);
    assert!((target.target.target_weights[1].target_weight - 0.257142857142).abs() < 1e-6);
}

#[test]
fn checker_limits_action_list_to_symbol_net_exposure() {
    let checker = RiskChecker;
    let mut core_ir = sample_core_ir_with_risk(vec!["agent_1"], 0);
    core_ir.risk_policies[0].max_position_ratio = 1.0;
    core_ir.risk_policies[0].max_symbol_net_exposure_ratio = Some(0.25);

    let mut portfolio = PortfolioState::new(100_000.0, 0);
    portfolio.positions.push(Position {
        exchange: Exchange::Binance,
        symbol: Symbol::BtcUsdt,
        net_qty: 0.4,
        frozen_qty: 0.0,
        avg_entry_price: 50_000.0,
        mark_price: 50_000.0,
        unrealized_pnl: 0.0,
        realized_pnl: 0.0,
    });
    portfolio.total_net_notional = 20_000.0;

    let decision = AgentDecision {
        decision_id: "decision_1".into(),
        agent_id: "agent_1".into(),
        symbol: Symbol::BtcUsdt,
        exchange_targets: vec![Exchange::Binance, Exchange::Okx],
        net_side: SignalSide::Long,
        net_strength: 0.5,
        portfolio_target_decision: None,
        proposed_actions: vec![
            ProposedAction {
                exchange: Exchange::Binance,
                side: qrpc_core::OrderSide::Buy,
                quantity_ratio: 0.2,
                reference_price: 50_000.0,
                strategy_tag: "test".into(),
            },
            ProposedAction {
                exchange: Exchange::Okx,
                side: qrpc_core::OrderSide::Buy,
                quantity_ratio: 0.2,
                reference_price: 50_000.0,
                strategy_tag: "test".into(),
            },
        ],
        reason: "test".into(),
        produced_at_ms: 100,
        trace_id: "trace".into(),
    };

    let output = checker
        .evaluate(RiskCheckRequest {
            decisions: &[decision],
            core_ir: &core_ir,
            portfolio: &portfolio,
            last_action_at_ms: &BTreeMap::new(),
            now_ms: 1_000,
            mode: RiskDecisionMode::Normal,
            trace_id: "trace",
        })
        .expect("risk evaluation");

    assert_eq!(output.decisions[0].status, DecisionStatus::Clamp);
    assert!(output.decisions[0]
        .reason_codes
        .contains(&RiskReasonCode::ExceedSymbolNetExposure));
    let total_buy = output.decisions[0]
        .adjusted_actions
        .iter()
        .map(|action| action.quantity_ratio)
        .sum::<f64>();
    assert!((total_buy - (0.25 - 20_000.0 / 120_000.0)).abs() < 1e-9);
}

#[test]
fn risk_events_include_explanation_and_pre_post_sizing() {
    let checker = RiskChecker;
    let mut core_ir = sample_core_ir_with_risk(vec!["agent_rebalance"], 0);
    core_ir.risk_policies[0].max_single_weight = Some(0.3);
    let btc = Symbol::BtcUsdt;
    let decision = AgentDecision {
        decision_id: "decision_rebalance".into(),
        agent_id: "agent_rebalance".into(),
        symbol: btc.clone(),
        exchange_targets: vec![Exchange::Binance],
        net_side: SignalSide::Neutral,
        net_strength: 0.0,
        portfolio_target_decision: Some(PortfolioTargetDecision {
            target_id: "target_rebalance".into(),
            target: PortfolioTarget {
                allocation_kind: "fixed".into(),
                target_weights: vec![TargetWeight {
                    exchange: Exchange::Binance,
                    symbol: btc,
                    target_weight: 0.8,
                    current_weight: 0.0,
                    reference_price: 50_000.0,
                    signal_score: Some(1.0),
                }],
            },
            reason: "fixed weight".into(),
        }),
        proposed_actions: Vec::new(),
        reason: "rebalance".into(),
        produced_at_ms: 100,
        trace_id: "trace".into(),
    };

    let output = checker
        .evaluate(RiskCheckRequest {
            decisions: &[decision],
            core_ir: &core_ir,
            portfolio: &PortfolioState::new(100_000.0, 0),
            last_action_at_ms: &BTreeMap::new(),
            now_ms: 1_000,
            mode: RiskDecisionMode::Normal,
            trace_id: "trace",
        })
        .unwrap();

    let payload = &output.events[0].payload;
    assert_eq!(
        payload["reason_text"],
        "portfolio target clamped by [ExceedSingleWeight] (execution_venue=paper)"
    );
    assert_eq!(payload["limit_triggered"], "max_single_weight");
    assert_eq!(payload["sizing_mode"], "portfolio_target");
    assert_eq!(payload["pre_risk"]["max_target_weight"], 0.8);
    assert_eq!(payload["post_risk"]["max_target_weight"], 0.3);
    assert_eq!(
        payload["explanation_summary"],
        "Risk clamped sizing after triggering max_single_weight."
    );
}
