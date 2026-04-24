# Spread Formal QuantScript Admission Contract

This document defines the first honest admission contract for spread in formal QuantScript.

That contract is now the landed formal QuantScript surface for the first spread slice.
It does not widen the product boundary beyond the existing graph/runtime and Strategy IR slice.

It is downstream from:

- [Spread Minimal Contract](./guide-spread-minimal-contract.md)
- [Spread Graph Runtime Minimal Design](./guide-spread-graph-runtime-minimal-design.md)
- [Spread Strategy IR Minimal Contract](./guide-spread-strategy-ir-minimal-contract.md)

## Purpose

graph/runtime and Strategy IR now already admit the same narrow spread slice:

- exactly two inputs
- explicit as-of alignment
- positive tolerance
- `bps` output
- one-sided `>` / `>=` threshold

formal QuantScript is now the third adopter for the same narrow slice.

This document freezes the only helper shape that is admitted.

## Minimal admission target

formal QuantScript admits only this narrow form:

```qs
let left_aligned = align_asof(left, direction="backward", tolerance_ms=1000)
let right_aligned = align_asof(right, direction="backward", tolerance_ms=1000)
let s = spread(left_aligned, right_aligned, output="bps")
if s > 5 {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
```

Or the equivalent inline form:

```qs
if spread(
    align_asof(left, direction="backward", tolerance_ms=1000),
    align_asof(right, direction="backward", tolerance_ms=1000),
    output="bps"
) > 5 {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
```

Both forms must mean exactly the same thing as the existing graph/runtime and Strategy IR slice.

## Admission requirements

formal QuantScript should admit spread only if all of the following remain true:

1. The helper still means exactly two inputs.
2. `output="bps"` remains the only admitted product output for the first slice.
3. the aligned spread inputs are built explicitly through `align_asof(...)`, and that alignment remains bounded by `tolerance_ms`.
4. The threshold remains one-sided `>` or `>=`.
5. The resulting lowering path still produces the same Core IR shape already used by graph/runtime and Strategy IR.
6. The happy path does not depend on widening matcher recovery beyond the current spread helper boundary.

If any of these stop being true, this landed admission should be reconsidered instead of widened silently.

## Explicit helper shape

The first admitted formal QuantScript helper surface must stay narrow:

- outer comparison target: `spread(...)`
- positional arguments to `spread(...)`: exactly two aligned source expressions
- required keyword arguments to `spread(...)`:
  - `output="bps"`
- required alignment wrapper on both inputs:
  - `align_asof(target, direction="backward", tolerance_ms=<positive integer>)`
- admitted comparison:
  - `spread(...) > <number>`
  - `spread(...) >= <number>`
- admitted action side:
  - the current landed helper path still flows through one-sided conditional `emit Intent(...)` and lowers to the existing `QuoteObserve` runtime intent shape without widening the spread contract

The first admission must not imply:

- dual-sided long/short spread logic
- `<` / `<=` sell-style spread admission
- ratio or absolute output
- spread line-vs-line compare
- custom spread arithmetic
- more than two inputs

## Time alignment policy

formal QuantScript must not hide alignment policy behind vague helper defaults when this contract is admitted.

For the first slice:

- `align_asof(...)` must stay explicit in contract examples and tests
- `tolerance_ms` must be present and positive on both aligned operands
- missing counterpart points within tolerance still mean the spread sample is absent
- no silent zero-fill
- no silent carry-forward beyond tolerance

If implementation convenience uses internal defaults, docs and tests must still describe the explicit product rule.

## Output policy

The first formal QuantScript spread admission includes only:

- `output="bps"`

The following remain out of scope:

- `output="ratio"`
- `output="absolute"`

This mirrors the current graph/runtime and Strategy IR contract exactly.

## Lowering target

The first admitted formal QuantScript spread slice must lower into the same shared-core shape already used by the other two landed entry points:

- `CoreIndicatorKind::Spread`
- `SpreadSpec { left, right, align, output=bps, ... }`
- `ScalarExpr::Compare`

That compare must remain structurally equivalent to:

```text
spread_ref > threshold
```

or

```text
spread_ref >= threshold
```

If formal QuantScript cannot lower to that same shape honestly, it should not be admitted yet.

## Rejection requirements

formal QuantScript currently rejects:

- spread helpers with fewer or more than two inputs
- omitted or non-positive `tolerance_ms` on either aligned operand
- any `output` other than `bps`
- non-one-sided threshold shapes
- malformed helper calls that would otherwise fall back to broad matcher recovery

The first admitted formal path should prefer explicit structured diagnostics over generic helper argument failures.

## Current implementation truth

Current code status:

- graph/runtime already admits the narrow `bps` one-sided threshold slice
- Strategy IR already admits the same narrow slice
- graph/runtime and Strategy IR already have a cross-entry equivalence guardrail for that slice
- formal QuantScript now admits the same narrow spread slice as the third landed entry point
- admitted formal spread helpers now lower into the same structured spread compare shape already used by graph/runtime and Strategy IR
- malformed or non-admitted spread helper shapes in the formal path still collapse to the existing structured `QPQSLOW001` contract rather than a dedicated spread admission contract
- those formal rejection paths now also have golden-like API response-shape coverage for the explicit non-`bps`, missing `align_asof(...)`, non-positive `tolerance_ms`, and non-one-sided threshold cases

So this contract is now a landed capability boundary, not a future-only plan.

## Landed implementation order

The landed sequence was:

1. keep the helper shape exactly isomorphic to the existing graph/runtime and Strategy IR slice
2. add explicit admission and rejection rules for the narrow helper form
3. lower only that admitted helper form into the same structured spread compare
4. add three-entry equivalence tests across formal QuantScript, graph/runtime, and Strategy IR
5. keep `ratio`, `absolute`, and non-one-sided spread forms out of the admitted surface

## Usage rule

Do not describe formal QuantScript spread as a broad spread language feature.

It should currently be described only as:

- supporting the same narrow two-input `align_asof(...) + spread(..., output="bps") + one-sided >/>=` slice already landed in graph/runtime and Strategy IR
- rejecting broader spread helper shapes through the existing structured `QPQSLOW001` contract
