fn strategy() {
    # risk
    risk.profile(
        "global",
        max_position=0.20,
        max_total_leverage=1.5,
        max_exchange_leverage=1.5,
        min_action_interval_ms=250
    )

    # execution
    execution.profile("paper", fee_bps=10.0, slippage_bps=5.0)

    # data
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let liquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=100)
    let selected = top(sort_by(liquid, key="market_cap", order="desc"), 3)

    # intent
    for s in selected {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        let score = momentum(closes, 20)
        if score > 0.03 {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }

    # agent
    rebalance(rank_weight(selected, method="linear"), every="weekly")
}
