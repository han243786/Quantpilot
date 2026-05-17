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
    name: "v1.1.0 回测指标验证"
    cover: ["V110-BACKTEST-001"]
}

@step("编译策略") {
    @compile
    @assert compile.compilable == true
}

@step("回测验证新指标") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
    @assert backtest.metrics.total_return_ratio > -1.0
    @assert backtest.metrics.risk_adjusted.sharpe_ratio > -10.0
    @assert backtest.metrics.trade_analysis.profit_factor >= 0.0
    @assert backtest.metrics.drawdown_analysis.max_drawdown_ratio >= 0.0
}
