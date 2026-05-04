# ═══ 策略模板：ETH 双均线 ═══
fn strategy() {
    let closes = fetch("ETHUSDT", interval="1d", lookback=300)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)
    if fast > slow {
        emit Intent("BUY", instrument="ETHUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="ETHUSDT", quantity=1.0)
    }
}
