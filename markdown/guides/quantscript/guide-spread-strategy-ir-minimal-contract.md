# Spread Strategy IR Minimal Contract

This document defines the first honest `Strategy IR` contract for spread.

It is intentionally downstream from:

- [Spread Minimal Contract](./guide-spread-minimal-contract.md)
- [Spread Graph Runtime Minimal Design](./guide-spread-graph-runtime-minimal-design.md)

It does not define a broader spread language for `Strategy IR`.
It only permits the same narrow slice that graph/runtime compile already admits.

## Purpose

`Strategy IR` is the second entry point that should adopt spread.

It is a better second adopter than formal QuantScript because:

- it already has explicit indicator kinds and parameter fields
- it can mirror graph/runtime shape directly
- it does not require reopening a helper-driven text DSL before the shared-core slice is stable

The goal is simple:

- let `Strategy IR` express the exact same two-input `bps` one-sided threshold slice
- do not let `Strategy IR` invent a wider spread condition language

## Minimal admitted slice

The first admitted `Strategy IR` spread slice is exactly:

- `IndicatorKind::Spread`
- exactly two inputs
- `align_direction_code` support, with the current landed implementation inheriting graph/runtime's default `backward` direction when the code is omitted
- explicit positive `max_time_diff_ms`
- `spread_output_code = bps`
- one-sided threshold compare only

Canonical intended shape:

```text
spread_signal > 5
```

Where `spread_signal` is backed by:

- two spread inputs
- `bps` output
- explicit as-of alignment policy

The threshold remains numeric and one-sided.

## Strict isomorphism rule

This contract is only valid if it stays structurally isomorphic to graph/runtime compile.

That means `Strategy IR` must not admit spread features that graph/runtime does not already admit for this slice.

For the first landed contract, `Strategy IR` should match graph/runtime on:

- exactly two inputs
- `bps` output only
- the same current alignment-direction behavior as graph/runtime compile
- explicit positive tolerance
- one-sided `>` / `>=` threshold semantics

If graph/runtime rejects a shape, `Strategy IR` must reject it too.

## Inputs

The first contract requires:

- exactly two declared inputs
- no implicit third leg
- no dynamic venue expansion
- no best-leg search

The two inputs should be treated as the left and right spread operands in the same order as graph/runtime compile.

## Alignment policy

The first `Strategy IR` spread contract inherits the same time-alignment policy as graph/runtime:

- join mode: `asof`
- direction: `backward`, `forward`, or `nearest`
- tolerance: `max_time_diff_ms > 0`
- unmatched samples are absent

`Strategy IR` must not hide these semantics behind a new higher-level spread policy name.

If a future product version wants named alignment presets, that should be added later and only after the first narrow contract is stable.

## Output policy

The first `Strategy IR` spread contract admits only:

- `spread_output_code = bps`

The existence of runtime/compiler support for `ratio` or `absolute` does not make them part of the first admitted `Strategy IR` product contract.

## Condition policy

The first admitted condition shape is:

- one-sided threshold
- `spread_signal > threshold`
- or `spread_signal >= threshold`

This contract does not admit:

- dual-sided merged spread rules
- `<` / `<=` sell-style spread threshold for the first slice
- line-vs-line spread compare
- spread threshold with non-numeric right side

The point is not that these are impossible forever.
The point is that they are not the first stable shared-core slice.

## Required `Strategy IR` shape

The first `Strategy IR` spread contract should continue to use the existing structured surface:

- `IndicatorKind::Spread`
- `indicator.inputs = [left, right]`
- `indicator.params`

Required parameter family:

- `align_direction_code`
- `max_time_diff_ms`
- `spread_output_code`

Required logic rule family:

- a single one-sided threshold comparison over the spread signal id

This avoids inventing a second spread-specific mini-language inside `Strategy IR`.

## Explicitly not allowed

The first `Strategy IR` spread contract must reject:

- more than two inputs
- `spread_output_code != bps`
- missing or non-positive `max_time_diff_ms`
- unsupported `align_direction_code`
- spread condition shapes other than one-sided threshold
- custom spread transforms
- spread line/signal style comparisons
- dual-sided long/short spread logic
- generic arbitrage workflow encoded in `Strategy IR` logic text

## Current implementation truth

Current code truth should be described honestly:

- `Strategy IR` already has `IndicatorKind::Spread`
- compiler code can already derive a `SpreadSpec` from `Strategy IR`
- the first narrow `Strategy IR` spread threshold contract is now landed for the same graph/runtime-isomorphic slice:
  - exactly two inputs
  - `spread_output_code = bps`
  - positive `max_time_diff_ms`
  - one-sided `>` / `>=` threshold over the spread signal id
- invalid shapes now reject explicitly instead of silently falling back to raw condition text
- formal QuantScript still does not admit the same spread slice, so no three-entry shared-core claim should be made yet

## Acceptance rule

This contract is only ready to claim as landed when all of the following are true:

1. `Strategy IR` accepts only the graph/runtime-isomorphic spread slice
2. invalid shapes reject explicitly
3. the resulting Core IR condition is structured compare, not only raw text
4. graph/runtime and `Strategy IR` can be checked with a cross-entry equivalence test

The first four items are now true for the same narrow `bps` one-sided threshold slice.

## Next implementation order

Once work starts on `Strategy IR` spread, the order should be:

1. keep the narrow shape frozen
2. keep the explicit rejection tests green
3. keep graph/runtime vs `Strategy IR` equivalence tests in place
4. only after that decide whether formal QuantScript should become the third adopter

## Usage rule

Do not describe `Strategy IR` spread support as broader than graph/runtime spread support.

If the graph/runtime narrow slice is the executable truth, `Strategy IR` must remain isomorphic to that truth until a larger spread contract is explicitly designed and approved.
