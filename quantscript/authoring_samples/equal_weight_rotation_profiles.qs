fn strategy() {
    # risk
    risk.profile(
        "global",
        max_position=0.30,
        max_total_leverage=2.0,
        max_exchange_leverage=2.0,
        min_action_interval_ms=250
    )

    # execution
    execution.profile("paper", fee_bps=10.0, slippage_bps=5.0)

    # data
    let base = symbols(["BTCUSDT", "ETHUSDT", "SOLUSDT"])

    # intent
    for s in base {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        let score = momentum(closes, 20)
        if score > 0.03 {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }

    # agent
    rebalance(equal_weight(base), every="1d")
}
