use super::super::lower_script_to_runtime_config;
use crate::parse_quant_script_module;
use qrpc_compiler::compile_runtime_protocol_config;
use qrpc_core::{RebalanceSchedule, Symbol};

#[test]
fn lowers_equal_weight_rebalance_helper_into_portfolio_rebalance_agent() {
    let module = parse_quant_script_module(
        r#"
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
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    assert_eq!(
        config.agents[0].params.get("portfolio_rebalance"),
        Some(&1.0)
    );
    assert_eq!(
        config.agents[0].params.get("max_quantity_ratio"),
        Some(&1.0)
    );
    assert_eq!(
        config.agents[0]
            .params
            .get("portfolio_rebalance_symbol_count"),
        Some(&2.0)
    );
    assert_eq!(
        config.agents[0].rebalance_symbols,
        vec![Symbol::BtcUsdt, Symbol::parse("ETHUSDT")]
    );
    assert_eq!(
        config.agents[0].rebalance_schedule,
        Some(RebalanceSchedule::Every1d)
    );
    assert_eq!(
        config.agents[0].rebalance_allocation_kind.as_deref(),
        Some("equal_weight")
    );
    assert_eq!(config.risks[0].max_position_ratio, 1.0);
    let compiled = compile_runtime_protocol_config(&config).unwrap();
    assert_eq!(
        compiled.core_ir.agent_policies[0].kind,
        qrpc_core_ir::AgentPolicyKind::PortfolioRebalance
    );
}

#[test]
fn lowers_fixed_weights_rebalance_helper_into_portfolio_rebalance_agent() {
    let module = parse_quant_script_module(
        r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(fixed_weights(base, weights=[0.7, 0.3]), every="slow")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        if sma(closes, 20) > sma(closes, 100) {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    assert_eq!(
        config.agents[0].rebalance_allocation_kind.as_deref(),
        Some("fixed_weights")
    );
    assert_eq!(config.agents[0].rebalance_target_weights, vec![0.7, 0.3]);
    assert_eq!(
        config.agents[0].rebalance_schedule,
        Some(RebalanceSchedule::EverySlow)
    );
}

#[test]
fn lowers_rank_weight_rebalance_helper_into_portfolio_rebalance_agent() {
    let module = parse_quant_script_module(
        r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT", "SOLUSDT"])
    rebalance(rank_weight(base, method="inverse_rank"), every="1d")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        if sma(closes, 20) > sma(closes, 100) {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    assert_eq!(
        config.agents[0].rebalance_allocation_kind.as_deref(),
        Some("rank_weight")
    );
    assert_eq!(
        config.agents[0].rebalance_rank_method.as_deref(),
        Some("inverse_rank")
    );
}

#[test]
fn lowers_score_weight_rebalance_helper_into_portfolio_rebalance_agent() {
    let module = parse_quant_script_module(
        r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT", "SOLUSDT"])
    rebalance(score_weight(base, normalize="sum"), every="1d")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        if sma(closes, 20) > sma(closes, 100) {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    assert_eq!(
        config.agents[0].rebalance_allocation_kind.as_deref(),
        Some("score_weight")
    );
    assert_eq!(
        config.agents[0].rebalance_score_normalize.as_deref(),
        Some("sum")
    );
}

#[test]
fn rejects_rebalance_helper_with_unsupported_frequency() {
    let module = parse_quant_script_module(
        r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="1h")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)
        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
    )
    .unwrap();

    let err = lower_script_to_runtime_config(&module).unwrap_err();
    assert!(err
        .to_string()
        .contains("rebalance(..., every=...) 当前仅支持"));
}

#[test]
fn rejects_rank_weight_with_unsupported_method() {
    let module = parse_quant_script_module(
        r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(rank_weight(base, method="weird"), every="1d")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        if sma(closes, 20) > sma(closes, 100) {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
    )
    .unwrap();

    let err = lower_script_to_runtime_config(&module).unwrap_err();
    assert!(err
        .to_string()
        .contains("rank_weight(..., method=...) 当前仅支持"));
}

#[test]
fn lowers_weekly_rebalance_schedule_into_agent_schedule() {
    let module = parse_quant_script_module(
        r#"
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="weekly")
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        if sma(closes, 20) > sma(closes, 100) {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    assert_eq!(
        config.agents[0].rebalance_schedule,
        Some(RebalanceSchedule::Weekly)
    );
}
