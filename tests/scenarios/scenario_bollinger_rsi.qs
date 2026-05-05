# ═══ 策略：布林带 + RSI 组合 ═══
# 布林带下轨 + RSI 超卖 → 买入；布林带上轨 + RSI 超买 → 卖出
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let rsi_val = rsi(closes, 14)
    let bb = bollinger(closes, 20, 2.0)
    if closes[0] < bb.lower and rsi_val < 30 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    if closes[0] > bb.upper and rsi_val > 70 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "场景：布林带+RSI 组合"
    cover: []
}

@step("编译") {
    @compile
    @assert compile.compilable == true
}

@step("回测") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 50
}
