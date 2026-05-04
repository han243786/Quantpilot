# QuantPilot Compile-Chain Contract

## Purpose

This document is the active closeout contract for compile interpretation.
Use it whenever compile UI, compile-summary wording, diagnostics routing, or
runtime artifact descriptions change.

It does not introduce a new compile lane.
It only locks the current beta chain into one explicit interpretation.

## Fixed order

QuantPilot compile must be read in exactly this order:

1. `strategy_ir` semantic preflight
2. optional `quantscript.formal_source` lowering
3. `/api/runtime/compile` as the runnable result authority

## Interpretation rules

- `strategy_ir` may fail early and should surface diagnostics honestly.
- `strategy_ir` never decides the final runnable output.
- `quantscript.formal_source` may provide the runtime-compile input when present.
- if formal lowering is unavailable, runtime compile may fall back to the
  graph-generated `runtime_config`.
- regardless of input path, the final runnable result always follows
  `/api/runtime/compile`.

## Required UI wording

The following meaning must stay consistent across property panels, workspace
summary cards, failure notices, tests, and docs:

- `Strategy IR role` means preflight role only
- `Runtime source` means which artifact fed runtime compile
- `Runnable truth` means the final result still follows
  `/api/runtime/compile`

When preflight passes but runtime compile fails, the UI must explain:

- preflight success does not imply runnable success
- the operator must repair the artifact that actually entered runtime compile
- structured diagnostics and the runtime-truth field should be read together
- that warning copy must come from one shared frontend contract source instead
  of per-panel inline wording

## Current accepted runtime-source labels

- `Formal QuantScript lowering 输入`
- `图生成的 runtime_config 输入`
- `图生成的 runtime_config 回退输入`

## Current accepted runnable-truth label

- `以 /api/runtime/compile 输出为准`

## Current accepted conflict guidance copy

- conflict warning message and hint are owned by
  `frontend/src/utils/compileContract.js`
- property panels, action guidance, and tests must reuse that wording instead
  of restating it locally
- action failure copy may add action-specific recovery steps, but runtime-truth
  wording must still reuse `COMPILE_CONTRACT.runtimeSourceOfTruthLabel`

## Shared copy-source inventory

- compile conflict truth: `frontend/src/utils/compileContract.js`
- action failure next steps: `frontend/src/utils/actionFailure.js`
- capability exposure and support labels:
  `frontend/src/capabilities/supportMatrix.js`
- capability overstatement gates:
  `frontend/src/capabilities/capabilityGovernance.js` and
  `tools/check-user-facing-text.ps1`

## Drift checks

Any compile-chain change must update all affected layers together:

- frontend compile summary projection
- workspace compile context cards
- compile failure guidance text
- compile-related tests
- support/governance docs when wording changed

## References

- [Support Matrix](./implementation-support-matrix.md)
- [Capability Governance](./implementation-capability-governance.md)
- [Current Status And Release State](../../overview/overview-current-status-and-roadmap.md)
- [Archived Functional Closeout Ledger](../../archive/planning-retired/implementation-functional-closeout-task-table.md)
