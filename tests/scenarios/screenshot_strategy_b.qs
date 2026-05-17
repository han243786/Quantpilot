fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let rsi_value = rsi(closes, 14)
    if rsi_value < 40 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    if rsi_value > 60 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "策略B: BTC RSI保守区间(14,40/60)"
    cover: ["V110-SCREENSHOT-B"]
}

@step("编译策略B") {
    @compile
    @assert compile.compilable == true
}

@step("回测策略B") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}
