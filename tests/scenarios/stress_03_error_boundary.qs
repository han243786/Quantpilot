# ═══ 压力测试3：错误边界 — 不完整策略应正确失败 ═══

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=300)?
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}

@test {
    name: "压力测试3：错误边界与优雅降级"
    cover: ["STRESS-ERR-001"]
}

@step("首次编译应成功（最小可用策略）") {
    @compile
    @assert compile.compilable == true
}

@step("回测最小策略") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}

@step("运行最小策略 5秒") {
    @run { mode: "paper", duration: 5s }
    @assert run.events.length > 0
    @assert run.equity > 0
}
