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
    name: "M2 Compare Test"
    cover: []
}

@step("compile") {
    @compile
    @assert compile.compilable == true
    @assert compile.counts.data_sources == 1
}

@step("bt1") {
    @backtest { source: "deterministic_mock", seed: 7, volatility: 2.0 }
    @assert backtest.metrics.step_count >= 100
}

@step("bt2") {
    @backtest { source: "deterministic_mock", seed: 42 }
    @assert backtest.metrics.step_count >= 100
}

@step("compare") {
    @compare_backtests { left: 0, right: 1 }
}
