runtime {
  initial_cash_balance: 100000
  taker_fee_bps: 10
  default_slippage_bps: 5
  total_cost_buffer_bps: 20
}

data binance_btc_150d_1d {
  exchange: binance
  symbol: BTCUSDT
  market_type: spot
  kind: kline
  days: 150
  interval: "1d"
}

data binance_btc_quote {
  exchange: binance
  symbol: BTCUSDT
  market_type: spot
  kind: quote
}

intent intent_long_buy {
  name: "Long Buy"
  kind: long_term_buy
  inputs: [binance_btc_150d_1d]
}

intent intent_binance_quote {
  name: "Binance Quote"
  kind: quote_observe
  inputs: [binance_btc_quote]
}

agent agent_long_term {
  name: "Long Term Agent"
  intents: [intent_long_buy]
}

agent agent_quote_watch {
  name: "Quote Watch Agent"
  intents: [intent_binance_quote]
}

risk risk_global {
  name: "Global Risk"
  agents: [agent_long_term, agent_quote_watch]
  max_total_leverage: 3
  max_exchange_leverage: 3
  min_action_interval_ms: 100
}
