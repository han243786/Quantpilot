fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=100)?
    let ma5 = sma(closes, 5)
    let ma10 = sma(closes, 10)
    if ma5 > ma10 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    if ma5 < ma10 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test { name: "Screen策略A:激进短线5/10" cover: ["SCR-A"] }
@step("编译") { @compile @assert compile.compilable == true }
@step("回测") { @backtest { source: "deterministic_mock" } @assert backtest.metrics.step_count >= 100 }
