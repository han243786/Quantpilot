# QuantScript AI Guide

This document is for general-purpose AI systems that need to generate usable QuantScript `V1`.

Its goal is simple:

- generate QuantScript that compiles and lowers today
- avoid parser-only or roadmap-only features
- stay inside the retained strategy-development trunk

Use this file as an operational rulebook, not as a language wishlist.

For layout and naming, also follow:

- [Module Authoring Convention](./QUANTSCRIPT_MODULE_AUTHORING_CONVENTION.md)
- [QuantScript Authoring View Minimal Contract](./QUANTSCRIPT_AUTHORING_VIEW_MINIMAL_CONTRACT.md)
- [QuantScript Instrument Pool Minimal Contract](./QUANTSCRIPT_INSTRUMENT_POOL_MINIMAL_CONTRACT.md)

## 1. Core rule

Write only what the current runtime can actually lower.

Do not assume that "the parser accepts it" means "the product supports it".

If you want frontend tooling and future AI patching to stay stable, prefer
explicit section comments:

- `# risk`
- `# execution`
- `# data`
- `# intent`
- `# agent`

These comments are not new syntax.
They are authoring hints that improve the current `quantscript_authoring_view`
artifact, which now drives the frontend source-order module view and
pipeline-order flow view, and the read-only pool-pipeline view when compile
metadata includes derived pool semantics.

## 2. Default strategy template

Unless the user explicitly needs the retained rebalance path, generate strategies in this shape:

```qs
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?

    # indicator logic

    if CONDITION {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
```

If you do not know which symbol to use, ask the user or choose a clearly stated placeholder.

## 3. Safe building blocks

Prefer these building blocks:

- `fn strategy()`
- `fetch(...)`
- `get_data(...)`
- `let`
- `if / else if / else`
- `emit Intent(...)`
- `sma(...)`
- `ema(...)`
- `rsi(...)`
- `momentum(...)`
- `zscore(...)`
- `align_asof(...)`
- `spread(..., output="bps")`
- `risk.profile("global", ...)`
- `execution.profile("paper", fee_bps=..., slippage_bps=...)`
- retained rebalance helpers:
  - `universe(...)`
  - `filter(...)`
  - `sort_by(...)`
  - `top(...)`
  - `rebalance(...)`
  - `equal_weight(...)`
  - `fixed_weights(...)`
  - `rank_weight(...)`
  - `score_weight(...)`

## 4. Unsafe or forbidden building blocks

Do not generate these for `V1` strategy work:

- `async fn`
- `await`
- `while`
- `match`
- recursion
- `.push(...)`
- `.ok()`
- `.retryable()`
- plain `import foo as bar`
- postfix `?` on anything except fetch/get_data-like expressions
- broader spread forms such as:
  - `spread(..., output="ratio")`
  - `spread(..., output="absolute")`
  - dual-sided spread logic
  - line-vs-line spread compare
- wider roadmap-only features such as:
  - `MACD` shared-core expansion
  - per-trade compare
  - fill timeline compare

## 5. Supported strategy patterns

### 5.1 Direct moving-average crossover

Use:

```qs
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)

    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
```

### 5.2 One-sided RSI

Use:

```qs
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = rsi(closes, 14)

    if score < 30 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
```

### 5.3 One-sided momentum

Use:

```qs
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = momentum(closes, 20)

    if score > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
```

### 5.4 One-sided z-score

Use:

```qs
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = zscore(closes, 20)

    if score < -2 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
```

### 5.5 Narrow spread slice

Only use this exact family:

```qs
fn strategy() {
    let left = align_asof(fetch("BTCUSDT", interval="1m", lookback=200)?, direction="backward", tolerance_ms=1000)
    let right = align_asof(fetch("ETHUSDT", interval="1m", lookback=200)?, direction="backward", tolerance_ms=1000)
    let score = spread(left, right, output="bps")

    if score > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
```

Do not improvise broader spread syntax.

## 6. Rebalance pattern

Only generate rebalance strategies when the user explicitly asks for portfolio allocation behavior.

Use patterns like:

```qs
fn strategy() {
    let base = top(
        sort_by(
            universe(exchange="binance", market="spot", quote="USDT"),
            key="volume_24h",
            order="desc"
        ),
        10
    )

    rebalance(equal_weight(base), every="1d")
}
```

Rules:

- only one `rebalance(...)`
- only `every="slow"`, `"1d"`, or `"weekly"`
- `fixed_weights(...)` requires one numeric weight per symbol
- `rank_weight(..., method=...)` only `"linear"` or `"inverse_rank"`
- `score_weight(..., normalize=...)` only `"sum"`

## 7. Profiles

If the user asks for runtime risk or paper execution defaults, you may emit:

```qs
risk.profile("global", max_position=0.25, max_total_leverage=2.0, max_exchange_leverage=2.0, min_action_interval_ms=100)
execution.profile("paper", fee_bps=10.0, slippage_bps=5.0)
```

Rules:

- each appears at most once
- each must be a top-level statement inside `fn strategy()`
- only `global` and `paper` are valid current profile ids

## 8. Authoring checklist for AI

Before returning QuantScript, check:

1. Is there exactly one `fn strategy()`?
2. Is there at least one `fetch(...)` or `get_data(...)`?
3. Is there at least one `emit Intent(...)`?
4. Are all indicator helpers rooted in real data sources?
5. Did you avoid parser-only constructs?
6. Did you stay inside retained spread / rebalance / profile slices?
7. Did you avoid deprecated config-style QuantScript?

If any answer is "no", rewrite before returning.

## 9. Error-aware rewrite rules

If you encounter these diagnostics, rewrite as follows:

- `QS0601` / `QS0602`
  - remove async/await
- `QS0603`
  - replace `while` with a straight-line retained pattern
- `QS0604`
  - replace `match` with `if / else if / else`
- `QS0605`
  - remove recursion
- `QS0606`
  - only iterate real `Universe` expressions
- `QS0607`
  - move postfix `?` back onto fetch/get_data calls only
- `QS0608`
  - replace `import foo as bar` with `from foo import thing as alias`
- `QS0609`
  - replace mutable list building with direct literal or retained helper flow
- `QS0610`
  - remove `.ok()` / `.retryable()`
- `QPQSLOW001`
  - simplify the condition to a retained MA / RSI / momentum / zscore / narrow spread form
- `QPQSLOW006`
  - add `fn strategy()`
- `QPQSLOW007`
  - add a fetch/get_data source

## 10. How to answer when the user asks for unsupported features

Do not fake support.

If the user asks for something outside the retained surface:

- say that current QuantScript `V1` does not support it as a stable strategy-development feature
- offer the nearest retained rewrite
- keep the rewrite inside the current executable trunk

Example:

- user asks for broad `MACD` shared-core
- you answer with a retained MA / RSI / momentum / z-score / narrow spread alternative, or clearly say it is post-`V1`

For cross-sectional work:

- current retained rule is still metadata-ranked selection plus per-symbol signal
  gating
- do not claim that dynamic factor-ranked universe construction is supported yet
- the next-phase design direction is documented in
  [QuantScript Instrument Pool Minimal Contract](./QUANTSCRIPT_INSTRUMENT_POOL_MINIMAL_CONTRACT.md)

## 11. Final rule

When in doubt, generate less.

A smaller QuantScript program that clearly fits the retained trunk is better than a broader program that only parses.

Also keep the source organized as:

- `risk`
- `execution`
- `data`
- `intent`
- `agent`

with explicit section comments, even when the frontend later renders the conceptual pipeline as:

- `Data -> Intent -> Agent -> Risk -> Execution`
