# Spread Formal QuantScript Minimal Implementation Plan

This document turns the admitted spread helper boundary into the smallest honest implementation plan for formal QuantScript.

It does not widen the contract.
It records the implementation that landed for the already narrowed helper form:

- `align_asof(...)` on both inputs
- `spread(..., output="bps")`
- one-sided `>` / `>=` threshold

It is downstream from:

- [Spread Minimal Contract](./guide-spread-minimal-contract.md)
- [Spread Strategy IR Minimal Contract](./guide-spread-strategy-ir-minimal-contract.md)
- [Spread Formal QuantScript Admission Contract](./guide-spread-formal-admission-contract.md)

## Implementation goal

Make formal QuantScript the third adopter of the already landed spread slice without changing the helper language.

The target executable shape is:

```qs
if spread(
    align_asof(left, direction="backward", tolerance_ms=1000),
    align_asof(right, direction="backward", tolerance_ms=1000),
    output="bps"
) > 5 {
    emit Intent("OBSERVE", instrument="BTCUSDT")
}
```

And the target lowered shape is the same one already used by graph/runtime and Strategy IR:

- `CoreIndicatorKind::Spread`
- `SpreadSpec`
- `ScalarExpr::Compare`

## Non-goals

Do not implement any of the following in this step:

- new spread helper syntax
- `spread(..., align=..., tolerance_ms=...)` direct argument admission
- `output="ratio"`
- `output="absolute"`
- `<` / `<=` sell-style spread threshold
- dual-sided spread rules
- spread line-vs-line compare
- custom spread arithmetic
- three-input or N-input spread

## Smallest code-change set

The smallest honest implementation should happen in three stages.

### Stage 1. Explicit admission gate

Owner:

- `quantscript/src/lowering/intents.rs`

Goal:

- keep using the existing spread helper surface
- reject everything outside the admitted slice before it can look like supported product capability

Required checks:

1. spread expression must come from `match_explicit_spread_call(...)`
2. exactly two inputs
3. both operands must already carry explicit `align_asof(...)` metadata
4. both operands must agree on:
   - `align_direction_code`
   - `tolerance_ms`
5. `tolerance_ms` must be positive
6. `output` must resolve to `SpreadOutputKind::Bps`
7. relation must be `>` or `>=`
8. threshold must be numeric
9. action side must stay within the admitted one-sided observe path

Implementation note:

- do not relax the existing parser or helper decoding logic
- do not add new fallback matchers
- do not keep the broader ratio/absolute spread shapes on the admitted formal path

### Stage 2. Structured compare bridge

Owner:

- `quantscript/src/lowering/intents.rs`

Goal:

- once the narrow spread slice is admitted, carry the same threshold-compare bridge metadata already used by graph/runtime and Strategy IR

Required params on the resulting runtime intent:

- `spread_output_code = 1`
- `comparison_shape_code = 1`
- `comparison_op_code = 2 or 3`
- `comparison_threshold = <number>`

Keep:

- `IntentKind::QuoteObserve`
- the existing `SpreadSpec`-shaped runtime params such as
  - `align_direction_code`
  - `max_time_diff_ms`
  - field/window/resample params

Do not:

- rely on `spread_trigger_bps` alone for the admitted slice
- claim structured compare admission if the compare bridge params are missing

### Stage 3. Three-entry equivalence guardrail

Owner:

- `src/main.rs`

Goal:

- prove that formal QuantScript now lowers the same admitted spread slice into the same Core IR condition shape already used by graph/runtime and Strategy IR

Add:

1. one formal success test for the admitted helper form
2. one formal rejection test for a non-admitted shape
3. one cross-entry equivalence test comparing:
   - formal QuantScript
   - graph/runtime compile
   - Strategy IR

The equivalence view should keep the same normalization rule already used for other shared-core slices:

- allow naming differences such as ref names or data ids
- do not hide condition-shape differences

## Landed implementation order

The landed implementation followed this order:

1. add the explicit admission gate
2. add rejection tests for non-admitted spread shapes
3. add structured compare bridge params on the admitted path
4. add one formal success test that proves structured compare lowering
5. add the three-entry equivalence guardrail
6. only after all of the above, update roadmap wording from "planned admission contract" to "landed third adopter"

## Recommended first rejection set

The first rejection set should stay small and directly aligned with the contract:

1. `output="ratio"`
2. missing `align_asof(...)` on one or both operands
3. non-positive `tolerance_ms`
4. `<` or `<=`
5. malformed helper shape that still currently falls into `QPQSLOW001`

Do not invent a new family of formal spread diagnostics unless the current structured lowering contract truly needs one.

## Files likely to change

Expected product-code changes:

- `quantscript/src/lowering/intents.rs`
- possibly `src/main.rs` for formal compile endpoint tests only

Expected documentation changes after the code lands:

- `markdown/guides/quantscript/guide-spread-formal-admission-contract.md`
- `markdown/guides/quantscript/guide-spread-minimal-contract.md`
- `markdown/guides/quantscript/guide-quantscript-first-lowering-split-patch-plan.md`
- `markdown/overview/overview-current-status-and-roadmap.md`
- `markdown/guides/quantscript/guide-formal-quantscript-syntax.md`

## Acceptance rule

This plan is complete only when all of the following are true:

1. formal QuantScript admits only the exact narrow helper surface described in the admission contract
2. formal QuantScript rejects non-admitted spread shapes explicitly
3. the admitted formal path lowers to the same structured spread compare used by graph/runtime and Strategy IR
4. a three-entry equivalence guardrail is green
5. docs are updated to say that formal QuantScript is now the third adopter of the narrow spread slice

Those conditions are now satisfied for the narrow formal spread slice.
