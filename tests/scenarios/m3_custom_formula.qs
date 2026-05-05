fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    # Custom formula: RSI-like — ratio of avg gains to avg losses
    let gains = closes[1..].sum() / closes.len()
    let losses = closes[0..].sum() / closes.len()
    if gains > losses {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "M3 Custom Formula Test"
    cover: []
}

@step("compile") {
    @compile
    @assert compile.compilable == true
}

@step("backtest") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}
