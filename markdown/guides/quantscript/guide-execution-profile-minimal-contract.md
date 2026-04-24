# QuantScript `execution.profile(...)` Minimal Contract

## Purpose

This document defines the first minimal contract for moving execution
configuration out of QuantScript trunk syntax and into a profile-shaped
boundary.

The goal is not to introduce a general-purpose execution DSL.
The goal is to give QuantScript, graph runtime compile, and Strategy IR a
small, honest, shared way to point at the existing paper execution module
without pushing more execution semantics into the trunk language.

## Current Runtime Anchor

Today the supported runtime execution module is still:

- `builtin.execution.paper`

The already-real module fields exposed by the first contract are:

- `fee_bps`
- `slippage_bps`

## Minimal Contract

### Shape

The first contract is a single builtin helper:

```qs
execution.profile("paper")
```

or:

```qs
execution.profile("paper", fee_bps=10.0, slippage_bps=5.0)
```

Current implementation note:

- in the formal QuantScript path, `execution.profile(...)` must currently
  appear as a single top-level statement inside `fn strategy()`
- do not split this call across multiple lines until the generic statement
  parser is widened honestly

### Required positional argument

- `profile_id: string`

The first release only allows:

- `"paper"`

### Allowed keyword fields

- `fee_bps`
  - float
  - must be `>= 0`
  - maps one-to-one to runtime `taker_fee_bps`
  - projects back to frontend execution node config `fee_bps`
- `slippage_bps`
  - float
  - must be `>= 0`
  - maps one-to-one to runtime `default_slippage_bps`
  - projects back to frontend execution node config `slippage_bps`

### Defaults

If fields are omitted, the contract falls back to the same defaults used by the
current graph/runtime compile path for `builtin.execution.paper`:

- `fee_bps = 10.0`
- `slippage_bps = 5.0`

This default is now shared across:

- formal QuantScript lowering
- graph runtime compile
- Strategy IR lowering

## Semantics

The first `execution.profile(...)` contract does **not** introduce a new
execution engine.
It only selects and parameterizes the existing paper execution module.

That means:

- no inline execution expressions
- no broker routing language
- no venue-switching policy
- no per-order execution overrides from QuantScript trunk
- no custom execution plugin selection

The compile result still lowers to the existing runtime execution shape and the
existing frontend/runtime module key:

- `builtin.execution.paper`

## Out Of Scope

The first contract must not include:

- `mode`
- `order_type`
- `time_in_force`
- `slippage_model`
- `latency_assumption_ms`
- `capital_base`
- custom execution module selection
- broker-specific execution behavior

These may be revisited later, but they are not part of the first minimal
profile contract.

For the next `P4` backtest slice, this document should be read together with
[Backtest Execution-Assumptions Minimal Contract](./guide-backtest-execution-assumptions-minimal-contract.md).

That planned split is:

- `execution.profile(...)` owns reusable strategy defaults
- the backtest request owns run-scoped overrides

`latency_ms` should remain request-scoped for the first backtest-assumptions
slice instead of entering trunk QuantScript syntax.

## Cross-Entry Alignment Rule

This contract is only worth implementing if the same semantic shape can be
expressed across:

- formal QuantScript
- graph runtime compile
- Strategy IR

For the first step, that shared semantic shape is simply:

- one paper execution profile
- one optional slippage setting
- one-to-one lowering into `builtin.execution.paper`

## Suggested Diagnostics

When implementation begins, diagnostics should stay narrow and product-facing:

- unsupported `profile_id`
- unsupported keyword field
- non-numeric `fee_bps`
- negative `fee_bps`
- non-numeric `slippage_bps`
- negative `slippage_bps`
- duplicate execution profile declaration

Do not leak low-level helper or parser error strings when these become stable
product contracts.

## Current Status

The first compile-path implementation is now landed across:

- formal QuantScript
- graph runtime compile
- Strategy IR

Current boundary:

- only `profile_id="paper"` is supported
- only `fee_bps` and `slippage_bps` are supported
- the lowering target remains exactly `builtin.execution.paper`
- this contract does not replace the broader `execution` block in Strategy IR;
  it provides a narrower profile-shaped path for the current runtime
- capability output has not been widened for profile-specific reporting yet; the
  current landed scope is compile/runtime lowering plus cross-entry tests
- `latency_ms` remains intentionally outside `execution.profile(...)` and
  belongs to the backtest request layer for the first `P4` slice
