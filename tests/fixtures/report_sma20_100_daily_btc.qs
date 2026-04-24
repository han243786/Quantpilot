fn strategy() {
    let data_feed = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=200)?
    let fast = sma(data_feed, 20)
    let slow = sma(data_feed, 100)

    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }

    if (fast - slow) / slow > 0.04 {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
