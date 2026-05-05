fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = ema(closes, 10)
    let slow = ema(closes, 30)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "M1 EMA Test"
    cover: []
}

@step("comp+backtest") {
    @compile
    @assert compile.compilable == true
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}
