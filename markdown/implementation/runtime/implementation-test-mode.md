# Deterministic Test Mode

This document defines the deterministic test mode used by QuantPilot during Week 1 and beyond.

Its purpose is not to simulate reality perfectly.
Its purpose is to make compile, run, replay, backtest, and CI behavior repeatable enough to test the current beta safely.

## Role

Deterministic test mode exists to support:

- backend service-level API tests
- runtime replay and regression checks
- compile and run contract validation
- CI-safe backtest smoke coverage
- future frontend E2E flows that must not depend on live timing variance

It does not redefine the trading model.
It constrains execution so that the same inputs produce stable outputs under the same mode and seed.

## Non-goals

Deterministic test mode is not:

- a live trading environment
- a claim of real market fidelity
- a replacement for future accurate backtest mode
- a license to bypass declared capability gates

If a behavior only works in deterministic test mode but cannot be expressed through the same public runtime contracts, it should be treated as test-only scaffolding rather than product capability.

## Minimum guarantees

When deterministic test mode is enabled, QuantPilot should aim to guarantee:

- fixed seed or explicit no-randomness behavior
- fixed or simulated clock behavior
- stable event ordering
- no dependence on external network state
- no dependence on process-global mutable toggles
- repeatable replay from the same input bundle
- capability boundaries identical to normal beta mode

The key rule is simple:
test mode may simplify timing and execution conditions, but it must not invent unsupported product features.

## Required controls

The runtime entry point for test mode should expose these controls explicitly.

Current Week 1 implementation lives in:

- [qrpc_runtime/src/sandbox.rs](/D:/rust-js-pr/QuantPilot/quantpilot/qrpc_runtime/src/sandbox.rs)
- exported through [qrpc_runtime/src/lib.rs](/D:/rust-js-pr/QuantPilot/quantpilot/qrpc_runtime/src/lib.rs)

The current Rust surface is:

- `DeterministicTestMode`
- `DeterministicClockMode`
- `DeterministicEventOrdering`
- `DeterministicParallelismPolicy`
- `RuntimeSupportBoundary`

- `enabled`: whether deterministic test mode is active
- `seed`: reproducible pseudo-random seed when randomness exists
- `clock_mode`: wall clock or simulated clock
- `start_time`: fixed logical start timestamp when relevant
- `event_ordering`: deterministic ordering policy for queued events
- `parallelism_policy`: constrained threading or single-thread execution when required for reproducibility

These controls must not be hidden in environment variables or ad hoc globals.
Current Week 1 tests also lock the requirement that different sandbox instances can carry different test-mode configs without relying on any process-global switch.

## Where it applies

### Compile and capability validation

Deterministic test mode does not relax capability gating.

The same graph or script that is rejected in normal beta mode should still be rejected in test mode when it uses:

- unsupported indicators
- unsupported execution modules
- hidden or gated frontend modules
- undeclared runtime modes

### Service-level API tests

For `/api/compile`, `/api/run`, `/api/backtest`, and `/api/capabilities` tests:

- inputs should be fixed fixtures
- returned event order should be stable
- tests should assert structured fields rather than incidental formatting
- failures should remain attributable to contract drift, not timing noise

### Replay and regression

Replay is one of the main consumers of test mode.

To keep replay meaningful:

- the same input graph, market data fixture, and runtime config should reproduce the same event sequence shape
- fill behavior should remain stable under the same seed and mode
- snapshots and summaries should be comparable across runs

### Frontend E2E

When Week 2 introduces browser E2E coverage, frontend smoke tests should target deterministic backend behavior rather than a timing-sensitive live loop.

This matters especially for:

- compile button flows
- event stream assertions
- backtest summary rendering
- capability-driven module visibility

## Relationship to sandbox modes

Deterministic test mode is orthogonal to sandbox mode.

It can be used inside:

- `RealTimeSandbox` for repeatable runtime smoke coverage
- `FastBacktestSandbox` for CI-safe replay validation

Current Week 1 implementation stores the selected test mode inside sandbox instances and exposes it through `SandboxSnapshot`.

It should not be confused with the future `AccurateBacktestSandbox`.
Accurate simulation is about fidelity.
Deterministic test mode is about repeatability and contract confidence.

## Current beta boundary

At the current stage, deterministic test mode should remain aligned with the actual beta boundary:

- `paper` only
- limited exchange set
- limited symbol set
- capability-gated modules remain gated

Current Week 1 implementation also exports the runtime-aligned support boundary through `RuntimeSupportBoundary`, so `/api/capabilities` and runtime compile gating do not need to duplicate the runtime mode and execution-module truth separately.

This keeps test infrastructure honest.
The project should never use test mode to create the appearance of support that the product does not actually provide.

## Acceptance criteria

Week 1 documentation for deterministic test mode is complete when:

- the purpose and boundary of test mode are written down clearly
- test mode is explicitly distinguished from live trading and high-fidelity backtest
- replay, CI, service-level tests, and future E2E usage are described
- related implementation guides reference this document instead of redefining the same concept differently
