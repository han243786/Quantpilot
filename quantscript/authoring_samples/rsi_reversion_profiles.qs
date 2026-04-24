fn strategy() {
    # risk
    risk.profile(
        "global",
        max_position=0.20,
        max_total_leverage=1.5,
        max_exchange_leverage=1.5,
        min_action_interval_ms=100
    )

    # execution
    execution.profile("paper", fee_bps=8.0, slippage_bps=4.0)

    # data
    let closes = fetch("ETHUSDT", exchange="binance", interval="1d", lookback=200)?
    let score = rsi(closes, 14)

    # intent
    if score < 28 {
        emit Intent("BUY", instrument="ETHUSDT", quantity=1.0)
    }
}
