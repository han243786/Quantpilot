fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "M4 Debug Test"
    cover: []
}

@step("compile") {
    @compile
    @assert compile.compilable == true
}

@step("debug_backtest") {
    @debug(closes, fast, slow)
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}
