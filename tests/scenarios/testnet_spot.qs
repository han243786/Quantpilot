fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 5)
    let slow = sma(closes, 20)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=0.01)
    }
}

@test {
    name: "模拟盘现货策略-激进"
    cover: []
}

@step("testnet_run") {
    @run { mode: "testnet", duration: 10s }
}
