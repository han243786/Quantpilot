# ═══ 压力测试2：快速多轮编译→运行→回测循环 ═══

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 15)
    let slow = sma(closes, 60)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "压力测试2：快速多轮编译→运行→回测"
    cover: ["STRESS-CYCLE-001"]
}

@step("第1轮：编译") {
    @compile
    @assert compile.compilable == true
}

@step("第1轮：运行 5秒") {
    @run { mode: "paper", duration: 5s }
    @assert run.events.length > 0
    @assert run.equity > 0
}

@step("第1轮：回测") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}

@step("第2轮：重新编译") {
    @compile
    @assert compile.compilable == true
}

@step("第2轮：运行 10秒") {
    @run { mode: "paper", duration: 10s }
    @assert run.events.length > 0
    @assert run.equity > 0
}

@step("第2轮：回测") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}

@step("第3轮：编译") {
    @compile
    @assert compile.compilable == true
}

@step("第3轮：运行 5秒") {
    @run { mode: "paper", duration: 5s }
    @assert run.events.length > 0
}

@step("第3轮：回测") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}

@step("最终保存") {
    @save_run
}
