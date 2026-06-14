use super::*;
use crate::parse_quant_script_module;
use qrpc_compiler::compile_runtime_protocol_config;
use qrpc_core::{Exchange, IntentKind, RebalanceSchedule, Symbol};
use qrpc_runtime::RuntimeCoordinator;

mod manual_formula_tests;
mod spread_lowering_tests;

const MA_SCRIPT: &str = r#"
import math
from data import fetch as get_data
from signals@1.2 import sma

fn strategy() {
    let closes = get_data("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 60)

    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;

const RSI_SCRIPT: &str = r#"
from data import fetch as get_data
from signals import rsi

fn strategy() {
    let closes = get_data("BTCUSDT", interval="1d", lookback=200)?
    let r = rsi(closes, 14)
    if r < 30 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if r > 70 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#;

#[test]
fn lowers_ma_cross_into_runtime_config() {
    let module = parse_quant_script_module(MA_SCRIPT).unwrap();
    let config = lower_script_to_runtime_config(&module).unwrap();
    assert_eq!(config.data_sources.len(), 1);
    assert_eq!(config.intents.len(), 2);
    assert!(config.intents.iter().any(|intent| matches!(
        intent.kind,
        IntentKind::LongTermBuy | IntentKind::SmaCrossover
    )));
}

#[test]
fn lowers_ma_cross_with_aliased_data_binding_into_consistent_input_ids() {
    let module = parse_quant_script_module(
        r#"
fn strategy() {
    let data_btc_daily_series = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(data_btc_daily_series, 20)
    let slow = sma(data_btc_daily_series, 50)

    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    assert_eq!(config.data_sources[0].data_id, "data_btc_daily");
    assert!(config
        .intents
        .iter()
        .all(|intent| intent.input_data_ids == vec!["data_btc_daily".to_string()]));
}

#[test]
fn lowers_rsi_thresholds_into_single_runtime_intent() {
    let module = parse_quant_script_module(RSI_SCRIPT).unwrap();
    let config = lower_script_to_runtime_config(&module).unwrap();
    assert_eq!(config.intents.len(), 1);
    assert_eq!(config.intents[0].kind, IntentKind::Rsi);
    assert_eq!(config.intents[0].params.get("period"), Some(&14.0));
    assert_eq!(
        config.intents[0].params.get("oversold_threshold"),
        Some(&30.0)
    );
    assert_eq!(
        config.intents[0].params.get("overbought_threshold"),
        Some(&70.0)
    );
}

#[test]
fn lowered_script_runs_in_runtime() {
    let module = parse_quant_script_module(MA_SCRIPT).unwrap();
    let config = lower_script_to_runtime_config(&module).unwrap();
    let compiled = compile_runtime_protocol_config(&config).unwrap();
    let mut runtime = RuntimeCoordinator::new(compiled);
    let session = runtime
        .run_session(1_700_000_000_000, 1_700_000_005_000)
        .unwrap();
    assert!(!session.slow_cycle.intent_signals.is_empty());
}

#[test]
fn lowers_user_defined_helper_function_calls() {
    let module = parse_quant_script_module(
        r#"
fn fast_ma(series, period) {
    return sma(series, period)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = fast_ma(closes, 20)
    let slow = sma(closes, 60)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    assert!(config.intents.iter().any(|intent| matches!(
        intent.kind,
        IntentKind::LongTermBuy | IntentKind::SmaCrossover
    )));
}

#[test]
fn rejects_semantic_errors_before_runtime_lowering() {
    let module = parse_quant_script_module(
        r#"
fn strategy() {
    let signal = missing_helper(1)
    if 42 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    if signal > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let err = lower_script_to_runtime_config(&module).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("QS0005"));
    assert!(message.contains("missing_helper"));
    assert!(message.contains("QS0006"));
}

#[test]
fn lowers_fetch_exchange_argument_into_data_source() {
    let module = parse_quant_script_module(
        r#"
fn strategy() {
    let data_price_feed_series = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let intent_entry_signal = rsi(data_price_feed_series, 14)
    if intent_entry_signal < 30 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    assert_eq!(
        config
            .data_sources
            .iter()
            .map(|source| source.data_id.clone())
            .collect::<Vec<_>>(),
        vec!["data_price_feed".to_string()]
    );
    assert_eq!(config.data_sources[0].exchange, Exchange::Okx);
    assert_eq!(config.data_sources[0].data_id, "data_price_feed");
    assert_eq!(config.intents[0].intent_id, "intent_entry");
}

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
