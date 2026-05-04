fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=300)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "场景一：BTC双均线Paper运行"
    cover: ["P-03", "STRAT-001", "COMP-001", "RUN-001"]
}

@step("编译策略") {
    @compile
    @assert compile.compilable == true
}

@step("Paper运行10秒") {
    @run { mode: "paper", duration: 10s }
    @assert run.events.length > 0
    @assert run.equity > 0
}

@step("回测") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}
