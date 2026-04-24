fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = closes[20..].sum() / 20
    let slow = closes[50..].sum() / 50

    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
