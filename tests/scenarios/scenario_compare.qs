fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 15)
    let slow = sma(closes, 40)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "场景：两次回测对比"
    cover: ["R3-2"]
}

@step("编译") {
    @compile
    @assert compile.compilable == true
}

@step("回测A") {
    @backtest { source: "deterministic_mock", seed: 7 }
}

@step("回测B") {
    @backtest { source: "deterministic_mock", seed: 42 }
}

@step("对比") {
    @compare_backtests { left: 0, right: 1 }
}
