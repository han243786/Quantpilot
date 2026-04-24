# QuantScript Supported Surface

This document is the release-facing support boundary for QuantScript `V1`.

It is intentionally narrower than the parser surface.
If code can be parsed but cannot be used to build a real executable strategy in the current runtime, it is not listed as supported here.

The implementation facts behind this document come from:

- `quantscript/src/lib.rs`
- `quantscript/src/script.rs`
- `quantscript/src/resolve.rs`
- `quantscript/src/analysis.rs`
- `quantscript/src/lowering/orchestrator.rs`
- `quantscript/src/lowering/intents.rs`
- `quantscript/src/lowering/universe.rs`

## Release position

QuantScript can be used for real strategy development in `V1`, but only inside a constrained executable trunk.

QuantScript `V1` is suitable for:

- single-strategy formal QuantScript files centered on `fn strategy()`
- data fetch + indicator condition + `emit Intent(...)` workflows
- a narrow set of structured portfolio rebalance helpers
- a narrow set of runtime profile declarations
- a narrow backtest and compare workflow built around the current runtime

QuantScript `V1` is not a general-purpose programming language, not a full quantitative research DSL, and not a complete indicator platform.

## Supported product path

Supported product path:

- `parse_formal_quant_script_config(...)`
- `parse_formal_quant_script_typed_hir(...)`
- `analyze_formal_quant_script(...)`

Compatibility-only path:

- deprecated config-style QuantScript parsing and compilation in `quantscript/src/lib.rs`

For new development, use formal QuantScript source only.

Retained positive examples live in `quantscript/authoring_samples/`.
Intentional retained-boundary failures live in `quantscript/boundary_samples/`
and must not be treated as supported authoring fixtures.

## Required executable contract

An executable strategy currently requires:

- a top-level `fn strategy() { ... }`
- at least one reachable `fetch(...)` or `get_data(...)` call
- at least one reachable `emit Intent(...)`

Current runtime-oriented optional top-level declarations inside `strategy()`:

- `risk.profile("global", ...)`
- `execution.profile("paper", ...)`

Both must appear as top-level statements inside `fn strategy()`.

## Supported syntax that is part of the executable trunk

### File structure

Supported top-level items:

- `import ...`
- `from ... import ...`
- `fn ...`

Parsed but not part of the executable trunk:

- `async fn ...`

### Statements

Supported in real strategy development:

- `let ... = ...`
- `return ...`
- `if / else if / else`
- `emit Intent(...)`
- `for ... in ...` only when iterating a `Universe`-valued expression

Parsed but rejected for executable strategies:

- `while`
- `match`

### Expressions

Supported in the current executable trunk:

- identifiers
- numeric / string / bool literals
- lists
- function calls
- member access
- indexing and trailing windows such as `series[20..]`
- boolean and arithmetic comparisons needed by current lowering
- postfix `?` only on fetch-like expressions

Parsed but rejected or unsupported for executable strategies:

- `await`
- postfix `?` on non-fetch expressions
- mutable list building via `.push(...)`
- convenience helpers such as `.ok()` and `.retryable()`

## Supported data access

Supported data-source entry points:

- `fetch(...)`
- `get_data(...)`

Current support expectation:

- fetch-like calls produce the data source backbone for lowering
- indicator helpers must ultimately root in recognized fetch/get_data sources

## Supported indicator and intent lowering surface

### Shared-core conditions currently supported

Current stable shared-core slices:

- direct moving-average compare
- one-sided `RSI`
- one-sided `momentum`
- one-sided `zscore`
- first narrow spread slice

### Moving averages

Supported:

- direct MA compare forms using recognized moving-average helpers such as `sma(...)` and `ema(...)`
- `BUY` and `SELL` conditional emit lowering around the current MA compare slice

Not supported as `V1` shared-core scope:

- broader MA formula pattern mining beyond the retained lowering slice
- general indicator algebra advertised as product capability

### RSI

Supported:

- one-sided threshold conditions that lower into the retained structured RSI compare path

Not supported:

- broader RSI DSL
- arbitrary RSI helper ecosystems beyond the retained lowering slice

### Momentum

Supported:

- one-sided momentum threshold conditions in the retained lowering slice

### Z-score

Supported:

- one-sided z-score threshold conditions in the retained lowering slice

### Spread

Supported formal QuantScript spread slice:

- explicit `align_asof(...) + spread(..., output="bps") + one-sided >/>= threshold`

Scope constraints:

- exactly two inputs
- explicit `align_asof(...)`
- positive `tolerance_ms`
- `output="bps"`
- one-sided threshold only

Not supported in `V1`:

- `ratio`
- `absolute`
- dual-sided spread conditions
- line-vs-line spread compare
- timeline compare

### Intent actions

Stable runtime-lowered actions:

- `BUY`
- `SELL`

Unsupported runtime actions are rejected with structured lowering diagnostics.

## Supported universe and rebalance surface

Supported compile-time universe/rebalance helper family:

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

Current constraints:

- compile-time universe operations require `universe_snapshot`
- only one `rebalance(...)` directive is supported
- supported rebalance cadence values are:
  - `"slow"`
  - `"1d"`
  - `"weekly"`
- `rank_weight(..., method=...)` currently supports only:
  - `"linear"`
  - `"inverse_rank"`
- `score_weight(..., normalize=...)` currently supports only:
  - `"sum"`

This is a restricted portfolio rebalance path, not a general universe DSL.

## Supported outward-moved profile surface

### Risk profile

Supported:

- `risk.profile("global", max_position=..., max_total_leverage=..., max_exchange_leverage=..., min_action_interval_ms=...)`

Current constraints:

- only `profile_id="global"`
- only one declaration
- top-level statement inside `fn strategy()`

### Execution profile

Supported:

- `execution.profile("paper", fee_bps=..., slippage_bps=...)`

Current constraints:

- only `profile_id="paper"`
- only one declaration
- top-level statement inside `fn strategy()`

## Supported diagnostics boundary

Current stable diagnostic families:

- resolver and semantic diagnostics:
  - `QS0001`
  - `QS0002`
  - `QS0003`
  - `QS0005`
  - `QS0006`
- executable trunk diagnostics:
  - `QS0401`
  - `QS0402`
  - `QS0403`
  - `QS0501`
  - `QS0601`
  - `QS0602`
  - `QS0603`
  - `QS0604`
  - `QS0605`
  - `QS0606`
  - `QS0607`
  - `QS0608`
  - `QS0609`
  - `QS0610`
- structured lowering diagnostics:
  - `QPQSLOW001` through `QPQSLOW028` where implemented in the current lowering stack

These diagnostics are part of the current product boundary.

## Explicitly not supported in V1

Not supported for release-facing strategy development:

- general-purpose asynchronous programming
- recursion
- mutable collection workflows
- arbitrary parser-accepted constructs treated as product capability
- broad indicator synthesis outside the retained lowering slices
- broad spread DSL expansion
- `MACD` shared-core expansion
- broader risk / execution DSLs
- per-trade compare
- fill timeline compare
- full research-report platform behavior

## Comparison to the current development goal

Current development goal:

- determine whether QuantScript can already be used for real strategy development
- if yes, document the retained support boundary honestly

Comparison result:

- **Pass:** there is a coherent executable trunk for real strategy development in `V1`
- **Pass:** the runtime/backtest/compare stack now has a retained product surface around that trunk
- **Gap:** parser surface is still broader than product surface, so developer-facing docs must keep saying "parsed" is not equal to "supported"
- **Gap:** legacy config-style QuantScript still exists for compatibility and must not be documented as the preferred path
- **Gap:** one existing Markdown source in the old guide set still appears to have encoding issues in local inspection, so the new support documents should be treated as the release-facing source of truth until that older file is repaired

## Practical release summary

QuantScript `V1` is ready for constrained first-release strategy development if you stay inside:

- formal QuantScript source
- `fn strategy()` executable trunk
- retained indicator slices
- retained rebalance helpers
- retained `risk.profile("global")`
- retained `execution.profile("paper")`

Anything outside that boundary should be treated as deferred, compatibility-only, or parse-only.
