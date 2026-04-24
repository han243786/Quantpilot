# QuantScript Module Authoring Convention

This document defines the recommended authoring convention for QuantScript `V1`.

Its purpose is not to widen the language.
Its purpose is to force real strategy code into a stable, readable module order so that:

- developers can scan a strategy quickly
- frontend views can present the strategy in a predictable structure
- AI systems can generate and revise QuantScript without inventing new layout rules
- users can understand data flow and make small edits safely

The target module order is:

1. `data`
2. `intent`
3. `agent`
4. `risk`
5. `execution`

## 1. Important boundary

QuantScript `V1` does **not** yet have first-class syntax blocks named `data`, `intent`, `agent`, `risk`, and `execution`.

This is an authoring convention over the current formal QuantScript trunk.

Current mapping:

- `data` is written with `fetch(...)`, `get_data(...)`, and retained universe helpers
- `intent` is written with indicator calculations and `emit Intent(...)`
- `agent` is mostly implicit today and is inferred by lowering from the strategy body and rebalance helpers
- `risk` is written with `risk.profile("global", ...)`
- `execution` is written with `execution.profile("paper", ...)`

So this document standardizes **ordering, naming, and separation of concerns**, not new grammar.

## 2. Canonical module order inside `fn strategy()`

Write the body of `fn strategy()` in this order:

1. profile declarations
2. data declarations
3. intent signals and conditions
4. agent shaping logic
5. final `emit Intent(...)`

Because `risk.profile(...)` and `execution.profile(...)` are currently required to be top-level statements, the practical order for `V1` should be:

1. `risk`
2. `execution`
3. `data`
4. `intent`
5. `agent`

This ordering keeps the runtime-required top-level profile placement while preserving the conceptual pipeline.

## 3. Section headers

Always divide a real strategy with explicit comment headers.

Use this exact style:

```qs
fn strategy() {
    # risk
    risk.profile("global", max_position=0.25)

    # execution
    execution.profile("paper", fee_bps=10.0, slippage_bps=5.0)

    # data
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?

    # intent
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)

    # agent
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
```

Frontend and AI tooling should treat these headers as display hints, not as executable syntax.

## 4. Module meanings

### 4.1 Data module

The `data` section declares all raw inputs used by the strategy.

Allowed content:

- `fetch(...)`
- `get_data(...)`
- retained `universe(...)` helper family
- retained data transforms that are still part of the current executable trunk

Rules:

- declare data before indicator logic
- avoid mixing data-source acquisition with final action emission
- each fetched or derived series should have a clear name

Recommended naming:

- `closes`
- `btc_daily`
- `eth_quote`
- `base_universe`
- `ranked_universe`

### 4.2 Intent module

The `intent` section turns data into actionable conditions.

Allowed content:

- retained indicator helper calls
- retained spread slice
- thresholds and comparisons that map to current lowering

Rules:

- do not emit actions yet
- compute scores, thresholds, and comparisons here
- one variable should represent one decision concept

Recommended naming:

- `fast_ma`
- `slow_ma`
- `rsi_score`
- `momentum_score`
- `zscore_signal`
- `spread_signal`

### 4.3 Agent module

The `agent` section defines how the strategy acts on intent.

In `V1`, the agent layer is still mostly implicit.
It is inferred by lowering from:

- `emit Intent(...)`
- retained `rebalance(...)` helpers

Rules:

- final action conditions belong here
- keep branch logic shallow and explicit
- if you use portfolio rebalance, keep all allocation helpers together here

Recommended patterns:

- direct conditional `emit Intent(...)`
- one retained `rebalance(...)`

Avoid:

- scattering `emit Intent(...)` across unrelated helper branches
- mixing rebalance declarations into the data section

### 4.4 Risk module

The `risk` section declares the one supported retained risk profile.

Current `V1` pattern:

```qs
risk.profile("global", max_position=0.25, max_total_leverage=2.0, max_exchange_leverage=2.0, min_action_interval_ms=100)
```

Rules:

- at most one risk profile
- must be top-level inside `strategy()`
- must appear before data and intent logic

### 4.5 Execution module

The `execution` section declares the one supported retained execution profile.

Current `V1` pattern:

```qs
execution.profile("paper", fee_bps=10.0, slippage_bps=5.0)
```

Rules:

- at most one execution profile
- must be top-level inside `strategy()`
- must appear before data and intent logic

## 5. Naming convention

Use names that preserve the module chain.

Recommended prefixes:

- data variables:
  - `data_`
  - `series_`
  - `universe_`
- intent variables:
  - `signal_`
  - `score_`
  - `fast_`
  - `slow_`
- branch booleans:
  - `should_buy`
  - `should_sell`
  - `should_rebalance`

Examples:

```qs
let series_closes = fetch("BTCUSDT", interval="1d", lookback=200)?
let signal_fast_ma = sma(series_closes, 20)
let signal_slow_ma = sma(series_closes, 50)
let should_buy = signal_fast_ma > signal_slow_ma
```

Avoid vague names such as:

- `x`
- `value`
- `tmp`
- `score1`

## 6. Single-responsibility rule

Each section should answer one question only:

- `risk`: what global limits apply?
- `execution`: what paper execution defaults apply?
- `data`: what inputs enter the strategy?
- `intent`: what market condition is detected?
- `agent`: what action is taken?

If a line does not clearly belong to one of these questions, rewrite it.

## 7. Frontend display contract

Frontend display should present the strategy in this conceptual order:

1. data inputs
2. intent logic
3. agent behavior
4. risk profile
5. execution profile

But the source file may keep `risk` and `execution` first because of current lowering constraints.

So frontend should distinguish between:

- **source order**
- **conceptual pipeline order**

Recommended frontend presentation:

- show the conceptual module chain as:
  - `Data -> Intent -> Agent -> Risk -> Execution`
- show each extracted section with its real source snippet
- show arrows based on references:
  - fetch/get_data output -> indicator variables
  - indicator variables -> action branch
  - action branch -> implicit agent/rebalance
  - risk/execution profiles as attached policy modules

## 8. AI generation contract

AI-generated QuantScript should follow these rules:

1. Always emit section headers.
2. Always keep profile declarations at the top.
3. Always separate data acquisition from decision logic.
4. Always separate decision logic from action emission.
5. Never hide strategy meaning in deeply nested helper flows.
6. Prefer one decision variable per emitted action.

## 9. Recommended templates

### 9.1 Direct signal strategy

```qs
fn strategy() {
    # risk
    risk.profile("global", max_position=0.25)

    # execution
    execution.profile("paper", fee_bps=10.0, slippage_bps=5.0)

    # data
    let series_closes = fetch("BTCUSDT", interval="1d", lookback=200)?

    # intent
    let signal_fast_ma = sma(series_closes, 20)
    let signal_slow_ma = sma(series_closes, 50)
    let should_buy = signal_fast_ma > signal_slow_ma

    # agent
    if should_buy {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
```

### 9.2 Portfolio rebalance strategy

```qs
fn strategy() {
    # risk
    risk.profile("global", max_position=1.0)

    # execution
    execution.profile("paper", fee_bps=10.0, slippage_bps=5.0)

    # data
    let universe_base = top(
        sort_by(
            universe(exchange="binance", market="spot", quote="USDT"),
            key="volume_24h",
            order="desc"
        ),
        10
    )

    # intent
    let should_rebalance = true

    # agent
    if should_rebalance {
        rebalance(equal_weight(universe_base), every="1d")
    }
}
```

## 10. Explicit anti-patterns

Do not write strategies like this:

- interleave `fetch(...)` and `emit Intent(...)` line by line
- place `risk.profile(...)` below indicator branches
- hide execution defaults at the bottom of the strategy
- mix rebalance helpers with unrelated scalar indicator code
- use parser-only syntax to fake modular sophistication

Bad example:

```qs
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    if sma(closes, 20) > sma(closes, 50) {
        risk.profile("global", max_position=0.25)
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
```

This is harder to visualize, harder to audit, and violates the retained modular convention.

## 11. Release rule

For `V1`, a QuantScript strategy should be considered well-structured only if:

- it stays inside the retained executable trunk
- it follows the section ordering in this document
- its naming makes data flow obvious
- frontend can extract a stable pipeline view from it
- AI can revise one module without rewriting unrelated modules

That is the current definition of readable, frontend-friendly QuantScript authoring.
