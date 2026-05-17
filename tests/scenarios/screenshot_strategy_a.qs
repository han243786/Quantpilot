fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let rsi_value = rsi(closes, 14)
    if rsi_value < 30 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    if rsi_value > 70 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "策略A: BTC RSI超卖买入(14)"
    cover: ["V110-SCREENSHOT-A"]
}

@step("编译策略A") {
    @compile
    @assert compile.compilable == true
}

@step("回测策略A") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}
