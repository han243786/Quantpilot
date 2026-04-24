# MACD Minimal Shared-Core Feasibility Review

This document evaluates whether `MACD` is ready to be promoted into the same
shared Core IR slice that already covers:

- direct moving-average compare
- one-sided `RSI`
- one-sided `momentum`
- one-sided `zscore`
- `Strategy IR` direct `MaCross`

The review is intentionally narrow. It only checks the two smallest remaining
candidate slices:

1. `histogram sign`
2. `line vs signal`

The goal is to decide whether either slice is stable enough to become a
cross-entry shared-core contract across:

- formal QuantScript
- graph/runtime compile
- Strategy IR

## Review result

Current conclusion: **do not promote `MACD` into shared-core yet**.

The strongest candidate is `histogram sign`, but it is still not admitted.
`line vs signal` is clearly not admitted yet.

## Entry-point matrix

### Candidate A: `histogram sign`

Target semantic:

- `macd_histogram > 0`
- `macd_histogram < 0`

#### Formal QuantScript

- Status: `partial`
- Pass:
  - `macd(...)` is a real supported indicator family in the formal path.
  - indicator input validation already treats `macd` as a first-class helper.
- Fail:
  - the current lowering path still depends on manual formula recovery and
    `fallback` handling for MACD-specific shapes.
  - the shared Core IR path is not exposed as a stable histogram-sign contract.
- Verdict: `not admitted`

#### Graph/runtime compile

- Status: `fail`
- Pass:
  - `builtin.intent.macd` exists as a runtime intent kind.
  - `CoreIndicatorKind::Macd` already exists.
- Fail:
  - there is no `lower_runtime_intent_condition(...)` branch that lowers MACD
    into a structured `ScalarExpr::Compare`.
  - graph config currently does not carry a stable bridge contract equivalent
    to the existing one-sided `RSI/momentum/zscore` compare metadata.
- Verdict: `not admitted`

#### Strategy IR

- Status: `fail`
- Pass:
  - `IndicatorKind::Macd` exists and lowers as an indicator kind.
- Fail:
  - there is no narrow structured condition path for MACD in
    `lower_strategy_logic_condition(...)`.
  - Strategy IR currently has no stable, minimal contract for “histogram sign”
    that avoids inventing a broader condition language.
- Verdict: `not admitted`

#### Shared-core admission result

- QuantScript: `fail`
- graph/runtime: `fail`
- Strategy IR: `fail`
- Decision: `defer`

### Candidate B: `line vs signal`

Target semantic:

- `macd_line > signal_line`
- `macd_line < signal_line`

#### Formal QuantScript

- Status: `partial`
- Pass:
  - there is already explicit MACD matcher/recovery logic for line/signal
    relationships in lowering.
  - `match_macd_line_signal_pair` proves the repo recognizes this semantic
    family today.
- Fail:
  - the current implementation is still matcher-driven and remains explicitly
    parked in lowering compatibility logic.
  - this is not yet a resolve-first or runtime-stable contract.
- Verdict: `not admitted`

#### Graph/runtime compile

- Status: `fail`
- Pass:
  - `IntentKind::Macd` exists and compiles as an indicator kind.
- Fail:
  - runtime intent params do not currently encode an explicit `line vs signal`
    compare contract.
  - `lower_runtime_intent_condition(...)` does not lower MACD into structured
    compare output.
- Verdict: `not admitted`

#### Strategy IR

- Status: `fail`
- Pass:
  - `IndicatorKind::Macd` exists.
- Fail:
  - Strategy IR has no stable narrow condition contract for
    `macd_line > signal_line`.
  - implementing this now would force either a broader parser or a matcher-like
    string recovery path, which would conflict with the current development
    direction.
- Verdict: `not admitted`

#### Shared-core admission result

- QuantScript: `fail`
- graph/runtime: `fail`
- Strategy IR: `fail`
- Decision: `defer`

## Why MACD is still deferred

`MACD` is not blocked by missing indicator enums. It is blocked by contract
stability.

The current codebase shows that:

- runtime/compiler knows `Macd` as an indicator kind
- formal QuantScript can recognize MACD-related shapes
- lowering still relies on MACD-specific matcher/recovery paths
- graph/runtime and Strategy IR do not yet expose a single stable condition
  contract that maps naturally onto the same Core IR predicate shape

That means `MACD` has shared business meaning, but not yet shared executable
contract.

## Admission checklist

Either candidate may be promoted later only if all items below become `pass`.

1. QuantScript no longer depends on a MACD-specific local matcher as the main
   semantic path.
2. graph/runtime compile can express the same slice through stable intent
   params, not ad hoc bridge guessing.
3. Strategy IR can express the same slice through a narrow contract without
   inventing a general expression language.
4. compiler can lower the slice into one stable Core IR predicate shape.
5. dual-sided merging does not force a dishonest structured condition.
6. compiler unit tests exist.
7. graph/runtime API tests exist.
8. Strategy IR API tests exist.

## Recommended next action

Do not implement `MACD` shared-core alignment yet.

If MACD work is resumed, the next step should be a narrower follow-up review:

1. decide whether `histogram sign` is the only admissible first slice
2. confirm whether QuantScript can move that slice off matcher-first recovery
3. only then consider graph/runtime and Strategy IR alignment
