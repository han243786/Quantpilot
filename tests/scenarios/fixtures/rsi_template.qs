# ═══ 策略模板：RSI 策略 ═══
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let rsi_val = rsi(closes, 14)
    let oversold = rsi_val < 30
    let overbought = rsi_val > 70
    if oversold {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        if overbought {
            emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
        }
    }
}
