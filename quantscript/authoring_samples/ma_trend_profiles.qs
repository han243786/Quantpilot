fn strategy() {
    # risk
    risk.profile(
        "global",
        max_position=0.25,
        max_total_leverage=2.0,
        max_exchange_leverage=2.0,
        min_action_interval_ms=100
    )

    # execution
    execution.profile("paper", fee_bps=10.0, slippage_bps=5.0)

    # data
    let closes = fetch("BTCUSDT", exchange="okx", interval="1d", lookback=220)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 100)

    # intent
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
