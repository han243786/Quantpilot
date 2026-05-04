# ═══ 压力测试1：最复杂策略 + 全断言算子 + 多轮参数修改 ═══

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
    name: "压力测试1：全断言算子 + 参数修改链"
    cover: ["STRESS-MAX-001"]
}

@step("编译 — 验证 compilable + protocol_name") {
    @compile
    @assert compile.compilable == true
    @assert compile.protocol_name != null
    @assert compile.diagnostics.length == 0
}

@step("Paper运行 30秒") {
    @run { mode: "paper", duration: 30s }
    @assert run.events.length > 0
    @assert run.equity > 0
    @assert run.equity != null
    @assert run.status == "completed"
    @assert run.has_event("DataUpdated")
    @assert run.positions.length >= 0
}

@step("回测 — 验证 step_count + 数据结构") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
    @assert backtest.metrics.total_fills >= 0
    @assert backtest.metrics.max_drawdown_pct >= 0
    @assert backtest.metrics.step_count >= 200
}

@step("修改参数：快线30 慢线100") {
    @modify { node: "dual_ma", param: "fast_period", value: 30 }
}

@step("修改后重新编译") {
    @compile
    @assert compile.compilable == true
}

@step("修改后重新回测") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}

@step("第三次参数修改：快线10 慢线200") {
    @modify { node: "dual_ma", param: "fast_period", value: 10 }
}

@step("保存运行记录") {
    @save_run
}
