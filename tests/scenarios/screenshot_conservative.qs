fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let ma20 = sma(closes, 20)
    let ma50 = sma(closes, 50)
    if ma20 > ma50 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    if ma20 < ma50 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test { name: "Screen策略B:保守长线20/50" cover: ["SCR-B"] }
@step("编译") { @compile @assert compile.compilable == true }
@step("回测") { @backtest { source: "deterministic_mock" } @assert backtest.metrics.step_count >= 100 }
