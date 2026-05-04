# ═══ 策略模板：BTC 双均线（基础参数） ═══
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
