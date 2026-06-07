use super::*;
use qrpc_core::ExecutionPlan;

fn sample_order_with_side(
    side: OrderSide,
    order_type: OrderType,
    limit_price: Option<f64>,
) -> SimOrder {
    SimOrder {
        order_id: "ord_1".into(),
        exchange: Exchange::Binance,
        symbol: Symbol::BtcUsdt,
        side,
        order_type,
        quantity: 1.0,
        limit_price,
        time_in_force: TimeInForce::Gtc,
        allow_partial: false,
        reference_price: 50_000.0,
        slippage_bps: 5.0,
        fee_bps: 10.0,
        strategy_tag: "test".into(),
    }
}

fn sample_order(order_type: OrderType, limit_price: Option<f64>) -> SimOrder {
    sample_order_with_side(OrderSide::Buy, order_type, limit_price)
}

fn sample_plan(order: SimOrder) -> ExecutionPlan {
    ExecutionPlan {
        plan_id: "plan_1".into(),
        source_risk_decision_id: "risk_1".into(),
        orders: vec![order],
        created_at_ms: 1,
        trace_id: "trace_1".into(),
    }
}

fn sample_portfolio() -> PortfolioState {
    PortfolioState::new(100_000.0, 0)
}

fn sample_portfolio_with_long_position(qty: f64, avg_entry_price: f64) -> PortfolioState {
    let mut portfolio = PortfolioState::new(100_000.0, 0);
    portfolio.positions.push(Position {
        exchange: Exchange::Binance,
        symbol: Symbol::BtcUsdt,
        net_qty: qty,
        frozen_qty: 0.0,
        avg_entry_price,
        mark_price: avg_entry_price,
        unrealized_pnl: 0.0,
        realized_pnl: 0.0,
    });
    portfolio
}

fn sample_market(price: f64) -> BTreeMap<(Exchange, Symbol), MarketState> {
    BTreeMap::from([(
        (Exchange::Binance, Symbol::BtcUsdt),
        MarketState {
            price,
            bid_price: None,
            ask_price: None,
            buy_liquidity: 10.0,
            sell_liquidity: 10.0,
        },
    )])
}

#[test]
fn market_order_fills_immediately() {
    let mut engine = FillEngine::default();
    let plan = sample_plan(sample_order(OrderType::Market, None));
    let mut portfolio = sample_portfolio();

    let result = engine.submit_plan(
        &plan,
        &sample_market(50_000.0),
        &mut portfolio,
        10,
        "trace_1",
    );

    assert!(matches!(result.status, ExecutionStatus::Filled));
    assert_eq!(result.fills.len(), 1);
    assert!(portfolio.cash_balance < 100_000.0);
}

#[test]
fn order_level_slippage_changes_market_fill_price() {
    let mut low_slippage = FillEngine::default();
    let mut high_slippage = FillEngine::default();
    let mut base_order = sample_order(OrderType::Market, None);
    base_order.fee_bps = 0.0;

    let mut low_order = base_order.clone();
    low_order.slippage_bps = 0.0;
    let mut high_order = base_order;
    high_order.slippage_bps = 100.0;

    let low = low_slippage.submit_plan(
        &sample_plan(low_order),
        &sample_market(50_000.0),
        &mut sample_portfolio(),
        10,
        "trace_low",
    );
    let high = high_slippage.submit_plan(
        &sample_plan(high_order),
        &sample_market(50_000.0),
        &mut sample_portfolio(),
        10,
        "trace_high",
    );

    assert!(high.fills[0].filled_price > low.fills[0].filled_price);
}

#[test]
fn gtc_buy_order_is_capped_by_available_cash() {
    let mut engine = FillEngine::default();
    let mut order = sample_order(OrderType::Market, None);
    order.quantity = 3.0;
    order.allow_partial = true;
    order.fee_bps = 10.0;
    order.slippage_bps = 10.0;
    let plan = sample_plan(order);
    let mut portfolio = PortfolioState::new(50_000.0, 0);

    let result = engine.submit_plan(
        &plan,
        &sample_market(50_000.0),
        &mut portfolio,
        10,
        "trace_1",
    );

    assert_eq!(result.fills.len(), 1);
    assert!(result.fills[0].filled_qty < 1.0);
    assert!(portfolio.cash_balance >= -1e-6);
    assert!(portfolio.available_cash_balance <= 1e-6);
}

#[test]
fn resting_limit_order_stays_open_when_not_marketable() {
    let mut engine = FillEngine::default();
    let plan = sample_plan(sample_order(OrderType::Limit, Some(49_000.0)));
    let mut portfolio = sample_portfolio();

    let result = engine.submit_plan(
        &plan,
        &sample_market(50_000.0),
        &mut portfolio,
        10,
        "trace_1",
    );

    assert!(matches!(result.status, ExecutionStatus::Open));
    assert_eq!(result.open_orders.len(), 1);
    assert_eq!(engine.open_order_count(), 1);
    assert_eq!(portfolio.positions.len(), 0);
    assert!(portfolio.frozen_cash_balance > 0.0);
}

#[test]
fn repeated_plan_id_is_idempotent() {
    let mut engine = FillEngine::default();
    let plan = sample_plan(sample_order(OrderType::Market, None));
    let mut portfolio = sample_portfolio();

    let first = engine.submit_plan(
        &plan,
        &sample_market(50_000.0),
        &mut portfolio,
        10,
        "trace_1",
    );
    let cash_after_first = portfolio.cash_balance;
    let second = engine.submit_plan(
        &plan,
        &sample_market(50_000.0),
        &mut portfolio,
        11,
        "trace_1",
    );

    assert_eq!(first.fills.len(), second.fills.len());
    assert_eq!(cash_after_first, portfolio.cash_balance);
}

#[test]
fn open_order_fills_on_later_market_update() {
    let mut engine = FillEngine::default();
    let plan = sample_plan(sample_order(OrderType::Limit, Some(49_000.0)));
    let mut portfolio = sample_portfolio();

    let submit = engine.submit_plan(
        &plan,
        &sample_market(50_000.0),
        &mut portfolio,
        10,
        "trace_1",
    );
    assert!(matches!(submit.status, ExecutionStatus::Open));
    let update = engine.on_market_update(&sample_market(48_500.0), &mut portfolio, 20, "trace_2");

    assert_eq!(update.fills.len(), 1);
    assert_eq!(engine.open_order_count(), 0);
    assert_eq!(portfolio.positions.len(), 1);
    assert_eq!(portfolio.frozen_cash_balance, 0.0);
}

#[test]
fn ioc_partial_fill_does_not_rest() {
    let mut engine = FillEngine::default();
    let mut order = sample_order(OrderType::Market, None);
    order.quantity = 12.0;
    order.time_in_force = TimeInForce::Ioc;
    order.allow_partial = true;
    let plan = sample_plan(order);
    let mut portfolio = PortfolioState::new(1_000_000.0, 0);
    let result = engine.submit_plan(
        &plan,
        &sample_market(50_000.0),
        &mut portfolio,
        10,
        "trace_1",
    );

    assert!(matches!(result.status, ExecutionStatus::Filled));
    assert_eq!(result.fills[0].filled_qty, 10.0);
    assert_eq!(result.open_orders.len(), 0);
}

#[test]
fn fok_rejects_when_liquidity_is_insufficient() {
    let mut engine = FillEngine::default();
    let mut order = sample_order(OrderType::Market, None);
    order.quantity = 12.0;
    order.time_in_force = TimeInForce::Fok;
    let plan = sample_plan(order);
    let mut portfolio = sample_portfolio();
    let result = engine.submit_plan(
        &plan,
        &sample_market(50_000.0),
        &mut portfolio,
        10,
        "trace_1",
    );

    assert!(matches!(result.status, ExecutionStatus::Rejected));
    assert_eq!(result.fills.len(), 0);
    assert_eq!(result.events[0].payload["lifecycle_stage"], "accepted");
    assert_eq!(result.events[1].payload["lifecycle_stage"], "rejected");
    assert_eq!(
        result.events[1].payload["explanation_summary"],
        "Order rejected: FOK 流动性不足."
    );
}
#[test]
fn resting_sell_order_freezes_position_and_releases_on_fill() {
    let mut engine = FillEngine::default();
    let plan = sample_plan(sample_order_with_side(
        OrderSide::Sell,
        OrderType::Limit,
        Some(51_000.0),
    ));
    let mut portfolio = sample_portfolio_with_long_position(2.0, 48_000.0);

    let submit = engine.submit_plan(
        &plan,
        &sample_market(50_000.0),
        &mut portfolio,
        10,
        "trace_1",
    );

    assert!(matches!(submit.status, ExecutionStatus::Open));
    assert_eq!(submit.open_orders[0].reserved_qty, 1.0);
    assert_eq!(portfolio.positions[0].frozen_qty, 1.0);
    assert_eq!(submit.events[1].payload["lifecycle_stage"], "open");

    let update = engine.on_market_update(&sample_market(51_000.0), &mut portfolio, 20, "trace_2");
    assert_eq!(update.fills.len(), 1);
    assert_eq!(engine.open_order_count(), 0);
    assert!((portfolio.positions[0].net_qty - 1.0).abs() < 1e-9);
    assert_eq!(portfolio.positions[0].frozen_qty, 0.0);
    assert_eq!(update.events[0].payload["lifecycle_stage"], "completed");
    assert_eq!(update.events[1].payload["lifecycle_stage"], "completed");
}

#[test]
fn resting_sell_order_rejects_when_position_is_unavailable() {
    let mut engine = FillEngine::default();
    let mut first_order = sample_order_with_side(OrderSide::Sell, OrderType::Limit, Some(51_000.0));
    first_order.order_id = "ord_1".into();
    let mut second_order =
        sample_order_with_side(OrderSide::Sell, OrderType::Limit, Some(52_000.0));
    second_order.order_id = "ord_2".into();
    let mut portfolio = sample_portfolio_with_long_position(1.0, 48_000.0);

    let first = engine.submit_plan(
        &sample_plan(first_order),
        &sample_market(50_000.0),
        &mut portfolio,
        10,
        "trace_1",
    );
    assert!(matches!(first.status, ExecutionStatus::Open));

    let second_plan = ExecutionPlan {
        plan_id: "plan_2".into(),
        source_risk_decision_id: "risk_1".into(),
        orders: vec![second_order],
        created_at_ms: 2,
        trace_id: "trace_2".into(),
    };
    let second = engine.submit_plan(
        &second_plan,
        &sample_market(50_000.0),
        &mut portfolio,
        11,
        "trace_2",
    );

    assert!(matches!(second.status, ExecutionStatus::Rejected));
    assert_eq!(engine.open_order_count(), 1);
    assert_eq!(portfolio.positions[0].frozen_qty, 1.0);
}

#[test]
fn market_sell_order_rejects_when_position_is_unavailable() {
    let mut engine = FillEngine::default();
    let plan = sample_plan(sample_order_with_side(
        OrderSide::Sell,
        OrderType::Market,
        None,
    ));
    let mut portfolio = sample_portfolio();

    let result = engine.submit_plan(
        &plan,
        &sample_market(50_000.0),
        &mut portfolio,
        10,
        "trace_1",
    );

    assert!(matches!(result.status, ExecutionStatus::Rejected));
    assert!(result.fills.is_empty());
    assert!(portfolio.positions.is_empty());
}
