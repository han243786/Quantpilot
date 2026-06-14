use super::super::lower_script_to_runtime_config;
use crate::parse_quant_script_module;
use qrpc_core::IntentKind;

#[test]
fn lowers_manual_moving_average_helper_formula() {
    let module = parse_quant_script_module(
        r#"
fn moving_average(series, period) {
    let n = period
    return series[n..].sum() / n
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = moving_average(closes, 20)
    let slow = moving_average(closes, 60)
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
fn lowers_manual_momentum_formula() {
    let module = parse_quant_script_module(
        r#"
fn momentum_score(series, lookback) {
    return series[0] - series[lookback]
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = momentum_score(closes, 14)
    if score > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    assert!(config
        .intents
        .iter()
        .any(|intent| intent.kind == IntentKind::Momentum));
}

#[test]
fn lowers_manual_zscore_formula() {
    let module = parse_quant_script_module(
        r#"
fn zscore_signal(series, window) {
    let scope = series[window..]
    return (series[0] - mean(scope)) / stddev(scope)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = zscore_signal(closes, 20)
    if score > 2 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    } else if score < -2 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    assert!(config
        .intents
        .iter()
        .any(|intent| intent.kind == IntentKind::ZScore));
}

#[test]
fn lowers_manual_macd_histogram_formula() {
    let module = parse_quant_script_module(
        r#"
fn macd_hist(series, fast, slow, signal) {
    let macd_line = ema(series, fast) - ema(series, slow)
    let signal_line = ema(macd_line, signal)
    return macd_line - signal_line
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let hist = macd_hist(closes, 12, 26, 9)
    if hist > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if hist < 0 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    assert!(config
        .intents
        .iter()
        .any(|intent| intent.kind == IntentKind::Macd));
}

#[test]
fn lowers_manual_momentum_ratio_formula() {
    let module = parse_quant_script_module(
        r#"
fn momentum_ratio(series, lookback) {
    return (series[0] / series[lookback]) - 1
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = momentum_ratio(closes, 20)
    if score > 0.03 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if score < -0.03 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    assert!(config
        .intents
        .iter()
        .any(|intent| intent.kind == IntentKind::Momentum));
    let momentum = config
        .intents
        .iter()
        .find(|intent| intent.kind == IntentKind::Momentum)
        .unwrap();
    assert_eq!(momentum.params.get("lookback"), Some(&20.0));
    assert_eq!(momentum.params.get("threshold_ratio"), Some(&0.03));
}

#[test]
fn lowers_manual_ma_gap_ratio_formula() {
    let module = parse_quant_script_module(
        r#"
fn ma_gap(series, fast, slow) {
    return (sma(series, fast) - sma(series, slow)) / sma(series, slow)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let gap = ma_gap(closes, 20, 60)
    if gap > 0.02 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    let entry = config
        .intents
        .iter()
        .find(|intent| {
            intent.kind == IntentKind::LongTermBuy || intent.kind == IntentKind::SmaCrossover
        })
        .unwrap();
    assert_eq!(entry.params.get("fast_period"), Some(&20.0));
    assert_eq!(entry.params.get("slow_period"), Some(&60.0));
    assert_eq!(entry.params.get("entry_ratio"), Some(&1.02));
}

#[test]
fn lowers_manual_rsi_formula_from_rs_ratio() {
    let module = parse_quant_script_module(
        r#"
fn manual_rsi(series, period) {
    let rs = rma(gains(series), period) / rma(losses(series), period)
    return 100 - (100 / (1 + rs))
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = manual_rsi(closes, 14)
    if score < 30 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if score > 70 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    let rsi = config
        .intents
        .iter()
        .find(|intent| intent.kind == IntentKind::Rsi)
        .unwrap();
    assert_eq!(rsi.params.get("period"), Some(&14.0));
    assert_eq!(rsi.params.get("oversold_threshold"), Some(&30.0));
    assert_eq!(rsi.params.get("overbought_threshold"), Some(&70.0));
}

#[test]
fn lowers_manual_rsi_formula_with_avg_gain_loss_aliases() {
    let module = parse_quant_script_module(
        r#"
fn manual_rsi(series, period) {
    let avg_gain = wilders(gains(series), period)
    let avg_loss = wilders(losses(series), period)
    let rs = avg_gain / avg_loss
    return 100 - 100 / (1 + rs)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = manual_rsi(closes, 21)
    if score < 35 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    let rsi = config
        .intents
        .iter()
        .find(|intent| intent.kind == IntentKind::Rsi)
        .unwrap();
    assert_eq!(rsi.params.get("period"), Some(&21.0));
    assert_eq!(rsi.params.get("oversold_threshold"), Some(&35.0));
}

#[test]
fn lowers_manual_ema_rsi_formula() {
    let module = parse_quant_script_module(
        r#"
fn ema_rsi(series, period) {
    let avg_gain = ema(gains(series), period)
    let avg_loss = ema(losses(series), period)
    let rs = avg_gain / avg_loss
    return 100 - 100 / (1 + rs)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = ema_rsi(closes, 10)
    if score > 65 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    let rsi = config
        .intents
        .iter()
        .find(|intent| intent.kind == IntentKind::Rsi)
        .unwrap();
    assert_eq!(rsi.params.get("period"), Some(&10.0));
    assert_eq!(rsi.params.get("smoothing_method"), Some(&1.0));
    assert_eq!(rsi.params.get("overbought_threshold"), Some(&65.0));
}

#[test]
fn lowers_manual_cutler_rsi_formula() {
    let module = parse_quant_script_module(
        r#"
fn cutler_rsi(series, period) {
    let avg_gain = sma(gains(series), period)
    let avg_loss = sma(losses(series), period)
    let rs = avg_gain / avg_loss
    return 100 - 100 / (1 + rs)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = cutler_rsi(closes, 12)
    if score < 28 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let config = lower_script_to_runtime_config(&module).unwrap();
    let rsi = config
        .intents
        .iter()
        .find(|intent| intent.kind == IntentKind::Rsi)
        .unwrap();
    assert_eq!(rsi.params.get("period"), Some(&12.0));
    assert_eq!(rsi.params.get("smoothing_method"), Some(&2.0));
    assert_eq!(rsi.params.get("oversold_threshold"), Some(&28.0));
}

#[test]
fn rejects_manual_rsi_formula_from_loop_built_gain_loss_lists_in_formal_path() {
    let module = parse_quant_script_module(
        r#"
fn loop_gains(series) {
    let mut out = []
    for i in 1..series.len() {
        let diff = series[i] - series[i - 1]
        if diff > 0 {
            out.push(diff)
        } else {
            out.push(0)
        }
    }
    return out
}

fn loop_losses(series) {
    let mut out = []
    for i in 1..series.len() {
        let diff = series[i] - series[i - 1]
        if diff < 0 {
            out.push(-diff)
        } else {
            out.push(0)
        }
    }
    return out
}

fn manual_rsi(series, period) {
    let avg_gain = ema(loop_gains(series), period)
    let avg_loss = ema(loop_losses(series), period)
    let rs = avg_gain / avg_loss
    return 100 - 100 / (1 + rs)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = manual_rsi(closes, 14)
    if score > 70 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let err = lower_script_to_runtime_config(&module).unwrap_err();
    let message = err.to_string();
    assert!(
        message.contains("无法以符号方式展开 for 循环的可迭代对象") || message.contains("for 循环")
    );
}

#[test]
fn rejects_manual_rsi_formula_from_while_loop_gain_loss_lists_in_formal_path() {
    let module = parse_quant_script_module(
        r#"
fn while_gains(series) {
    let mut out = []
    let mut i = 1
    while i < series.len() {
        let diff = series[i] - series[i - 1]
        if diff > 0 {
            out.push(diff)
        } else {
            out.push(0)
        }
        let i = i + 1
    }
    return out
}

fn while_losses(series) {
    let mut out = []
    let mut i = 1
    while i < series.len() {
        let diff = series[i] - series[i - 1]
        if diff < 0 {
            out.push(abs(diff))
        } else {
            out.push(0)
        }
        let i = i + 1
    }
    return out
}

fn manual_rsi(series, period) {
    let avg_gain = sma(while_gains(series), period)
    let avg_loss = sma(while_losses(series), period)
    let rs = avg_gain / avg_loss
    return 100 - 100 / (1 + rs)
}

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = manual_rsi(closes, 9)
    if score < 25 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
    )
    .unwrap();

    let err = lower_script_to_runtime_config(&module).unwrap_err();
    assert!(err.to_string().contains("while 循环"));
}
