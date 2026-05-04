# Completed Functional Closeout Ledger

This file is the completed ledger for the functional closeout phase.
It is intentionally preserved as a record of the fixing, closing, and polishing
work already landed against the existing product surface.

It is no longer an active backlog.
Do not append new task lanes here.
Current baseline go/no-go and owner-only release blockers live in
[First Release Readiness](../../implementation/planning/implementation-first-release-readiness.md).

## Phase rules

- no new feature lanes
- no support-matrix expansion
- no new product narrative
- no second truth source beside backend capability and runtime facts
- no widening of QuantScript beyond the retained `V1` trunk

## Completed objective

The closeout phase aligned the existing beta surface across:

- frontend exposure
- backend validation
- runtime behavior
- backtest artifacts
- diagnostics and explanation
- docs and tests

## Scoring baseline behind this ledger

- functional progress is already high enough that the main work is closure, not invention
- repository stability is now primarily blocked by integration reliability and release hygiene
- release readiness is still blocked by test orchestration and owner decisions, not by a lack of breadth

## Completed execution order

1. close false-green or false-red gate gaps
2. align all exposed behavior with the retained support boundary
3. harden explanation, persistence, and replay on the existing protocol surface
4. compress stale docs and keep only one live planning entrypoint

## Historical task shape

Each completed task was executed as one reviewable batch with:

- `ID`
- `Priority`
- `Goal`
- `Scope`
- `Actions`
- `Acceptance`

## P0

Status: completed.

### CL-P0-001 E2E closed-loop repair

- `ID`: `CL-P0-001`
- `Priority`: `P0`
- `Goal`:
  Make the documented frontend E2E gate represent a real runnable product loop.
- `Scope`:
  `frontend/playwright.config.js`, E2E harness setup, backend startup contract,
  any supporting scripts or README instructions.
- `Actions`:
  Decide on one test mode only:
  either start the backend automatically for E2E, or fully intercept the API
  surface so no request leaks to the Vite proxy.
  Remove the current mixed state where the frontend boots but `/api` still
  depends on a missing backend.
  Re-run the existing Playwright suite without manual pre-start steps.
- `Acceptance`:
  `cmd /c npm run test:e2e` passes from the documented repo flow.
  No E2E case fails with `ECONNREFUSED 127.0.0.1:3000`.
  README startup and CI wording match the actual E2E contract.

### CL-P0-002 Gate command normalization

- `ID`: `CL-P0-002`
- `Priority`: `P0`
- `Goal`:
  Make local and CI gate commands behave the same on Windows.
- `Scope`:
  README gate commands, frontend command examples, CI commands, local helper
  scripts.
- `Actions`:
  Normalize commands that currently depend on PowerShell execution policy or
  shell-specific wrappers.
  Prefer command forms that work from a clean Windows shell without hidden
  machine setup.
  Keep command wording consistent across docs and CI.
- `Acceptance`:
  The documented commands are directly runnable in the repo environment.
  No frontend gate depends on `npm.ps1` being allowed by local execution
  policy.
  CI, README, and local helper scripts use the same Windows command forms.

### CL-P0-003 Baseline commit readiness

- `ID`: `CL-P0-003`
- `Priority`: `P0`
- `Goal`:
  Finish the repo-closeout prerequisites for a trustworthy baseline snapshot.
- `Scope`:
  repo hygiene docs, ignored artifacts, release-readiness notes.
- `Actions`:
  Confirm the first stable baseline checklist is explicit.
  Keep generated/runtime artifacts out of versioned product truth.
  Ensure the repo can be captured as a clean first milestone once gates are
  green.
- `Acceptance`:
  No planning doc still implies that the repo is in open-ended expansion mode.
  The remaining blockers to a baseline commit are explicit and short.
  Baseline go/no-go and owner-only actions are readable from one release-readiness doc.

## P1

Status: completed.

### CL-P1-001 Capability-driven frontend exposure audit

- `ID`: `CL-P1-001`
- `Priority`: `P1`
- `Goal`:
  Ensure all visible frontend actions follow backend capability truth exactly.
- `Scope`:
  toolbar actions, module sidebar, workspace entry actions, fallback and
  degraded-capability UI states.
- `Actions`:
  Walk the current exposed actions one by one:
  compile, simulation, backtest, template load, experiment entry, version
  history, collaboration metadata.
  Verify each action is enabled, disabled, hidden, or explained solely from the
  current capability contract and retained beta boundary.
  Remove any stale optimistic exposure.
- `Acceptance`:
  No visible action suggests support that the backend rejects as unsupported.
  Capability fallback, cache fallback, and safe fallback modes are covered by
  stable tests.

### CL-P1-002 Compile-chain contract tightening

- `ID`: `CL-P1-002`
- `Priority`: `P1`
- `Goal`:
  Keep compile output and diagnostics on one fixed interpretation path.
- `Scope`:
  compile routes, frontend compile state mapping, compile summary panels,
  diagnostics source labels, related tests and docs.
- `Actions`:
  Re-verify the fixed order:
  `strategy_ir` semantic preflight, optional formal QuantScript lowering, then
  runtime compile as source of runnable truth.
  Remove duplicate or ambiguous wording in UI and docs.
  Add or refresh regression coverage for disagreement and fallback cases.
- `Acceptance`:
  Compile summary, stored artifacts, and docs all describe the same source of
  truth.
  Diagnostics do not blur `strategy_ir`, `formal_quantscript`, and `runtime`.
  Active wording is centralized in
  `markdown/implementation/governance/implementation-compile-chain-contract.md`.
  Property-panel conflict guidance does not keep a separate inline wording
  copy.

### CL-P1-003 Runtime and backtest explanation unification

- `ID`: `CL-P1-003`
- `Priority`: `P1`
- `Goal`:
  Keep run detail, backtest detail, event stream, and diagnostics panel on the
  same explanation facts.
- `Scope`:
  `runtime_diagnostics`, event projection, detail DTOs, event stream cards,
  backtest detail sections.
- `Actions`:
  Audit the current explanation surfaces for risk, execution, fill lifecycle,
  and data quality.
  Remove any remaining locally invented interpretation path if it does not come
  from runtime facts or the structured diagnostics projection.
  Keep the event log and persisted detail views aligned.
- `Acceptance`:
  The same run/backtest fact is rendered consistently in diagnostics, event
  history, and detail pages.
  No second explanation DTO family is introduced.
  Active wording is centralized in
  `markdown/implementation/runtime/implementation-runtime-backtest-explanation-contract.md`.

### CL-P1-004 Persistence and replay consistency sweep

- `ID`: `CL-P1-004`
- `Priority`: `P1`
- `Goal`:
  Make persisted artifacts reload exactly as the active UI expects.
- `Scope`:
  graph versions, run history, backtest history, replay checkpoints, experiment
  records, storage-backed detail loading.
- `Actions`:
  Check that list views, detail views, compare views, and replay views all read
  stable persisted shapes instead of depending on transient in-memory state.
  Fix any drift between live response mapping and persisted reload mapping.
- `Acceptance`:
  Reloaded data renders without special-case reconstruction.
  Persisted detail pages and history cards agree on identifiers, sequence, and
  explanation rows.
  Active wording is centralized in
  `markdown/implementation/runtime/implementation-persistence-replay-contract.md`.

### CL-P1-005 QuantScript retained-surface hardening

- `ID`: `CL-P1-005`
- `Priority`: `P1`
- `Goal`:
  Make the retained formal QuantScript trunk stricter and more honest.
- `Scope`:
  parser vs lowering boundary, formal lowering diagnostics, authoring fixtures,
  syntax docs.
- `Actions`:
  Identify current cases that still parse broadly but should fail earlier under
  the retained `V1` path.
  Prefer explicit rejection over partial implied support.
  Keep fixtures and guides focused on the real admitted trunk only.
- `Acceptance`:
  Supported authoring samples compile consistently.
  Unsupported constructs fail early with stable diagnostics.
  Docs do not imply a broader research language than the code actually supports.
  Active wording is centralized in
  `markdown/implementation/governance/implementation-quantscript-retained-surface-contract.md`.

## P2

Status: completed.

### CL-P2-001 High-value smoke matrix

- `ID`: `CL-P2-001`
- `Priority`: `P2`
- `Goal`:
  Lock the existing product surface with a small but trustworthy smoke matrix.
- `Scope`:
  compile, simulation, backtest, detail reload, version restore, template load,
  capability fallback, collaboration metadata.
- `Actions`:
  Define one canonical smoke path per major existing feature.
  Prefer broad path coverage over more variations.
  Ensure each smoke case proves an existing user-facing promise.
- `Acceptance`:
  The smoke set is short, stable, and covers the retained beta promises.
  New fixes extend this set only when they close an existing gap.

### CL-P2-002 Documentation compression

- `ID`: `CL-P2-002`
- `Priority`: `P2`
- `Goal`:
  Keep only one active planning entrypoint for the current closeout phase.
- `Scope`:
  planning indexes, docs root fast paths, overview index, archived planning
  notes.
- `Actions`:
  Move obsolete phase plans out of the active planning folder.
  Point active indexes at this task table, release readiness, and the retained
  boundary docs.
  Remove links that still advertise abandoned feature-expansion planning as
  current work.
- `Acceptance`:
  A new contributor can find the current execution plan from one planning
  README.
  Historical plans remain archived, not active.

### CL-P2-003 Release wording honesty pass

- `ID`: `CL-P2-003`
- `Priority`: `P2`
- `Goal`:
  Make external wording no stronger than the implemented beta boundary.
- `Scope`:
  README, overview roadmap, support matrix, first-release readiness notes,
  any remaining frontend explanatory text.
- `Actions`:
  Re-check claims around exchange support, symbol support, paper-only mode,
  backtest semantics, plugin surface, and QuantScript breadth.
  Remove vague aspirational wording where it reads like current capability.
- `Acceptance`:
  README, docs, and product copy all describe the same limited beta truth.
  No statement reads like a promise for an unimplemented lane.

## Gate set

Canonical Windows wrapper:

```powershell
.\tools\run-closeout-gates.bat
```

Run these checks after each meaningful batch:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1
cargo test --workspace
cd frontend; cmd /c npm run test
cd frontend; cmd /c npm run build
cd frontend; cmd /c npm run test:e2e
```

Gate contract for this phase:

- `cmd /c npm run test:e2e` is a blocking gate and must not require a manually started backend
- unmocked E2E API requests are treated as gate failures, not as implicit proxy traffic
- Windows docs and CI must keep using the same command forms shown above

## Exit condition for this phase

This closeout phase is considered complete when:

- the gate set is green without hidden manual environment steps
- exposed functionality matches documented support exactly
- persisted details, history, and replay stay on one protocol family
- planning docs no longer pull the team back into speculative expansion

Current status: complete as a functional closeout ledger.
Do not use this file to create new feature or cleanup scope.
