# ═══ 策略：ADX趋势过滤 + Donchian突破 ═══
# ADX > 25 表示强趋势，Donchian通道突破确认入场方向
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let adx_val = adx(closes, 14)
    let donch = donchian(closes, 20)
    if adx_val.adx > 25 and closes[0] >= donch.upper {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    if adx_val.adx > 25 and closes[0] <= donch.lower {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "场景：ADX趋势过滤+Donchian突破"
    cover: []
}

@step("编译") {
    @compile
    @assert compile.compilable == true
}

@step("回测") {
    @backtest { source: "deterministic_mock" }
}
