# Spread Graph Runtime Minimal Design

This document defines the first implementation design for spread on the graph/runtime compile path.

It is downstream from [Spread Minimal Contract](./guide-spread-minimal-contract.md).
It does not broaden that contract.
It only turns the first admitted slice into an executable graph/runtime design target.

As of the current code state, the first graph/runtime-only slice described here is now implemented for the narrow `bps` plus one-sided threshold path.
This document therefore serves both as the design boundary and as the current landed-scope description for that first slice.

## Goal

The first executable spread slice on the graph/runtime path is:

- exactly two inputs
- quote/series spread aligned with explicit `asof` policy
- `bps` output only
- one-sided threshold condition only

Canonical target:

```text
spread_bps(left, right, align=backward, tolerance_ms=1000) > 5
```

This is still narrower than a general spread strategy language.

## Why graph/runtime goes first

Graph/runtime compile is already the most concrete spread anchor in the codebase:

- `builtin.intent.spread_observer` already exists
- graph config already carries `max_time_diff_ms`
- graph config already carries `align_direction_code`
- graph config already carries `spread_output_code`
- compiler/runtime already knows how to build a two-input `SpreadSpec`

That makes graph/runtime the honest first executable source of truth for the spread slice.

Formal QuantScript and Strategy IR should follow this shape later instead of inventing their own wider variants first.

## Current implementation truth

Today the graph/runtime path is only partially aligned with the desired product shape:

- graph compile already maps `builtin.intent.spread_observer` into `IntentKind::QuoteObserve`
- Core IR lowering can already emit `CoreIndicatorKind::Spread` plus `SpreadSpec`
- `SpreadSpec` already carries:
  - two series inputs
  - `AlignAsofSpec`
  - output kind
  - optional resample/window configuration

But current product wording is still honest that the backend only exposes `QuoteObserve` semantics and not yet a fully productized spread strategy lane.

So this design is no longer purely prospective.
The narrow graph/runtime-only slice is now implemented, but the whole spread lane is still not complete.

## Minimal executable shape

The first graph/runtime implementation should accept only this narrow shape:

- module key: `builtin.intent.spread_observer`
- exactly two `input_refs`
- `spread_output_code = bps`
- `align_direction_code` in `{ backward, forward, nearest }`
- `max_time_diff_ms > 0`
- one-sided threshold comparison

The threshold itself should be represented the same way the current graph/runtime path represents one-sided `RSI`, `momentum`, and `zscore`:

- `comparison_shape_code`
- `comparison_op_code`
- `comparison_threshold`

For the first slice:

- admitted `comparison_shape_code`: `buy`
- admitted operators: `>` or `>=`
- admitted threshold unit: `bps`

That gives one stable executable meaning:

- "go long / observe-positive when aligned spread in bps exceeds threshold"

## Config contract

The first graph/runtime config contract should be:

- `max_time_diff_ms`
- `align_direction_code`
- `spread_output_code`
- `comparison_shape_code`
- `comparison_op_code`
- `comparison_threshold`

The first design should not add new graph-only spread fields unless strictly required.

The current spread module already has other fields such as:

- `field_code`
- `resample_period_ms`
- `resample_agg_code`
- `window_size`
- `window_agg_code`

These should remain implementation-adjacent for now, not product-frontier claims.

For the first executable slice:

- keep `field_code` on its current default path
- do not expand resample/window semantics into the first user-facing spread contract

## Time alignment policy

The graph/runtime implementation must make the time alignment policy explicit.

The first version should enforce:

- join mode: `asof`
- direction from `align_direction_code`
- tolerance from `max_time_diff_ms`

Behavior rules:

- if no counterpart point exists within tolerance, no spread sample is produced
- there is no implicit zero fill
- there is no unbounded carry-forward

Recommended first defaults:

- `align_direction_code = backward`
- `max_time_diff_ms = 5000`

Those defaults already match the current graph/runtime spread-shaped config and are narrow enough to keep the first contract honest.

## Output policy

The first graph/runtime implementation should only admit:

- `spread_output_code = bps`

If other output codes remain accepted internally for compatibility, they should not be described as the first productized spread slice.

That means the implementation should clearly separate:

- compatibility surface
- product-admitted slice

## Core IR target

The first graph/runtime slice should lower into:

- `CoreIndicatorKind::Spread`
- `SpreadSpec { left, right, align, output=bps, ... }`
- a structured one-sided threshold `ScalarExpr::Compare`

The design target is:

- do not stop at `describe_runtime_intent_condition(...)`
- do not leave the first slice on raw condition text if the graph config already matches the admitted narrow shape

This is the key shared-core step for spread.

## Validation rules

The first graph/runtime implementation should reject the following at validation or compile time:

- fewer or more than two inputs
- `spread_output_code` not equal to `bps`
- missing or non-positive `max_time_diff_ms`
- unsupported `align_direction_code`
- missing threshold metadata
- non-buy/non-one-sided threshold shapes

The design should prefer explicit rejection over silently degrading into a broader `QuoteObserve` story.

Current landed compile guardrails:

- `QPSPREAD001` for non-`bps` output on the threshold slice
- `QPSPREAD002` for missing or non-positive `max_time_diff_ms`
- `QPSPREAD003` for non-one-sided or otherwise incomplete threshold metadata

## Capability and wording rule

Until the first slice is implemented and tested:

- keep `builtin.intent.spread_observer` described as partial / guarded capability
- do not advertise "real spread strategy support"

After the first slice is implemented:

- capability output and UI wording may describe only the narrow admitted slice
- they still must not claim a general spread DSL or arbitrage engine

## Required tests

The first graph/runtime implementation is not done until all of the following exist:

1. graph/runtime compile success test for two-input `bps` one-sided threshold
2. graph/runtime compile rejection test for non-`bps` output
3. graph/runtime compile rejection test for missing/invalid alignment tolerance
4. graph/runtime compile rejection test for non-one-sided threshold shape
5. Core IR assertion that the condition is structured compare, not only raw text

Cross-entry equivalence tests now exist between graph/runtime and Strategy IR for the same narrow `bps` threshold slice. Formal QuantScript still does not admit that slice, so no three-entry equivalence claim should be made yet.

## Deliberately not in this design

This first graph/runtime implementation does not include:

- formal QuantScript spread admission
- Strategy IR spread admission
- dual-sided spread thresholds
- spread line-vs-line compare
- ratio/absolute spread as productized first-class outputs
- venue policy
- multi-leg arbitrage
- generic spread scripting

## Next step after this design

Once this graph/runtime design is accepted, the next concrete implementation step should be:

1. keep the graph/runtime slice narrow and tested
2. keep the graph/runtime vs Strategy IR equivalence guardrail in place for that slice
3. only then decide whether formal QuantScript should be the next adopting entry point
