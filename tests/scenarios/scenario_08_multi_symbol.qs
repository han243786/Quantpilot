# ═══ 策略定义 ═══
# 双均线策略 — 用于多交易对/多交易所测试
fn strategy() {
    let closes = fetch("ETHUSDT", interval="1d", lookback=300)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)
    if fast > slow {
        emit Intent("BUY", instrument="ETHUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="ETHUSDT", quantity=1.0)
    }
}

# ═══ 测试指令 ═══
@test {
    name: "场景八：多交易对和多交易所"
    cover: ["RUN-011~015", "EDIT-031"]
}

@step("编译 ETHUSDT 策略") {
    @compile
    @assert compile.compilable == true
}

@step("Paper 运行 ETHUSDT") {
    @run { mode: "paper", duration: 10s }
    @assert run.events.length > 0
    @assert run.equity > 0
}

@step("回测 ETHUSDT") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}
