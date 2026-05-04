# ═══ 策略定义 ═══
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

# ═══ 测试指令 ═══
@test {
    name: "场景二：回测 + 参数对比"
    cover: ["BT-001~012", "COMP-002", "RUN-006~008"]
}

@step("编译策略") {
    @compile
    @assert compile.compilable == true
}

@step("第一次回测 — deterministic_mock") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}

@step("保存第一次回测记录") {
    @save_run
}

@step("第二次回测") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}
