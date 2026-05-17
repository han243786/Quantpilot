fn strategy() {
    let btc_closes = fetch("BTCUSDT", interval="1d", lookback=300)?
    let eth_closes = fetch("ETHUSDT", interval="1d", lookback=300)?

    let btc_fast = sma(btc_closes, 20)
    let btc_slow = sma(btc_closes, 50)
    if btc_fast > btc_slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }

    let eth_fast = sma(eth_closes, 10)
    let eth_slow = sma(eth_closes, 30)
    if eth_fast > eth_slow {
        emit Intent("BUY", instrument="ETHUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="ETHUSDT", quantity=1.0)
    }
}

@test {
    name: "v1.1.0 多标的策略验证"
    cover: ["V110-MULTI-001"]
}

@step("编译多标的策略") {
    @compile
    @assert compile.compilable == true
}

@step("回测多标的") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
    @assert backtest.metrics.trade_count >= 0
}
