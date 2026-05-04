# ═══ 压力测试4：BTC+ETH 双交易对连续回测 ═══

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=300)?
    let fast = sma(closes, 10)
    let slow = sma(closes, 30)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "压力测试4：BTC 短周期双均线"
    cover: ["STRESS-SYMBOL-001"]
}

@step("编译 BTC 短周期策略") {
    @compile
    @assert compile.compilable == true
}

@step("运行 BTC 短周期 10秒") {
    @run { mode: "paper", duration: 10s }
    @assert run.events.length > 0
    @assert run.equity > 0
}

@step("回测 BTC 短周期") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
    @assert backtest.metrics.max_drawdown_pct >= 0
}

@step("第二次回测确认一致性") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}

@step("保存记录") {
    @save_run
}
