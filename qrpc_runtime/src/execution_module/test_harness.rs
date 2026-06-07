use super::*;
use qrpc_core::{
    DataQualitySnapshot, Exchange, ExecutionStatus, MarketType, PortfolioTarget,
    PortfolioTargetDecision, QuoteSnapshot, RiskDecisionMode, RiskReasonCode, SourceStatus,
    TargetWeight,
};
use qrpc_core_ir::{CoreMetadata, CoreSourceKind, CoreTimeInForce, ExecutionRule};
use std::collections::BTreeMap;

fn sample_quote(mid_price: f64) -> NormalizedMarketData {
    NormalizedMarketData::Quote(QuoteSnapshot {
        data_id: "binance_btc_quote".into(),
        exchange: Exchange::Binance,
        symbol: Symbol::BtcUsdt,
        market_type: MarketType::Spot,
        best_bid: mid_price - 5.0,
        best_ask: mid_price + 5.0,
        bid_size: 10.0,
        ask_size: 10.0,
        mid_price,
        ts_ms: 10,
        source_latency_ms: 0,
        source_status: SourceStatus::Healthy,
        data_quality: DataQualitySnapshot::default(),
    })
}

fn sample_core_ir() -> CoreStrategyIr {
    CoreStrategyIr {
        ir_version: qrpc_core::CORE_IR_V1_VERSION.to_string(),
        metadata: CoreMetadata {
            strategy_id: "execution_test".into(),
            name: "Execution Test".into(),
            source_kind: CoreSourceKind::RuntimeProtocol,
        },
        data_bindings: vec![],
        indicators: vec![],
        signal_rules: vec![],
        agent_policies: vec![],
        risk_policies: vec![],
        edges: vec![],
        execution: ExecutionRule {
            execution_id: "execution.paper".into(),
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
fn builtin_execution_module_plans_orders_from_risk_decisions() {
    let module = BuiltinExecutionModule::default();
    let core_ir = sample_core_ir();
    let decision = RiskDecision {
        risk_decision_id: "risk_1".into(),
        risk_id: "risk_global".into(),
        agent_decision_id: "decision_1".into(),
        symbol: Symbol::BtcUsdt,
        status: DecisionStatus::Approve,
        mode: RiskDecisionMode::Normal,
        adjusted_portfolio_target_decision: None,
        adjusted_actions: vec![qrpc_core::ProposedAction {
            exchange: Exchange::Binance,
            side: OrderSide::Buy,
            quantity_ratio: 0.2,
            reference_price: 50_000.0,
            strategy_tag: "test".into(),
        }],
        reason_codes: vec![RiskReasonCode::WithinLimit],
        reason_text: "approved".into(),
        produced_at_ms: 10,
        trace_id: "trace".into(),
    };

    let output = module.plan_execution(ExecutionPlanningRequest {
        risk_decisions: &[decision],
        core_ir: &core_ir,
        normalized_data: &[sample_quote(50_000.0)],
        portfolio: &PortfolioState::new(100_000.0, 0),
        now_ms: 10,
        trace_id: "trace",
    });

    assert_eq!(output.plans.len(), 1);
    assert_eq!(output.events.len(), 1);
    assert_eq!(
        output.events[0].payload["explanation_summary"],
        "Execution planned 1 order(s) from risk_adjusted_actions using equity_notional_ratio sizing."
    );
    assert_eq!(
        output.events[0].payload["sizing_source"],
        "risk_adjusted_actions"
    );
    assert_eq!(
        output.events[0].payload["order_previews"][0]["order_type_decision_reason"],
        "live_quote_available_or_direct_market_execution"
    );
}

#[test]
fn builtin_execution_module_submits_plan_through_fill_engine() {
    let mut module = BuiltinExecutionModule::default();
    let plan = ExecutionPlan {
        plan_id: "plan_1".into(),
        source_risk_decision_id: "risk_1".into(),
        orders: vec![SimOrder {
            order_id: "ord_1".into(),
            exchange: Exchange::Binance,
            symbol: Symbol::BtcUsdt,
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: 1.0,
            limit_price: None,
            time_in_force: TimeInForce::Gtc,
            allow_partial: false,
            reference_price: 50_000.0,
            slippage_bps: 5.0,
            fee_bps: 10.0,
            strategy_tag: "test".into(),
        }],
        created_at_ms: 10,
        trace_id: "trace".into(),
    };
    let mut portfolio = PortfolioState::new(100_000.0, 0);

    let result = module.submit_plan(
        &plan,
        &[sample_quote(50_000.0)],
        &mut portfolio,
        10,
        "trace",
    );

    assert!(matches!(result.status, ExecutionStatus::Filled));
    assert!(result.open_orders.is_empty());
}

#[test]
fn builtin_execution_module_uses_risk_decision_symbol_for_quotes_and_orders() {
    let module = BuiltinExecutionModule::default();
    let core_ir = sample_core_ir();
    let eth = Symbol::parse("ETHUSDT");
    let decision = RiskDecision {
        risk_decision_id: "risk_eth".into(),
        risk_id: "risk_global".into(),
        agent_decision_id: "decision_eth".into(),
        symbol: eth.clone(),
        status: DecisionStatus::Approve,
        mode: RiskDecisionMode::Normal,
        adjusted_portfolio_target_decision: None,
        adjusted_actions: vec![qrpc_core::ProposedAction {
            exchange: Exchange::Binance,
            side: OrderSide::Buy,
            quantity_ratio: 0.2,
            reference_price: 50_000.0,
            strategy_tag: "test".into(),
        }],
        reason_codes: vec![RiskReasonCode::WithinLimit],
        reason_text: "approved".into(),
        produced_at_ms: 10,
        trace_id: "trace".into(),
    };
    let eth_quote = NormalizedMarketData::Quote(QuoteSnapshot {
        data_id: "binance_eth_quote".into(),
        exchange: Exchange::Binance,
        symbol: eth.clone(),
        market_type: MarketType::Spot,
        best_bid: 3_995.0,
        best_ask: 4_005.0,
        bid_size: 10.0,
        ask_size: 10.0,
        mid_price: 4_000.0,
        ts_ms: 10,
        source_latency_ms: 0,
        source_status: SourceStatus::Healthy,
        data_quality: DataQualitySnapshot::default(),
    });

    let output = module.plan_execution(ExecutionPlanningRequest {
        risk_decisions: &[decision],
        core_ir: &core_ir,
        normalized_data: &[eth_quote],
        portfolio: &PortfolioState::new(100_000.0, 0),
        now_ms: 10,
        trace_id: "trace",
    });

    assert_eq!(output.plans.len(), 1);
    assert_eq!(output.plans[0].orders[0].symbol, eth);
    assert!((output.plans[0].orders[0].quantity - 5.0).abs() < 1e-9);
}

#[test]
fn builtin_execution_module_builds_basket_from_portfolio_target_diff() {
    let module = BuiltinExecutionModule::default();
    let core_ir = sample_core_ir();
    let btc = Symbol::BtcUsdt;
    let eth = Symbol::parse("ETHUSDT");
    let mut portfolio = PortfolioState::new(100_000.0, 0);
    portfolio.positions.push(qrpc_core::Position {
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

    let decision = RiskDecision {
        risk_decision_id: "risk_rebalance".into(),
        risk_id: "risk_global".into(),
        agent_decision_id: "decision_rebalance".into(),
        symbol: btc.clone(),
        status: DecisionStatus::Approve,
        mode: RiskDecisionMode::Normal,
        adjusted_portfolio_target_decision: Some(PortfolioTargetDecision {
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
            reason: "equal-weight rebalance".into(),
        }),
        adjusted_actions: Vec::new(),
        reason_codes: vec![RiskReasonCode::WithinLimit],
        reason_text: "approved".into(),
        produced_at_ms: 10,
        trace_id: "trace".into(),
    };
    let eth_quote = NormalizedMarketData::Quote(QuoteSnapshot {
        data_id: "binance_eth_quote".into(),
        exchange: Exchange::Binance,
        symbol: eth.clone(),
        market_type: MarketType::Spot,
        best_bid: 3_995.0,
        best_ask: 4_005.0,
        bid_size: 10.0,
        ask_size: 10.0,
        mid_price: 4_000.0,
        ts_ms: 10,
        source_latency_ms: 0,
        source_status: SourceStatus::Healthy,
        data_quality: DataQualitySnapshot::default(),
    });

    let output = module.plan_execution(ExecutionPlanningRequest {
        risk_decisions: &[decision],
        core_ir: &core_ir,
        normalized_data: &[sample_quote(50_000.0), eth_quote],
        portfolio: &portfolio,
        now_ms: 10,
        trace_id: "trace",
    });

    assert_eq!(output.plans.len(), 1);
    assert_eq!(output.plans[0].orders.len(), 2);
    let btc_order = output.plans[0]
        .orders
        .iter()
        .find(|order| order.symbol == btc)
        .expect("btc basket order");
    let eth_order = output.plans[0]
        .orders
        .iter()
        .find(|order| order.symbol == eth)
        .expect("eth basket order");
    assert_eq!(btc_order.side, OrderSide::Sell);
    assert_eq!(eth_order.side, OrderSide::Buy);
    assert!((btc_order.quantity - 0.4).abs() < 1e-9);
    assert!((eth_order.quantity - 12.5).abs() < 1e-9);
    assert_eq!(
        output.events[0].payload["sizing_source"],
        "portfolio_target_diff"
    );
}
