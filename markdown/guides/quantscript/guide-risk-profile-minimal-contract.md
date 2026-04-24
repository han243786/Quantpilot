# QuantScript `risk.profile(...)` Minimal Contract

## Purpose

This document defines the first minimal contract for moving risk configuration
out of QuantScript trunk syntax and into a profile-shaped boundary.

The goal is not to introduce a general-purpose risk DSL.
The goal is to give QuantScript, graph runtime compile, and Strategy IR a small,
honest, shared way to point at the existing global risk module without pushing
more risk semantics into the trunk language.

## Why This Exists

Current product direction is:

- keep QuantScript trunk focused on data, indicators, constrained universe
  flows, minimal control flow, and standardized `emit Intent(...)`
- move risk / execution / broker complexity outward
- avoid expanding main syntax into a second research language

So `risk.profile(...)` is only allowed if it stays:

- small
- explicit
- capability-gated
- one-to-one with existing runtime risk configuration

## Current Runtime Anchor

Today the supported runtime risk module is still:

- `builtin.risk.global`

The already-real runtime fields on that module are:

- `max_position`
- `max_total_leverage`
- `max_exchange_leverage`
- `min_action_interval_ms`

These are the only fields the first `risk.profile(...)` contract may expose.

## Minimal Contract

### Shape

The first contract should be a single builtin helper:

```qs
risk.profile("global")
```

or:

```qs
risk.profile("global", max_position=0.2, max_total_leverage=3.0, max_exchange_leverage=3.0, min_action_interval_ms=100)
```

Current implementation note:

- in the formal QuantScript path, `risk.profile(...)` must currently appear as a
  single top-level statement inside `fn strategy()`
- do not split this call across multiple lines until the generic statement
  parser is widened honestly

### Required positional argument

- `profile_id: string`

The first release only allows:

- `"global"`

This keeps the contract aligned with the existing `builtin.risk.global` module
instead of pretending there is already a profile marketplace or multiple risk
engines.

### Allowed keyword fields

- `max_position`
  - float
  - must be `> 0`
  - maps to runtime `RiskConfig.max_position_ratio`
- `max_total_leverage`
  - float
  - must be `>= 1`
  - maps to runtime `RiskConfig.max_total_leverage`
- `max_exchange_leverage`
  - float
  - must be `>= 1`
  - maps to runtime `RiskConfig.max_exchange_leverage`
- `min_action_interval_ms`
  - integer
  - must be `>= 0`
  - maps to runtime `RiskConfig.min_action_interval_ms`

### Defaults

If a field is omitted, the contract should fall back to the same defaults used
by the current graph/runtime compile path for `builtin.risk.global`:

- `max_position = 0.2`
- `max_total_leverage = 3.0`
- `max_exchange_leverage = 3.0`
- `min_action_interval_ms = 100`

These defaults are now shared across:

- formal QuantScript lowering
- graph runtime compile
- Strategy IR lowering

## Semantics

The first `risk.profile(...)` contract does **not** introduce a new risk engine.
It only selects and parameterizes the existing global risk module.

That means:

- no inline risk expressions
- no user-defined risk formulas
- no conditional risk branches
- no portfolio policy language
- no custom risk plugin selection from QuantScript trunk

The compile result should still lower to the existing runtime `RiskConfig`
shape and the existing frontend/runtime module key:

- `builtin.risk.global`

## Out Of Scope

The first contract must not include:

- `stop_loss_ratio`
- `take_profit_ratio`
- `max_drawdown_ratio`
- `max_trades_per_day`
- dynamic per-symbol overrides
- user-defined risk predicates
- cross-bar stateful risk scripting
- custom risk module selection
- broker-specific risk behavior

These may be revisited later, but they are not part of the first minimal
profile contract.

## Cross-Entry Alignment Rule

This contract is only worth implementing if the same semantic shape can be
expressed across:

- formal QuantScript
- graph runtime compile
- Strategy IR

For the first step, that shared semantic shape is simply:

- one global risk profile
- one small set of numeric limits
- one-to-one lowering into `builtin.risk.global`

If an addition cannot keep this one-to-one lowering, it should not be admitted
into the first contract.

## Suggested Diagnostics

When implementation begins, the first diagnostics should stay narrow and
product-facing:

- unsupported `profile_id`
- unsupported keyword field
- non-numeric field value
- out-of-range numeric field value
- duplicate risk profile declaration

Do not leak low-level helper or parser error strings when these become stable
product contracts.

## Implementation Order

1. Formalize this contract in docs.
2. Map it one-to-one to `builtin.risk.global`.
3. Add capability and compile-path support.
4. Add graph/runtime and Strategy IR alignment.
5. Add structured diagnostics and round-trip tests.

## Current Status

The first compile-path implementation is now landed across:

- formal QuantScript
- graph runtime compile
- Strategy IR

Current boundary:

- only `profile_id="global"` is supported
- only the four numeric fields above are supported
- the lowering target remains exactly `builtin.risk.global`
- this contract does not replace `risk_rules`; it provides a narrower
  profile-shaped path for the current runtime
- capability output has not been widened for profile-specific reporting yet; the
  current landed scope is compile/runtime lowering plus cross-entry tests

## Acceptance Rule

The first `risk.profile(...)` implementation is acceptable only if:

- it does not widen QuantScript trunk into a general risk DSL
- it lowers one-to-one into existing runtime risk config
- graph/runtime compile and Strategy IR can express the same contract honestly
- unsupported fields fail explicitly
- docs, tests, and capability output all agree on the same boundary
