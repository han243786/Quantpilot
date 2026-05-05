# ═══ 策略：ATR动态仓位 + Stochastic ═══
# ATR 计算动态仓位大小，Stochastic 超买超卖触发信号
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let atr_val = atr(closes, 14)
    let dynamic_qty = 0.01 / (atr_val / closes[0])
    let stoch_k = stoch(closes, 14, 3)
    if stoch_k.k_pct < 20 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=dynamic_qty)
    }
    if stoch_k.k_pct > 80 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=dynamic_qty)
    }
}

@test {
    name: "场景：ATR动态仓位+Stochastic"
    cover: []
}

@step("编译") {
    @compile
    @assert compile.compilable == true
}

@step("回测") {
    @backtest { source: "deterministic_mock" }
}
