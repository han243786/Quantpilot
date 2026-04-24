# Testing Module Implementation

This document focuses on fill simulation and testing infrastructure in QuantPilot.

Its role is not to create a separate backtesting world.
Its role is to strengthen the unified trading sandbox with reliable fill semantics, replay support, and regression coverage.

For the deterministic test mode boundary used by CI, replay, and E2E flows, see [implementation-test-mode.md](D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-mode.md).
For how to interpret targeted tests, the full gate wrapper, and the isolated E2E contract, see [implementation-test-layer-expectations.md](D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-layer-expectations.md).

## Current role

The testing module serves two purposes:

- provide stable fill behavior for simulation and backtest modes
- provide replay and audit foundations for regression and debugging

## Existing base

The repository already includes:

- `ExecutionPlan`
- `FillReport`
- `OpenOrder`
- `FillResult`
- `PortfolioState`
- `RuntimeEvent`
- independent `fill_engine.rs`
- tests for fill behavior and runtime integration

This means the task is no longer "invent fill protocol from scratch".
The task is now "turn the existing fill logic into a stable sandbox component".

## Near-term goals

1. keep fill semantics aligned with real-time simulation
2. support fast historical backtest
3. add replay and snapshot foundations
4. leave extension points for future accurate simulation

## Near-term tasks

### Task 1: stabilize fill engine I/O

Goal:

- keep one clear contract:
  `ExecutionPlan + MarketState -> FillResult`
- make the account update boundary easier to reason about

Acceptance:

- fixed inputs produce stable outputs under the same mode
- repeated submission does not double-book state

### Task 2: strengthen event payloads

Instead of large enum churn, extend current event payloads where needed.

Minimum expectations:

- `ExecutionPlanned` includes order state, remaining quantity, and limit information when relevant
- `ExecutionFilled` includes side, quantity, price, and execution status
- frontend can distinguish planned, waiting, partial, and complete states

### Task 3: replay and snapshot basics

Goal:

- support repeat execution and debugging from stable runtime outputs

Minimum outputs:

- event sequence export
- account state snapshot
- repeatable replay from fixed input

### Task 4: support fast backtest sandbox

Goal:

- work directly with the sandbox roadmap

Minimum support:

- K-line or L1-driven execution
- simplified matching model
- repeatable runs
- stable result output

### Task 5: encoding and user-facing text gates

Goal:

- prevent UTF-8 regressions and mojibake from re-entering frontend and docs

Minimum checks:

- reject UTF-8 BOM in frontend source and markdown docs
- reject replacement characters
- reject known mojibake fragments observed in prior regressions
- keep the checks runnable from local PowerShell and CI

Recommended scripts:

- `tools/check-utf8.ps1`
- `tools/check-user-facing-text.ps1`
- `tools/check-gates-smoke.ps1`

Recommended scan scope:

- `frontend/src`
- `frontend/index.html`
- `src/main.rs`
- `markdown`

Current Week 2 status:

- both gate scripts are wired into `.github/workflows/ci.yml`
- the gates now fail before frontend dependency install so mojibake regressions surface earlier
- `tools/check-gates-smoke.ps1` provides a minimal local regression sample that writes bad inputs and asserts both gates fail

Recommended local commands:

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-user-facing-text.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-gates-smoke.ps1`
- `cmd /c tools\run-closeout-gates.bat`

Windows gate normalization rule:

- frontend gates should use `cmd /c npm run ...` in docs, helper scripts, and CI
- do not rely on `npm.ps1` execution policy exemptions as part of the normal repo contract

### Task 6: test directory and naming contract

Goal:

- remove ambiguity before Week 2 adds more API and E2E coverage
- keep service-level, browser, fixture, and repo-quality tests in stable locations

Repository conventions:

- Rust unit tests stay close to the implementation module when they cover local logic only
- Rust service-level and integration-style API tests go under `tests/`
- shared Rust fixtures go under `tests/fixtures/`
- frontend E2E tests go under `frontend/tests/e2e/`
- frontend E2E fixtures go under `frontend/tests/fixtures/`
- repo-wide quality gates stay under `tools/`

Recommended file naming:

- Rust API tests use `api_*.rs`, for example `api_capabilities.rs`, `api_compile.rs`, `api_run.rs`, `api_backtest.rs`
- Rust protocol or fixture-heavy tests use `protocol_*.rs` or `replay_*.rs` when the focus is narrower than one endpoint
- Playwright specs use `*.spec.ts`, for example `editor-smoke.spec.ts`, `capabilities-gating.spec.ts`, `backtest-smoke.spec.ts`
- PowerShell quality gates use `check-*.ps1`

CI split:

- blocking checks: capability contract tests, UTF-8 checks, user-facing text checks, frontend build
- near-term blocking backend coverage: service-level tests for `/api/capabilities` and compile path smoke
- nightly or later-stage checks: heavier replay suites, larger backtest fixture suites, full browser matrices

The main rule is to name tests after the contract they protect, not the internal helper they happen to call today.

## Future extensions

### P1: statistical backtest behavior

- reproducible random seed support
- statistical slippage models
- explainable approximation rules

### P2: high-fidelity simulation

- L2/L3 support
- queue position
- latency model
- more realistic market impact behavior

