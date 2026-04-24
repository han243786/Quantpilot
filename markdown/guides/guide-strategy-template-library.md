# Strategy Template Library

## Current boundary

The current template library is a frontend-owned starter surface.

It is intentionally narrow:

- it uses a canonical local template list
- it builds starter graphs only from currently supported builtin modules
- it loads the selected template into the current working draft
- it does not create a second backend template transport
- it does not auto-persist a graph version on load

## Current starter templates

### `dual_ma_trend`

Purpose:

- start from a simple BTC trend-following graph

Modules:

- `builtin.data.kline`
- `builtin.intent.double_ma`
- `builtin.intent.ma_deviation`
- `builtin.agent.weighted`
- `builtin.risk.global`
- `builtin.execution.paper`

Symbols:

- `BTCUSDT`

### `rsi_reversion`

Purpose:

- start from a lightweight ETH mean-reversion graph

Modules:

- `builtin.runtime.control`
- `builtin.data.kline`
- `builtin.intent.rsi`
- `builtin.agent.weighted`
- `builtin.risk.global`
- `builtin.execution.paper`

Symbols:

- `ETHUSDT`

### `multi_symbol_rebalance`

Purpose:

- start from the current beta multi-symbol rebalance surface

Modules:

- `builtin.data.kline`
- `builtin.intent.double_ma`
- `builtin.intent.ma_deviation`
- `builtin.agent.weighted`
- `builtin.risk.global`
- `builtin.execution.paper`

Symbols:

- `BTCUSDT`
- `ETHUSDT`
- `SOLUSDT`

## Loading rule

Loading a template should be understood literally:

- the selected template replaces the current in-memory working draft
- the loaded graph remains on the existing graph/runtime configuration surface
- persisted history, experiments, and backtest index data stay outside the draft reset
- operators should explicitly save if they want the loaded draft to become a persisted graph version

## What this is not

The current template library is not:

- a backend template registry
- a marketplace
- a second graph DTO family
- a second starter-graph protocol beside the existing graph/runtime surface

Future widening should only happen when the current local canonical list becomes insufficient for the supported product surface.
