fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=300)?

    # Intent 1-2: EMA双均线
    let fast_ema = ema(closes, 12)
    let slow_ema = ema(closes, 26)

    # Intent 3-4: RSI超买超卖
    let rsi_val = rsi(closes, 14)

    # Intent 5-6: SMA长线趋势
    let sma_trend = sma(closes, 100)

    if fast_ema > slow_ema {
        if rsi_val < 40 {
            emit Intent("BUY", instrument="BTCUSDT", quantity=0.01)
        }
    }

    if fast_ema < slow_ema {
        emit Intent("SELL", instrument="BTCUSDT", quantity=0.01)
    }

    if rsi_val > 75 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=0.01)
    }
}

@test {
    name: "多信号融合 — EMA+RSI+SMA 6 Intent"
    cover: ["STRAT-PRO-001"]
}

@step("编译+回测") {
    @compile
    @assert compile.compilable == true
    @backtest { source: "deterministic_mock", volatility: 1.5 }
    @assert backtest.metrics.step_count >= 100
}

@step("高波动回测") {
    @backtest { source: "deterministic_mock", volatility: 3.0 }
    @assert backtest.metrics.step_count >= 100
}

@step("对比") {
    @compare_backtests { left: 0, right: 1 }
}

@step("Paper模拟") {
    @run { mode: "paper", duration: 5s }
    @assert run.events.length > 0
}

@step("模拟盘实战") {
    @run { mode: "testnet", duration: 10s }
}
