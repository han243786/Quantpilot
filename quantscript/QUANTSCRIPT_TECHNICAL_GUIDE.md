# QuantScript Technical Guide

This guide is for professional developers who want to build real strategies with QuantScript as it exists today.

It follows the current `V1` executable trunk.
It does not describe deprecated config-style QuantScript or future language expansion.

Read this together with:

- [Supported Surface](./QUANTSCRIPT_SUPPORTED_SURFACE.md)
- [Real Strategy Authoring Trial](./QUANTSCRIPT_REAL_STRATEGY_AUTHORING_TRIAL.md)
- [Module Authoring Convention](./QUANTSCRIPT_MODULE_AUTHORING_CONVENTION.md)
- [QuantScript Authoring View Minimal Contract](./QUANTSCRIPT_AUTHORING_VIEW_MINIMAL_CONTRACT.md)
- [QuantScript Instrument Pool Minimal Contract](./QUANTSCRIPT_INSTRUMENT_POOL_MINIMAL_CONTRACT.md)
- [Formal Syntax Reference](../markdown/guides/quantscript/guide-formal-quantscript-syntax.md)

## 1. Mental model

QuantScript `V1` is a constrained strategy language that compiles into the current QuantPilot runtime.

Think of it as three layers:

1. syntax and name resolution
2. semantic analysis and diagnostics
3. lowering into runtime config, intents, rebalance directives, and backtest/report modules

The parser accepts more syntax than the runtime lowering path supports.
When writing production strategies, target the executable trunk only.

## 2. What a real QuantScript strategy looks like

A real executable strategy normally has:

- one top-level `fn strategy()`
- one or more `fetch(...)` / `get_data(...)` calls
- one or more `emit Intent(...)` statements
- optional top-level `risk.profile(...)`
- optional top-level `execution.profile(...)`

Minimal example:

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

## 3. Development workflow

Recommended workflow:

1. write formal QuantScript source
2. validate it through the formal analysis path
3. lower it to runtime config
4. run backtest / artifact / compare using the retained `V1` runtime surface

On a successful formal compile, the backend emits:

- `artifacts.strategy.metadata.quantscript_authoring_view`

On a failed formal compile, the backend may also emit:

- `partial_artifacts.quantscript_authoring_view`

Treat this as a source-first authoring/display artifact.
Do not treat it as a replacement for formal source or runtime config.
The frontend source workspace now consumes the same artifact for:

- source-order module display
- pipeline-order flow display
- pool-pipeline display when compile metadata includes a derived
  `InstrumentPoolSpec`

Failed-compile output is best-effort only.
It currently keeps line-based sections and may include `pool_pipeline` when
internal extraction succeeds before lowering fails.
The frontend source workspace consumes the same fallback artifact through
`partial_artifacts.quantscript_authoring_view`.
When this fallback path is active, source workspace should show an explicit
partial-artifact status notice near the top of the authoring view.
The source workspace now also includes a dedicated editable/apply-able formal
QuantScript lane. Authoring-view section, stage, edge, and pool-stage
highlighting should target that formal editor, not the separate `strategy_graph`
draft editor.
Workspace compile controls should also explicitly surface whether the current
compile is using graph-generated formal source or an applied formal override.

Use formal QuantScript entry points, not deprecated config-style APIs.

## 4. File structure

Supported top-level items:

- `import ...`
- `from ... import ...`
- `fn ...`

Your real strategy entry point must be:

```qs
fn strategy() {
    ...
}
```

Avoid treating `async fn`, `while`, `match`, or parser-only conveniences as production features.

## 5. Data access

Current retained data-source entry points:

- `fetch(...)`
- `get_data(...)`

Typical pattern:

```qs
let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
```

Practical rules:

- use postfix `?` only on fetch-like expressions
- request enough `lookback` to satisfy indicator warmup
- treat resolved fetch/get_data series as the root for indicator helpers

## 6. Indicator conditions that actually lower today

### 6.1 Direct moving-average compare

Recommended pattern:

```qs
let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
let fast = sma(closes, 20)
let slow = sma(closes, 50)

if fast > slow {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
```

### 6.2 One-sided RSI

Recommended pattern:

```qs
let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
let score = rsi(closes, 14)

if score < 30 {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
```

### 6.3 One-sided momentum

Recommended pattern:

```qs
let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
let score = momentum(closes, 20)

if score > 0 {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
```

### 6.4 One-sided z-score

Recommended pattern:

```qs
let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
let score = zscore(closes, 20)

if score < -2 {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
```

### 6.5 Narrow spread slice

Only use the explicit admitted slice:

```qs
fn strategy() {
    let left = align_asof(fetch("BTCUSDT", interval="1m", lookback=200)?, direction="backward", tolerance_ms=1000)
    let right = align_asof(fetch("ETHUSDT", interval="1m", lookback=200)?, direction="backward", tolerance_ms=1000)
    let spread_signal = spread(left, right, output="bps")

    if spread_signal > 5 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
```

Do not rely on:

- `ratio`
- `absolute`
- dual-sided spread logic
- line-vs-line spread compare

## 7. Intent emission

Current retained action surface:

- `BUY`
- `SELL`

Recommended form:

```qs
emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
```

Do not assume broader runtime action coverage.

## 8. Portfolio rebalance path

QuantScript `V1` supports a restricted compile-time rebalance workflow.

Current helper family:

- `symbols(...)`
- `universe(...)`
- `filter(...)`
- `sort_by(...)`
- `top(...)`
- `rebalance(...)`
- `equal_weight(...)`
- `fixed_weights(...)`
- `rank_weight(...)`
- `score_weight(...)`

Example:

```qs
fn strategy() {
    let base = top(
        sort_by(
            filter(
                universe(exchange="binance", market="spot", quote="USDT"),
                listed=true
            ),
            key="volume_24h",
            order="desc"
        ),
        10
    )

    rebalance(equal_weight(base), every="1d")
}
```

Treat this as a narrow retained portfolio path, not as a general collection/query language.

The documented next-phase replacement for this helper-centered pool path is:

- [QuantScript Instrument Pool Minimal Contract](./QUANTSCRIPT_INSTRUMENT_POOL_MINIMAL_CONTRACT.md)

That contract records design direction only.
It does not change the current retained support boundary.

## 9. Profiles

### 9.1 Risk profile

Supported:

```qs
risk.profile(
    "global",
    max_position=0.25,
    max_total_leverage=2.0,
    max_exchange_leverage=2.0,
    min_action_interval_ms=100
)
```

Rules:

- only one declaration
- only `profile_id="global"`
- top-level statement inside `strategy()`

### 9.2 Execution profile

Supported:

```qs
execution.profile(
    "paper",
    fee_bps=10.0,
    slippage_bps=5.0
)
```

Rules:

- only one declaration
- only `profile_id="paper"`
- top-level statement inside `strategy()`

## 10. Backtest, artifacts, and compare

Current retained `V1` workflow supports:

- execution assumptions modules
- artifact/detail/list visibility
- compare/report workflow
- metrics summary and drill-down
- trade-ledger summary and compare
- minimal equity-curve drill-down

This is enough for practical first-release strategy iteration, but it is not yet a full research platform.

## 11. Diagnostics you should expect

Resolver and semantic diagnostics:

- `QS0001` duplicate function definition
- `QS0002` unresolved identifier
- `QS0003` resolver/type failure
- `QS0005` invalid or unknown call target
- `QS0006` non-bool condition

Analysis and trunk-boundary diagnostics:

- `QS0401` look-ahead via negative indexing
- `QS0402` look-ahead via `center=true`
- `QS0403` invalid zero-length trailing window
- `QS0501` insufficient warmup
- `QS0601` async functions unsupported
- `QS0602` await unsupported
- `QS0603` while unsupported
- `QS0604` match unsupported
- `QS0605` recursion unsupported
- `QS0606` non-`Universe` `for` loop unsupported
- `QS0607` postfix `?` only on fetch-like expressions
- `QS0608` plain `import foo as bar` unsupported
- `QS0609` mutable list-building unsupported
- `QS0610` `.ok()` / `.retryable()` unsupported

Lowering diagnostics:

- `QPQSLOW001` through the current retained `QPQSLOW028` family

Read diagnostics as the contract.
If a feature is rejected by these codes, it is outside the retained `V1` trunk.

## 12. Recommended authoring rules

For production QuantScript `V1`, keep to these rules:

- write only formal QuantScript
- keep everything centered on `fn strategy()`
- use explicit `fetch(...)` / `get_data(...)`
- use one of the retained indicator or rebalance slices
- keep profile declarations top-level and singular
- avoid parser-only constructs even if they parse
- assume diagnostics are authoritative

## 13. What not to build against

Do not build your strategy development workflow around:

- config-style QuantScript
- parser-only syntax accepted by `script.rs`
- deprecated or compatibility-only APIs
- future roadmap notes
- archived docs

For real work, the order of authority should be:

1. current code
2. support surface
3. technical guide
4. syntax guide
5. current active contracts

## 14. Current gap summary

QuantScript `V1` is coherent enough for real strategy development, but with explicit limits:

- it is not a full shared-core indicator language
- it is not a broad spread DSL
- it is not a general-purpose language
- it is not yet a full research analytics platform

If you stay inside the retained trunk, it is usable now.
If you need anything broader, treat it as post-`V1` work requiring a new contract review.

## 15. Authoring convention

When writing production QuantScript, also follow the module convention in:

- [Module Authoring Convention](./QUANTSCRIPT_MODULE_AUTHORING_CONVENTION.md)

In practice this means:

- keep `risk.profile(...)` and `execution.profile(...)` first because current lowering requires top-level placement
- then write `data`
- then write `intent`
- then write `agent`

The frontend can still render the conceptual pipeline as:

- `Data -> Intent -> Agent -> Risk -> Execution`

But source authoring should remain stable and predictable.

---
> 文档版本: v3.7.0 | 最后更新: 2026-05-21 | QuantPilot v3.7.0
