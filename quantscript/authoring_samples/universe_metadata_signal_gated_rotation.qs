fn strategy() {
    # risk
    risk.profile(
        "global",
        max_position=0.15,
        max_total_leverage=1.5,
        max_exchange_leverage=1.5,
        min_action_interval_ms=250
    )

    # execution
    execution.profile("paper", fee_bps=10.0, slippage_bps=5.0)

    # data
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let liquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=180)
    let leaders = top(sort_by(liquid, key="market_cap", order="desc"), 5)

    # intent
    for s in leaders {
        let closes = fetch(s, exchange="binance", interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 60)
        let rsi14 = rsi(closes, 14)
        if fast > slow {
            if rsi14 < 65 {
                emit Intent("BUY", instrument=s, quantity=1.0)
            } else {
                emit Intent("SELL", instrument=s, quantity=1.0)
            }
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }

    # agent
    rebalance(rank_weight(leaders, method="linear"), every="weekly")
}
