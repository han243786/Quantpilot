# Formal QuantScript Syntax Guide

This document is the current syntax reference for formal QuantScript in QuantPilot.

It describes the language that is actually implemented today by:

- `quantscript/src/script.rs`
- `quantscript/src/types.rs`
- `quantscript/src/resolve.rs`
- `quantscript/src/analysis.rs`
- `quantscript/src/lowering/mod.rs`

Current lowering is internally split across:

- `quantscript/src/lowering/orchestrator.rs`
- `quantscript/src/lowering/context.rs`
- `quantscript/src/lowering/shared.rs`
- `quantscript/src/lowering/semantic.rs`
- `quantscript/src/lowering/diagnostics.rs`
- `quantscript/src/lowering/universe.rs`
- `quantscript/src/lowering/binding_sources.rs`
- `quantscript/src/lowering/source_recovery.rs`
- `quantscript/src/lowering/bindings.rs`
- `quantscript/src/lowering/fallback.rs`
- `quantscript/src/lowering/intents.rs`

If an older roadmap, research note, or archive document disagrees with this file about current syntax, this file wins.

## Development baseline

This file is the source of truth for what the parser, resolver, analysis, and lowering path actually implement today.

It is not the source of truth for future language expansion.

Future QuantScript development must follow:

- [QuantScript Trunk Baseline](./guide-quantscript-trunk-baseline.md)

That means:

- parser-accepted syntax is not automatic product endorsement
- parse-only legacy constructs must not be used to justify broadening QuantScript into a general-purpose language
- when current syntax is broader than the trunk baseline, future work should converge toward the baseline instead of expanding the broader surface

## Scope

This guide covers the formal QuantScript product path carried in `quantscript.formal_source`.

It does not describe:

- `strategy_graph` graph-source import/export text
- the deprecated section-based config-style QuantScript
- future Typed HIR proposals that are not implemented yet

## Current lowering contract

Formal QuantScript is not a general-purpose language. The executable path currently expects:

- a top-level `fn strategy() { ... }`
- at least one `fetch(...)` or `get_data(...)` call reachable from `strategy`
- at least one `emit Intent(...)` in `strategy`
- multi-symbol strategies, when used, must still lower into a finite expanded set of per-symbol `fetch(...)` and `emit Intent(...)` statements at compile time
- optional `risk.profile("global", ...)` may appear as a single top-level statement inside `strategy`, and only lowers to the existing `builtin.risk.global` runtime module
- optional `execution.profile("paper", fee_bps=..., slippage_bps=...)` may appear as a single top-level statement inside `strategy`, and only lowers to the existing `builtin.execution.paper` runtime module
- spread semantics remain narrower than the current helper surface; the landed formal spread slice is limited to explicit `align_asof(...) + spread(..., output="bps") + one-sided >/>=` and broader parsed `spread(...)` shapes are still not stable shared-core capability

The parser accepts more syntax than the runtime lowering path guarantees.
Retained executable examples should come only from `quantscript/authoring_samples/`;
intentional rejection fixtures belong in `quantscript/boundary_samples/` so the
active authoring surface does not mix success and failure samples.
This guide marks that boundary explicitly.

## Lexical rules

### Comments

- Line comments start with `#`.
- There is no block comment syntax.
- Comment stripping is line-based.

Important limitation:

- `#` is protected inside double-quoted strings.
- `#` is not reliably protected inside single-quoted strings.
- If a string may contain `#`, prefer double quotes.

### Whitespace and layout

- The parser is line-oriented.
- Empty lines are ignored.
- Blocks are delimited by `{` and `}`.
- Function / `if` / `else if` / `else` / `for` / `while` / `match` headers must end with `{` on the same line.
- `} else if ... {` and `} else {` on one line are normalized and accepted.

### Identifiers

In expressions, identifiers are tokenized from ASCII letters, digits, and `_`.

Use simple ASCII names such as:

```qs
closes
fast_ma
signal_1
```

## File structure

A formal QuantScript file may contain only:

- `import ...`
- `from ... import ...`
- `fn ...`
- `async fn ...`

Anything else at the top level is rejected.

### Simplified top-level grammar

```text
module      := item*
item        := import_decl | from_import_decl | function_decl
import_decl := "import" module_name
from_import := "from" module_ref "import" import_name ("," import_name)*
function    := ["async"] "fn" name "(" params? ")" ["->" type] "{"
```

## Imports

### Plain import

```qs
import math
import signals
```

### From-import

```qs
from data import fetch
from data import fetch as get_data
from signals@1.2 import rsi, macd
from transforms import field, resample, align_asof
```

Rules:

- `from module import a, b, c` is supported.
- `from module@version import ...` is supported.
- `as` aliases are supported only in `from ... import ... as ...`.
- Plain `import foo as bar` is not supported.
- formal QuantScript compile now rejects plain module-alias imports such as `import foo as bar` with `QS0608`.
- Imports are parsed syntactically even if the helper is not meaningful for runtime lowering.

## Functions

### Syntax

```qs
fn strategy() {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}

fn moving_average(series: Series<Number>, period: Number) -> Number {
    return series[period..].sum() / period
}

async fn preload(symbols: List<String>) -> List<String> {
    return symbols
}
```

Rules:

- `fn` and `async fn` are both parsed.
- Parameters are comma-separated.
- Each parameter may optionally have a type annotation: `name: Type`.
- The function may optionally have a return type: `-> Type`.
- The current executable contract still centers on `fn strategy()`.

Practical boundary:

- Helper functions are supported and can participate in normalization/lowering.
- Recursive functions are not supported.
- formal QuantScript compile now rejects direct recursive helper calls with `QS0605`
- `async fn` and `await` are parseable legacy syntax, but they are not the stable runtime-lowering contract for executable strategy code and are not part of the future trunk direction.
- formal QuantScript compile now rejects `async fn` with `QS0601` and `await` expressions with `QS0602`

## Type annotations

The parser accepts the following type names:

- `Unknown`
- `Unit`
- `Bool` / `bool`
- `Number` / `number`
- `String` / `string`
- `Symbol` / `symbol`
- `Universe` / `universe`
- `Signal` / `signal`
- `Scalar<T>`
- `Series<T>`
- `Maybe<T>`
- `List<T>`

Examples:

```qs
fn helper(series: Series<Number>, period: Number) -> Number {
    return series[period..].mean()
}

fn names() -> List<String> {
    return ["BTCUSDT", "ETHUSDT"]
}
```

Notes:

- Type annotations are optional.
- Unsupported type names are rejected.
- Even when a type annotation parses, current lowering still relies heavily on resolver-inferred series/number semantics.
- `Symbol` and `Universe` are now recognized by the resolver and lowering path, but `Universe` is still a restricted capability, not a general collection API.

## Statements

### `let`

```qs
let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
let fast: Number = sma(closes, 20)
let mut out = []
```

Rules:

- Syntax: `let [mut] pattern [: Type] = expr`
- `mut` is parsed.
- The binding pattern is stored as text; simple identifier bindings are the supported path.

Important limitation:

- There is no standalone assignment statement like `x = y`.
- Use `let` for binding introduction.
- Mutable list-building conveniences such as `out.push(...)` are not part of the formal executable trunk; formal QuantScript compile now rejects them with `QS0609`.

### `return`

```qs
return
return score
return closes[20..].mean()
```

### `emit Intent(...)`

```qs
emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
emit Intent("SELL", instrument="BTCUSDT", quantity=0.5)
```

Rules:

- `emit Intent(...)` is a dedicated statement form.
- Arguments may be positional or named.
- Named arguments may use either `:` or `=`.

Examples:

```qs
emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
emit Intent(action="BUY", instrument="BTCUSDT", quantity=1.0)
```

Current lowering boundary:

- The runtime path requires an action.
- Executable lowering also requires at least one data source to be inferable from the strategy.
- In practice, keep `emit Intent(...)` close to standard trading fields such as `instrument` and `quantity`.
- Conditional `emit Intent(...)` is not a generic fallback surface.
- If the surrounding condition does not map to a supported indicator or spread intent, lowering now rejects the script instead of silently producing a generic runtime intent.
- Known lowering-contract failures are now surfaced as structured formal QuantScript compile diagnostics.
- When formal QuantScript compile succeeds, the resulting `core_ir.metadata.source_kind` is now explicitly `formal_quant_script` instead of being folded into the generic runtime-protocol source label.
- Direct single-source moving-average comparisons such as `if sma(data, 20) > sma(data, 100)` now lower into a structured Core IR predicate: `ScalarExpr::Compare` over shared `SeriesExpr::WindowAgg` nodes, instead of only falling back to raw condition text.
- Direct one-sided RSI threshold comparisons such as `if rsi(data, 14) < 25` now also lower into a structured Core IR predicate over the shared indicator reference plus numeric threshold. The current runtime-intent shape still merges dual-sided RSI buy/sell contracts into one node, so two-sided RSI forms continue to fall back to raw condition text for now.
- Direct one-sided `momentum` and `zscore` threshold comparisons such as `if momentum(data, 20) > 0.03` or `if zscore(data, 20) < -2` now also lower into structured Core IR predicates over the lowered indicator reference plus the original signed threshold. Dual-sided forms still remain on the raw-text path when lowering must merge both sides into a single runtime intent.
- The first landed formal spread slice now also lowers into a structured Core IR predicate, but only for the narrow helper form `spread(align_asof(...), align_asof(...), output="bps") > threshold` or `>= threshold`; ratio output, absolute output, `<` / `<=`, and helper-derived spread arithmetic remain outside the admitted formal surface.
- Examples include `QPQSLOW001` for unsupported conditional `emit Intent(...)` or malformed spread-helper conditions, `QPQSLOW004` for unsupported runtime actions, and `QPQSLOW007` when strategy lowering cannot infer any reachable `fetch(...)` or `get_data(...)` source.
- Universe/rebalance contract failures now also surface as structured diagnostics, including `QPQSLOW009` for unsupported `rebalance(..., every=...)` values, `QPQSLOW010` when snapshot-dependent universe operations are compiled without `universe_snapshot`, and `QPQSLOW012` for unsupported universe sort orders.
- Universe input-shape contracts are also moving into structured diagnostics: `QPQSLOW025` when helpers like `filter/sort_by/top` are missing their universe input or do not receive a universe-valued input, `QPQSLOW026` when `symbols(...)` is missing its list input or does not receive a list literal, `QPQSLOW027` when `symbols([...])` contains non-string items, and `QPQSLOW028` when `top(...)` does not receive a numeric count argument.
- Allocation/weights constraints are also beginning to use structured diagnostics, including `QPQSLOW013` when `rebalance(...)` is missing its allocation helper or does not receive an allocation helper, `QPQSLOW014` when an allocation helper is missing its selection input or when that input is not universe-valued, `QPQSLOW015` when the allocation resolves to an empty symbol set, `QPQSLOW016` for `fixed_weights` count mismatch, `QPQSLOW017` for negative fixed weights, `QPQSLOW018` for zero-total fixed weights, `QPQSLOW019` for unsupported `rank_weight(..., method=...)` values, `QPQSLOW020` for unsupported `score_weight(..., normalize=...)` values, and `QPQSLOW021` when `weights=...` is missing or is not a numeric list literal.
- Indicator input contracts are also moving into structured lowering diagnostics: `QPQSLOW022` for helpers like `rsi/macd/momentum/zscore` when the first argument is missing or does not lower to a `fetch/get_data` source, `QPQSLOW023` when period/lookback/window arguments are missing, non-numeric, or non-positive, and `QPQSLOW024` when moving-average helpers are missing their source input, do not receive a valid source input, or do not receive a recognized `MACD` line in the `ema(...)` compatibility path.

### `if / else if / else`

```qs
if fast > slow {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
} else if fast < slow {
    emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
} else {
    log_warn("flat")
}
```

### `for`

```qs
for s in selected {
    let closes = fetch(s, interval="1d", lookback=200)?
    emit Intent("BUY", instrument=s, quantity=1.0)
}
```

Boundary:

- iteration over `Universe` is supported for formal lowering
- non-`Universe` `for` loops are parseable legacy syntax, but formal QuantScript compile now rejects them with `QS0606`
- the current lowering path expands the loop into separate per-symbol branches before runtime compile
- this is not a general runtime portfolio loop or dynamic universe state machine
- this compatibility surface must not be treated as a license to expand general loop semantics in the main language

### `while`

```qs
while i < 10 {
    log_warn("loop")
}
```

Current status:

- `while` is parseable syntax only
- it is outside the recommended stable executable trunk
- formal QuantScript compile now rejects it early with `QS0603`
- future QuantScript development should not expand `while` into a general-purpose strategy control-flow surface

### `match`

```qs
match read_data("BTCUSDT") {
    Ok(k) => emit Intent("BUY", instrument="BTCUSDT", quantity=1.0),
    Err(e) => log_error(e)
}
```

Important limitations:

- Match arms are line-based.
- Each arm body is either:
  - a single expression, or
  - a single `emit Intent(...)` statement
- Block-style match arms are not supported.
- Match patterns are stored as raw text; the pattern language is not a formally specified type-checked pattern system today.
- `match` is not part of the recommended future trunk and should be treated as a limited legacy/compatibility surface unless a narrower IR-backed product need is proven
- formal QuantScript compile now rejects it early with `QS0604`

### Expression statements

Any expression may be used as a statement:

```qs
log_warn("retry")
```

## Expressions

### Literals

```qs
42
3.14
"BTCUSDT"
'BUY'
true
false
[1, 2, 3]
[]
```

Notes:

- Numeric literals are decimal numbers.
- Negative numbers are parsed as unary `-` applied to a positive number.
- Both single-quoted and double-quoted strings are tokenized.
- Double-quoted strings support escapes such as `\"`, `\\`, `\n`, `\t`.
- Single-quoted strings are simpler and should be treated as plain text literals.

### Calls

```qs
fetch("BTCUSDT", interval="1d", lookback=200)
sma(closes, 20)
align_asof(series, direction="nearest", tolerance_ms=10000)
```

Call arguments may be:

- positional: `sma(closes, 20)`
- named with `=`: `fetch("BTCUSDT", interval="1d")`
- named with `:`: `helper(period: 14)`

When using the current universe helpers, `fetch(...)` may also receive a `Symbol`-typed loop binding as positional argument 0:

```qs
let closes = fetch(s, interval="1d", lookback=200)?
```

### Member access

```qs
closes.mean()
closes.last()
scope.stddev()
```

### Indexing

```qs
closes[0]
closes[14]
closes[-1]
```

Notes:

- Negative indices parse.
- They may still trigger semantic diagnostics such as look-ahead risk.

### Slicing

```qs
closes[20..]
closes[..20]
closes[10..20]
```

### Ranges

```qs
1..10
start..end
```

### Prefix operators

```qs
-value
!flag
not flag
await task
```

### Postfix operators

```qs
fetch("BTCUSDT", interval="1d", lookback=200)?
get_data("BTCUSDT")?
```

`?` is parsed as a postfix try operator and is commonly used on `fetch(...)` / `get_data(...)`.

- formal QuantScript compile now rejects postfix `?` on non-fetch-like expressions with `QS0607`.
- in the current executable trunk, postfix `?` is a fetch-like data-source convenience, not a general result/error propagation feature.

### Binary operators

Supported infix operators:

- `*`, `/`, `%`
- `+`, `-`
- `>`, `>=`, `<`, `<=`
- `==`, `!=`
- `&&`, `||`
- `and`, `or`

Not supported:

- bitwise `&`
- bitwise `|`

### Precedence

From low to high:

1. range `..`
2. logical `or` / `||`
3. logical `and` / `&&`
4. equality `== !=`
5. comparison `> >= < <=`
6. additive `+ -`
7. multiplicative `* / %`
8. prefix `await`, unary `-`, unary `!`
9. postfix call / member / index / slice / `?`

## Builtins and helpers

The parser accepts arbitrary call names.
The resolver and lowering layers only give special meaning to a subset.

### Fetch-like sources

- `fetch`
- `get_data`

Current runtime-lowering defaults for `fetch` / `get_data`:

- positional arg 0: symbol string or `Symbol` binding, default `BTCUSDT`
- named `exchange`, default `binance`
- named `interval`, default `1d`
- named `lookback`, default `200`

Example:

```qs
let closes = fetch("BTCUSDT", exchange="okx", interval="1m", lookback=200)?
```

### Universe helpers recognized today

- `symbols`
- `universe`
- `filter`
- `sort_by`
- `top`
- `equal_weight`
- `fixed_weights`
- `rank_weight`
- `score_weight`
- `rebalance`

These helpers are only supported for the current restricted compile-time universe path.

Supported forms:

```qs
let selected = symbols(["BTCUSDT", "ETHUSDT"])
```

```qs
let base = universe(exchange="binance", market="spot", quote="USDT")
let ranked = sort_by(base, key="market_cap", order="desc")
let selected = top(ranked, 10)
```

Current semantics:

- `symbols([...])` accepts a list of string literals and returns `Universe`
- `universe(...)` reads from the compile request's `universe_snapshot`
- `universe_snapshot.as_of_ms` is now part of the lowering contract for metadata-backed selection
- each `UniverseAssetRecord` may now carry:
  - flat metadata such as `market_cap`, `volume_24h`, and `listing_age_days`
  - `listed_at_ms` for point-in-time listing eligibility
  - `metadata_history`, where the latest point at or before `as_of_ms` is used
- `filter(...)` currently supports snapshot-backed filtering keys such as:
  - `quote`
  - `exchange`
  - `market`
  - `min_market_cap`
  - `min_volume_24h`
  - `min_listing_age_days`
- `sort_by(...)` currently supports:
  - `key="symbol"`
  - `key="market_cap"` and this requires `universe_snapshot`
  - `key="volume_24h"` and this requires `universe_snapshot`
  - `key="listing_age_days"` and this requires `universe_snapshot`
- `top(...)` truncates a `Universe` to the first `N` entries after previous filtering/sorting
- `equal_weight(universe_expr)` currently marks a selected `Universe` for equal-weight portfolio rebalance lowering
- `fixed_weights(universe_expr, weights=[...])` currently assigns a fixed normalized weight vector to a selected `Universe`
- `rank_weight(universe_expr, method="linear" | "inverse_rank")` currently ranks selected symbols by signal score and derives weights from rank order
- `score_weight(universe_expr, normalize="sum")` currently normalizes selected signal scores into weights
- `rebalance(<allocation>, every="1d")`
- `rebalance(<allocation>, every="slow")`
- `rebalance(<allocation>, every="weekly")`
  currently enable the formal portfolio-rebalance lowering path

Current hard boundary:

- these helpers are not general-purpose collection transforms
- they are not evaluated continuously at runtime
- they are resolved once during formal lowering
- metadata-backed selection is now point-in-time aware at `universe_snapshot.as_of_ms`, but still only for the single compile-time snapshot supplied in the request
- current `for s in selected { ... }` loops still expand into concrete per-symbol branches
- current `rebalance(equal_weight(...), ...)` lowering instead carries an explicit target universe into runtime portfolio rebalance
- these allocation helpers are not a general allocation DSL
- `rebalance(...)` currently supports only:
  - `equal_weight(universe_expr)`
  - `fixed_weights(universe_expr, weights=[...])`
  - `rank_weight(universe_expr, method="linear" | "inverse_rank")`
  - `score_weight(universe_expr, normalize="sum")`
- `rebalance(...)` currently supports only the `every="slow"` / `every="1d"` / `every="weekly"` cadence forms

### Restricted portfolio rebalance helper

The current formal QuantScript path now supports a minimal portfolio-level rebalance entry.

Supported form:

```qs
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
```

Additional supported forms:

```qs
let base = symbols(["BTCUSDT", "ETHUSDT"])
rebalance(fixed_weights(base, weights=[0.7, 0.3]), every="slow")
```

```qs
let selected = top(sort_by(base, key="market_cap", order="desc"), 3)
rebalance(rank_weight(selected, method="inverse_rank"), every="1d")
```

```qs
let selected = top(sort_by(base, key="market_cap", order="desc"), 3)
rebalance(score_weight(selected, normalize="sum"), every="1d")
```

Current executable semantics:

- `rebalance(equal_weight(selected), ...)` does not create a separate portfolio object in script space
- it marks the lowered agent as a portfolio rebalance agent
- the selected `Universe` is carried into backend/runtime as the explicit rebalance target symbol set
- the backend/runtime now stores cadence as a typed `RebalanceSchedule` instead of a string-only helper flag
- the current runtime first builds a backend `PortfolioTargetDecision` using the selected allocation model
- risk currently passes that portfolio target through to execution without a separate portfolio-constraint DSL
- execution then compares current holdings against the target weights and generates the final buy/sell basket
- symbols in the explicit rebalance target set may still be sold down to zero even if they do not emit a fresh signal in the current evaluation pass

Current allocation semantics:

- `equal_weight(...)`
  - assigns the same target weight to every selected symbol
- `fixed_weights(..., weights=[...])`
  - requires one numeric weight per selected symbol
  - weights are normalized during lowering so they sum to 1
- `rank_weight(..., method="linear")`
  - sorts selected symbols by signal score descending
  - assigns weights proportional to `N, N-1, ..., 1`
- `rank_weight(..., method="inverse_rank")`
  - sorts selected symbols by signal score descending
  - assigns weights proportional to `1/1, 1/2, ..., 1/N`
- `score_weight(..., normalize="sum")`
  - assigns weights proportional to positive selected signal scores
  - currently supports only `normalize="sum"`

Current cadence semantics:

- `every="slow"` means the rebalance agent is evaluated on every slow cycle
- `every="1d"` means the rebalance agent is evaluated at most once per 24 hours
- `every="weekly"` means the rebalance agent is evaluated at most once per 7 rolling days
- the current 24-hour throttle is based on the last rebalance evaluation timestamp, not the last fill timestamp
- the current 7-day throttle is also based on the last rebalance evaluation timestamp, not the last fill timestamp
- if a rebalance evaluation runs and produces no fills, `every="1d"` still delays the next rebalance evaluation for 24 hours
- if a rebalance evaluation runs and produces no fills, `every="weekly"` still delays the next rebalance evaluation for 7 days

Current boundary:

- this path still depends on a finite compile-time `Universe`
- the runtime does not continuously rebuild the `Universe` on every bar
- only the restricted allocation helpers listed above are supported
- portfolio target generation exists, but advanced portfolio constraints are not yet part of formal QuantScript
- there is no user-defined weighting function, arbitrary comparator, or arbitrary target-weight map DSL in formal QuantScript yet

Current backend portfolio-risk support:

- the backend runtime now supports portfolio-target clamping fields on risk policy objects:
  - `max_single_weight`
  - `max_turnover`
  - `min_trade_weight`
  - `max_new_positions_per_rebalance`
- these constraints are applied against backend `PortfolioTargetDecision` objects before execution computes the final order basket
- current behavior is conservative:
  - `max_single_weight` clamps each target weight independently
  - `max_turnover` scales portfolio deltas toward current weights
  - `min_trade_weight` removes small deltas by snapping them back to current weights
  - `max_new_positions_per_rebalance` keeps only the highest-priority new entries and zeros the rest
- formal QuantScript does not yet provide direct syntax for these fields
- at the moment, these constraints are configurable only through backend/runtime risk policy objects

### Compile-time universe snapshot requirement

Snapshot-backed universe selection is not self-contained in the script source.

If a script uses:

- `universe(...)`
- `filter(...)` with snapshot-backed metadata filters
- `sort_by(..., key="market_cap")`
- `sort_by(..., key="volume_24h")`
- `sort_by(..., key="listing_age_days")`

then the formal compile request must provide `universe_snapshot`.

Without `universe_snapshot`, lowering fails with a structured compile error.

Current point-in-time metadata contract:

- `universe_snapshot.as_of_ms` is the metadata selection timestamp used during lowering
- if an asset provides `metadata_history`, lowering uses the latest entry with `entry.as_of_ms <= universe_snapshot.as_of_ms`
- if an asset provides `listed_at_ms`, the asset is treated as ineligible before that time even if later metadata exists
- if no eligible history point exists, lowering falls back to flat top-level fields when available
- this improves point-in-time correctness for compile-time universe selection, but it does not yet provide runtime-dynamic reselection

### Builtin math and series reducers

- `abs`
- `avg`
- `first`
- `last`
- `max`
- `mean`
- `min`
- `pow`
- `sqrt`
- `std`
- `stddev`
- `sum`
- `variance`

These can appear either as free functions or member-style calls when meaningful:

```qs
mean(closes[20..])
closes[20..].mean()
first(closes)
closes.last()
```

### Indicator helpers recognized by resolve/lowering

- `sma`
- `ema`
- `rsi`
- `macd`
- `momentum`
- `zscore`
- `z_score`

### Change helpers and smoothing aliases

Gain-like helpers:

- `gains`
- `gain`
- `up_moves`
- `positive_changes`
- `positive_deltas`

Loss-like helpers:

- `losses`
- `loss`
- `down_moves`
- `negative_changes`
- `negative_deltas`

Smoothing aliases:

- `rma`
- `wilders`
- `smma`

### Imported transform helpers recognized today

- `field`
- `resample`
- `align`
- `align_asof`
- `spread`

These are especially relevant to the current restricted spread / quote-observe lowering path.

### Member-style helpers folded by the evaluator

When used on compatible values, the evaluator can fold:

- `.len()`
- `.sum()`
- `.mean()`
- `.avg()`
- `.min()`
- `.max()`
- `.std()`
- `.stddev()`
- `.variance()`
- `.first()`
- `.last()`
- `.ok()`
- `.retryable()`

And mutable list construction may use:

- `.push(...)`

Notes:

- `.ok()` and `.retryable()` are currently helper/evaluator conveniences around fetch-like expressions.
- formal QuantScript compile now rejects `.ok()` / `.retryable()` helper conveniences in executable strategy code with `QS0610`.
- Their presence in parsed code does not mean the language has a complete result/error type system.
- `.push(...)` is also a helper/evaluator convenience only; formal QuantScript compile now rejects it in executable strategy code with `QS0609`.

## Lowering-friendly patterns

The current runtime path is strongest when you write one of these families:

- direct moving average crossover with `sma(...)` / `ema(...)`
- manual moving average windows such as `closes[20..].sum() / 20`
- RSI via `rsi(...)`
- MACD via `macd(...)` or recognized manual EMA formulas
- momentum via `momentum(...)` or recognized manual formulas
- z-score via `zscore(...)` or recognized manual formulas
- the narrow admitted spread helper form `align_asof(...) + spread(..., output="bps") + one-sided >/>=`; broader spread formulas and helper-derived arithmetic remain rejected
- restricted compile-time-expanded multi-symbol strategies using `symbols(...)` or `universe(...)` plus `for s in selected`
- restricted portfolio rebalance using the supported allocation helpers plus `every="slow"`, `every="1d"`, or `every="weekly"`

Examples:

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

```qs
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let score = momentum(closes, 14)

    if score > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
```

```qs
fn strategy() {
    let base = universe(exchange="binance", market="spot", quote="USDT")
    let selected = top(sort_by(base, key="market_cap", order="desc"), 2)

    for s in selected {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        } else {
            emit Intent("SELL", instrument=s, quantity=1.0)
        }
    }
}
```

This example is supported only when the compile request includes a matching `universe_snapshot`.

```qs
fn strategy() {
    let base = symbols(["BTCUSDT", "ETHUSDT"])
    rebalance(equal_weight(base), every="1d")

    for s in base {
        let closes = fetch(s, interval="1d", lookback=200)?
        let fast = sma(closes, 20)
        let slow = sma(closes, 100)

        if fast > slow {
            emit Intent("BUY", instrument=s, quantity=1.0)
        }
    }
}
```

This example currently lowers to:

- compile-time-expanded per-symbol `fetch(...)` and `emit Intent(...)`
- a backend `PortfolioRebalance` agent policy
- an explicit rebalance target symbol set carried into runtime
- runtime equal-weight target computation across the selected symbols
- runtime cadence gating according to backend `RebalanceSchedule`

## Diagnostics-relevant semantic rules

The syntax parser is permissive compared with the executable contract.
Current semantic analysis additionally checks for rules such as:

- unresolved names
- duplicate functions
- non-boolean conditions
- look-ahead risk from invalid history access
- insufficient warmup when `lookback` is smaller than required series history
- universe helpers that require snapshot metadata but were compiled without `universe_snapshot`
- unsupported universe sort keys or sort orders during lowering
- unsupported snapshot-backed filter keys during lowering
- unsupported `rebalance(..., every=...)` values during lowering
- unsupported `rebalance(...)` allocation forms other than the restricted helper set above
- direct recursive helper calls in executable strategy code
- non-`Universe` `for` loops in executable strategy code
- `every="1d"` throttles rebalance evaluation by elapsed runtime time; it is not tied to exchange sessions or calendar-day boundaries
- `every="weekly"` throttles rebalance evaluation by elapsed runtime time; it is not tied to calendar-week boundaries

Current warmup rule:

- only an explicit `lookback=` on `fetch(...)` / `get_data(...)` counts
- there is no hidden default warmup assumption in semantic analysis

## Important unsupported or unstable areas

The following are either unsupported, parse-only, or not part of the stable executable contract:

- arbitrary top-level statements outside imports/functions
- standalone reassignment statements like `x = y`
- block-style match arms
- a formal destructuring pattern system
- recursion
- general-purpose async strategy execution
- arbitrary host-code execution
- a complete user-defined state model
- a complete exception / result / trait / class / module system
- runtime-dynamic universe refresh or per-bar top-N reselection
- general portfolio/basket semantics beyond the current restricted allocation-helper rebalance path
- arbitrary user-defined collection transforms, lambdas, or custom comparators over `Universe`
- custom portfolio weighting functions, arbitrary target-weight maps, or user-defined rank/score DSL
- dynamic portfolio rebalance driven by runtime-refreshed `Universe` membership
- runtime-dynamic or per-bar universe reselection; current point-in-time metadata still applies only at the single compile-time `universe_snapshot.as_of_ms`
- sector caps, turnover controls, or other advanced portfolio policy DSL directly in formal QuantScript
- calendar-aware rebalance schedules beyond the current `slow` / rolling-24h / rolling-7d forms

Several of these constructs are intentionally listed here as non-trunk areas, not as a backlog for future expansion.
Use the trunk baseline guide when deciding whether a missing capability should enter the language or should instead live in profiles, custom nodes, modules, or tooling.

## Practical recommendations

To stay inside the stable path:

- define exactly one `fn strategy()`
- use `fetch(...)` or `get_data(...)` explicitly with `lookback=...`
- keep bindings simple and name intermediate series clearly
- prefer recognized helpers such as `sma`, `ema`, `rsi`, `macd`, `momentum`, `zscore`
- if you use multi-symbol selection, keep it to `symbols(...)` / `universe(...)` + `filter(...)` + `sort_by(...)` + `top(...)`
- when using metadata-backed universe selection, pass a `universe_snapshot` whose `as_of_ms` matches the backtest or run selection time you want
- use `listed_at_ms` or `metadata_history` in `universe_snapshot` when you need point-in-time-correct ranking/filtering instead of a single flat latest snapshot
- if you use portfolio rebalance, keep it to the restricted allocation helpers plus `every="slow"`, `every="1d"`, or `every="weekly"`
- use `every="slow"` when you want rebalance on each slow cycle
- use `every="1d"` when a rolling 24-hour throttle is acceptable
- use `every="weekly"` when a rolling 7-day throttle is acceptable
- provide `universe_snapshot` in the formal compile request whenever metadata-backed universe selection is used
- use `for` only for restricted `Universe` iteration such as `for s in selected`
- do not rely on `match`, recursion, or generic collection loops in executable formal QuantScript
- do not rely on plain `import foo as bar` or arbitrary postfix `?` as if formal QuantScript supported a complete module/error system
- do not rely on `.ok()`, `.retryable()`, or `.push(...)` as if they were stable formal language features
- prefer double-quoted strings
- keep identifiers ASCII

## Minimal example

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
