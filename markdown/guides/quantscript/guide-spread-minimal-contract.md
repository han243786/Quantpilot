# Spread Minimal Contract

This document defines the first honest product contract for `spread(...)`.

It does not define a general arbitrage DSL.
It does not authorize broad multi-source indicator growth.
It only freezes the narrowest spread semantics that are worth carrying forward into a shared-core design.

## Purpose

The current codebase already has partial spread-related machinery:

- a frontend/runtime module anchor: `builtin.intent.spread_observer`
- compile-time fields such as `max_time_diff_ms`, `align_direction_code`, `field_code`, and `spread_output_code`
- runtime lowering that can already build a two-input `SpreadSpec`

What is still missing is an explicit product boundary.
This document is that boundary.

## Minimal product goal

The first spread contract is intentionally narrow:

- exactly two inputs
- explicit as-of time alignment
- a single observable spread output
- a single threshold-style shared-core condition shape

Anything broader stays out of scope until this slice is stable across:

- formal QuantScript
- graph/runtime compile
- Strategy IR

## First admitted slice

The first spread slice that is worth treating as shared-core eligible is:

- `two-input aligned spread output in bps`
- used in `one-sided threshold` form

Canonical example:

```text
spread(
  align_asof(left, direction="backward", tolerance_ms=1000),
  align_asof(right, direction="backward", tolerance_ms=1000),
  output="bps"
) > 5
```

Why this slice comes first:

- it already matches the current runtime-oriented `SpreadSpec` shape
- `bps` is easier to explain than arbitrary ratio math for the first productized contract
- one-sided threshold conditions fit the same current shared-core direction used by one-sided `RSI`, `momentum`, and `zscore`

The following are not the first slice:

- histogram-like spread transforms
- arbitrary spread arithmetic
- dual-sided merged spread rules
- three-leg or N-leg arbitrage expressions
- venue-routing policy encoded in the spread expression itself

## Time alignment policy

Spread semantics are only admissible when time alignment is explicit and bounded.

The minimal alignment policy is:

- join mode: `asof`
- required direction: one of `backward`, `forward`, `nearest`
- required tolerance: `max_time_diff_ms`
- if no counterpart observation exists within tolerance, the spread sample is absent

Product rule:

- do not silently treat unmatched timestamps as zero
- do not silently carry forward observations beyond the declared tolerance
- do not hide the alignment direction inside helper defaults when the product contract is being described

Current recommended first policy:

- default direction: `backward`
- explicit tolerance required in product examples and tests

This keeps the first contract honest and matches the current runtime-facing `AlignAsofSpec` shape.

## Output policy

The runtime lowering path already recognizes multiple output encodings, but the product contract should not expose all of them equally on day one.

Current output vocabulary:

- `absolute`
- `ratio`
- `bps`

The first admitted shared-core slice is:

- `bps`

The other outputs remain implementation-adjacent, but not yet first-class shared-core commitments.

Rationale:

- `bps` is the easiest first unit for cross-market or cross-source divergence
- it avoids overcommitting to ratio normalization semantics before the contract is frozen

## Cross-entry contract

The first spread slice is only considered real product capability when the same semantic shape can be expressed honestly across all three entry points.

### Formal QuantScript

Formal QuantScript now admits the same narrow helper form for the first slice:

```qs
if spread(
    align_asof(left, direction="backward", tolerance_ms=1000),
    align_asof(right, direction="backward", tolerance_ms=1000),
    output="bps"
) > 5 {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
```

### Graph/runtime compile

Graph/runtime compile is the current anchor for the spread contract because it already carries:

- two explicit inputs
- `align_direction_code`
- `max_time_diff_ms`
- `spread_output_code`

For the first slice, graph/runtime compile should remain the source of truth for the executable shape.

### Strategy IR

Strategy IR should only admit the same narrow spread slice.

It should not invent a broader condition language for spread before the product contract is stable.

## Shared-core admission rule for spread

Spread should only move into the same shared-core lane as direct MA compare or one-sided `RSI` / `momentum` / `zscore` when all of the following are true:

1. The same two-input aligned `bps` spread can be expressed across formal QuantScript, graph/runtime compile, and Strategy IR.
2. The resulting condition lowers to one stable Core IR shape.
3. The contract no longer depends on ad hoc matcher recovery for its main happy path.
4. The alignment policy is fixed and tested.
5. Cross-entry equivalence tests can lock the same semantic condition shape.

Until then, spread remains a guarded mid-term item rather than a near-term shared-core promise.

## Explicitly out of scope

The first spread contract does not include:

- dynamic venue selection
- best-leg search across more than two sources
- cross-exchange routing policy
- generic arbitrage strategy DSL
- custom spread math functions
- user-defined spread comparators
- multi-window spread state machines
- implied support for `MACD`-like spread derivatives

## Current implementation truth

Current code status should be described honestly:

- graph/runtime compile already has a concrete two-input spread runtime shape
- the first graph/runtime-only slice is now landed for `bps` plus one-sided threshold, and it lowers to a structured compare instead of stopping at raw condition text
- that graph/runtime slice now also rejects non-`bps` output, non-positive tolerance, and non-one-sided threshold metadata as explicit compile-contract failures
- compiler/runtime code can already build `SpreadSpec`
- Strategy IR now also admits the same narrow two-input `bps` one-sided threshold slice and lowers it into a structured compare with explicit rejection guardrails
- graph/runtime and Strategy IR now also have a cross-entry equivalence guardrail for that same narrow slice, so the two landed entry points must keep lowering to the same Core IR condition shape
- formal QuantScript is now the third landed adopter for the same narrow spread slice
- formal QuantScript now lowers that admitted helper form into the same structured spread compare used by graph/runtime and Strategy IR
- graph/runtime, Strategy IR, and formal QuantScript now have a three-entry equivalence guardrail for that same narrow slice
- malformed spread helper shapes already surface as structured `QPQSLOW001` instead of leaking generic helper errors
- formal QuantScript spread rejection paths now also have golden-like API response-shape coverage for non-`bps`, missing `align_asof(...)`, non-positive `tolerance_ms`, and non-one-sided threshold shapes

That means the first narrow spread lane is now landed across all three entry points, but spread still is not a general arbitrage DSL or a broad multi-shape shared-core feature.

For the current roadmap, `P3` can now be treated as phase-complete for this first narrow spread slice.

## Next implementation order

When work resumes on spread, the order should be:

1. keep this contract narrow
2. implement only the two-input `bps` one-sided threshold slice
3. make graph/runtime compile the executable truth source for that slice
4. add matching narrow entry support in Strategy IR
5. keep graph/runtime vs Strategy IR equivalence tests in place
6. only after that evaluate whether `ratio` or `line-vs-line` style spread conditions deserve expansion

## Usage rule

Do not describe spread as a general arbitrage language feature.

Do not expose broader spread claims in UI, prompts, or docs until the first narrow slice is stable across all three entry points.
