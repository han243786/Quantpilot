fn strategy() {
    # risk
    risk.profile(
        "global",
        max_position=0.15,
        max_total_leverage=1.2,
        max_exchange_leverage=1.2,
        min_action_interval_ms=250
    )

    # execution
    execution.profile("paper", fee_bps=12.5, slippage_bps=7.5)

    # data
    let left = fetch("BTCUSDT", exchange="binance", interval="1m", lookback=200)?
    let right = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
    let left_aligned = align_asof(field(left, name="bid"), direction="backward", tolerance_ms=5000)
    let right_aligned = align_asof(field(right, name="ask"), direction="backward", tolerance_ms=5000)
    let spread_signal = spread(left_aligned, right_aligned, output="bps")

    # intent
    if spread_signal > 6 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
