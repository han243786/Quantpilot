# QuantPilot Capability Governance

## Purpose

This document is the P1 governance layer for capability-related rules.
It sits on top of the P0 support matrix and turns the current beta boundary into an auditable maintenance policy.

Use this document when:

- changing `/api/capabilities`
- changing frontend module exposure
- changing capability-governed UI action availability
- changing workspace-surface visibility or its source-of-truth classification
- changing user-facing wording about supported or unsupported capability
- deciding whether a capability should stay visible, stay locked, or disappear

This document does not expand product scope.
It only governs how existing capability boundaries are classified, owned, reviewed, and retired.

Machine-readable companion:

- [frontend/src/capabilities/capabilityGovernance.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/capabilities/capabilityGovernance.js)
- [implementation-capability-governance-registry.generated.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/governance/implementation-capability-governance-registry.generated.md)

## Source of truth chain

Capability governance follows this chain:

1. backend `/api/capabilities`
2. [implementation-support-matrix.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/governance/implementation-support-matrix.md)
3. frontend support matrix and capability gates
4. README, UI prompts, tests, and acceptance checks

If these layers disagree, backend `/api/capabilities` is authoritative and the other layers must be updated.

## Capability classes

Every capability must belong to exactly one of the following classes.

### `supported`

- Safe to describe as currently supported within the stated beta boundary.
- Must have backend support, frontend exposure rules, and regression coverage.
- May appear in README and UI as a normal supported path.

Current examples:

- `paper` runtime mode
- `builtin.execution.paper`
- `binance`, `okx`
- `BTCUSDT`, `ETHUSDT`, `SOLUSDT`
- current K-line driven intent modules
- version history and collaboration/audit workspace surfaces

### `restricted`

- Exists and may be usable in compile or runtime paths, but only within a clearly limited boundary.
- Must always carry boundary notes.
- Must not be marketed as broader platform support than is actually implemented.

Current examples:

- restricted `Custom` Strategy IR expression path
- Strategy IR semantic preflight
- formal QuantScript lowering when present
- spread-related lowering paths that exist only as limited beta compile/runtime behavior

### `trace_only`

- Exists in code or artifacts for beta compatibility, internal experiments, or transition state.
- May stay visible in code, fixtures, or artifact structures.
- Must not be used as evidence of supported product scope.

Current examples:

- arbitrage-related module keys that exist in the beta compile path
- legacy compatibility fields kept for capability response continuity

### `disallowed_claim`

- Must never appear as a positive support claim in user-facing material.
- Tests and wording gates should catch these claims.

Current examples:

- research-grade backtest support
- live trading support
- true arbitrage agent support
- third-party plugin marketplace support

## Capability registry and owners

Ownership is assigned by role, not by person name.
Each capability family must have one primary owner role.

| Capability family | Class | Owner role | Review responsibility |
|---|---|---|---|
| runtime modes | supported / restricted | backend runtime owner | backend contract, compile/runtime checks |
| execution modules | supported / restricted | backend runtime owner | execution semantics, capability response |
| exchanges and symbols | supported / restricted | backend market-data owner | market boundary, fixtures, wording |
| Strategy IR indicator kinds | supported / restricted | backend compile owner | lowering boundary, diagnostics |
| frontend module exposure | supported / trace_only | frontend editor owner | sidebar exposure, disabled reasons, UX |
| capability-governed UI actions | supported / restricted | frontend editor owner | action gating, reason text, E2E |
| workspace surfaces | supported / restricted | frontend editor owner | workspace exposure, backend route honesty, closeout audit |
| public wording | all classes | docs and QA owner | README, markdown, UI copy, text gates |

## Change policy

Any change to a capability must update all affected layers in the same batch.

### Required update checklist

- backend capability response or compatibility fixture
- frontend support matrix or capability gate logic
- tests that prove visibility, disabled state, or action routing
- user-facing wording when support claims or restrictions changed
- support matrix documentation

### Required review questions

- does this change expand, shrink, or only clarify the current boundary?
- does the frontend still avoid fake entry points?
- are visible workspace cards classified against the correct visibility source instead of being silently treated as capability-driven?
- does safe fallback remain more restrictive than normal mode?
- do README and UI wording still match backend truth?
- does E2E prove the user cannot reach a blocked backend path through normal interaction?

## Retirement and convergence policy

Capability lifecycle is intentionally conservative.

### When to move `trace_only` to `restricted`

- the capability has a real backend contract
- boundary notes are explicit
- tests prove the path is intentionally limited rather than accidental

### When to move `restricted` to `supported`

- backend and frontend semantics are stable
- wording can describe the capability without caveats that materially change user expectations
- regression coverage exists at the contract and UI layer

### When to remove a capability from user-facing surfaces

- backend no longer returns it as usable
- capability cannot be defended as a real supported or restricted path
- keeping it visible would create a fake entry point or support claim

## Drift prevention rules

- Never add a new frontend module card without updating the support matrix.
- Never add a new capability response field without deciding its class and owner role.
- Never add a new visible workspace surface without classifying whether it is capability-driven, local-only, or persistence-driven.
- Never add a new positive support statement without checking it against the `allowed_claim` whitelist and `disallowed_claim` set.
- Never rely on code presence alone as evidence of product support.

## Governance evidence

The following artifacts count as governance evidence:

- support matrix doc updates
- capability fixture updates
- frontend capability tests
- Playwright capability-path tests
- user-facing wording gate results

## References

- [implementation-support-matrix.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/governance/implementation-support-matrix.md)
- [implementation-compile-chain-contract.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/governance/implementation-compile-chain-contract.md)
- [implementation-functional-closeout-task-table.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/planning/implementation-functional-closeout-task-table.md)
- [overview-current-status-and-roadmap.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/overview/overview-current-status-and-roadmap.md)

