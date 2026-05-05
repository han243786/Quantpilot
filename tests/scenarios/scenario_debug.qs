fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "场景：@debug 调试输出"
    cover: ["R3-1"]
}

@step("编译") {
    @compile
    @assert compile.compilable == true
}

@debug(closes, fast, slow)

@step("回测") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 50
}
