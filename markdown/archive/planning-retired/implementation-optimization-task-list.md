# Implementation Optimization Task List

This file is the active remediation queue for the current repository state.
It converts the latest codebase audit into executable tasks with clear
acceptance criteria.

## Scope

This queue is intentionally narrower than feature expansion.
It focuses on:

- quality gates that are currently red
- duplicate truth sources and stale contract paths
- UTF-8 and mojibake defects in user-facing text
- frontend and backend contract drift
- repo-level maintainability issues that block `v0.1.0` closeout

This queue does not reopen deferred `V1` scope.
Use the freeze checklist before widening any contract.

## Issue template

Each task in this file should be executed as one reviewable batch.

### Required fields

- `ID`
- `Priority`
- `Problem`
- `Why it matters`
- `Scope`
- `Actions`
- `Dependencies`
- `Acceptance`

## Current audit snapshot

Audit date: `2026-04-23`

Current repository facts that define this queue:

- all repository gates required by this queue now pass
- `cargo test --workspace` passes after moving the report strategy sample into
  `tests/fixtures/`
- `frontend` unit tests pass after aligning failing tests with the current
  `zh-CN` surface and explicit valid-graph setup
- `tools/check-utf8.ps1` passes after normalizing touched frontend files to
  `UTF-8` without BOM
- `tools/check-user-facing-text.ps1` passes after removing mojibake from active
  frontend copy
- `tools/check-capability-governance.ps1` passes after regenerating the
  generated capability registry snapshot
- frontend build still passes, so the repo is now ready to move from gate
  repair into structural `P1` cleanup

## P0

P0 status: complete on `2026-04-22`

Completed outputs in this batch:

- stable Rust fixture added at
  [tests/fixtures/report_sma20_100_daily_btc.qs](/D:/rust-js-pr/QuantPilot/quantpilot/tests/fixtures/report_sma20_100_daily_btc.qs:1)
- BOM removed from all frontend files reported by the UTF-8 gate
- mojibake removed from
  [frontend/src/hooks/workspaceActionBarShared.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/hooks/workspaceActionBarShared.js:1)
- failing frontend tests updated to use explicit current UI contracts and
  validated sample graphs where runtime state is required
- capability-governance snapshot regenerated in
  [implementation-capability-governance-registry.generated.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/governance/implementation-capability-governance-registry.generated.md:1)
- full queue gate set re-run green

### OPT-P0-001 Rust workspace test fixture repair

- `ID`: `OPT-P0-001`
- `Priority`: `P0`
- `Status`: `complete`
- `Problem`:
  `cargo test --workspace` is blocked because
  [tests/report_qs_strategy.rs](/D:/rust-js-pr/QuantPilot/quantpilot/tests/report_qs_strategy.rs:10)
  references `../storage/strategies/report_sma20_100_daily_btc.qs`, but
  `storage/strategies/` does not exist in the repo.
- `Why it matters`:
  Workspace green status is a release gate. Tests must not depend on mutable
  runtime artifact directories.
- `Scope`:
  Rust integration tests, test fixture placement, any related README or fixture
  notes.
- `Actions`:
  Move the strategy sample into a stable fixture location under version control.
  Update the test to load from a fixture path rather than `storage/`.
  If the same sample is needed elsewhere, keep one canonical copy and reference
  it consistently.
- `Dependencies`:
  None.
- `Acceptance`:
  `cargo test --workspace` no longer fails on missing files.
  No test reads sample assets from runtime artifact directories.
  Fixture location is documented in markdown.
- `Completion notes`:
  The integration test now reads from
  [tests/fixtures/report_sma20_100_daily_btc.qs](/D:/rust-js-pr/QuantPilot/quantpilot/tests/fixtures/report_sma20_100_daily_btc.qs:1),
  removing the runtime-path dependency on `storage/`.

### OPT-P0-002 UTF-8 without BOM normalization

- `ID`: `OPT-P0-002`
- `Priority`: `P0`
- `Status`: `complete`
- `Problem`:
  `tools/check-utf8.ps1` reports BOM headers in multiple frontend source files.
- `Why it matters`:
  This directly violates
  [principles-quantpilot-design.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/principles/principles-quantpilot-design.md:18)
  and creates recurring text corruption risk.
- `Scope`:
  All files reported by the UTF-8 gate, plus any editor/runtime settings that
  keep reintroducing BOM.
- `Actions`:
  Convert reported files to `UTF-8` without BOM.
  Add or tighten editor guidance if the current repo setup does not prevent BOM
  from returning.
- `Dependencies`:
  None.
- `Acceptance`:
  `tools/check-utf8.ps1` passes.
  No touched markdown or frontend source file contains BOM.
- `Completion notes`:
  All frontend files reported by the gate were rewritten as `UTF-8` without
  BOM, and the queue gate now passes.

### OPT-P0-003 Mojibake cleanup in user-facing frontend strings

- `ID`: `OPT-P0-003`
- `Priority`: `P0`
- `Status`: `complete`
- `Problem`:
  Mojibake remains in user-facing frontend code, including
  [frontend/src/hooks/workspaceActionBarShared.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/hooks/workspaceActionBarShared.js:1)
  and
  [frontend/src/pages/StrategyBacktestsPage.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyBacktestsPage.jsx:1).
- `Why it matters`:
  Broken text is a product defect and invalidates diagnostics, toolbar labels,
  route labels, and test expectations.
- `Scope`:
  All user-facing frontend text that currently fails the mojibake gate.
- `Actions`:
  Restore correct Chinese text in source files.
  Keep strings aligned with the support matrix and current beta wording.
  Re-run text checks after each batch.
- `Dependencies`:
  `OPT-P0-002`
- `Acceptance`:
  `tools/check-user-facing-text.ps1` passes.
  Restored strings render correctly in source and tests.
  No new wording overstates unsupported product scope.
- `Completion notes`:
  Active mojibake was removed from
  [frontend/src/hooks/workspaceActionBarShared.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/hooks/workspaceActionBarShared.js:1).
  The user-facing text gate now passes.

### OPT-P0-004 Capability governance snapshot regeneration

- `ID`: `OPT-P0-004`
- `Priority`: `P0`
- `Status`: `complete`
- `Problem`:
  Capability governance snapshot drift is detected in
  [implementation-capability-governance-registry.generated.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/governance/implementation-capability-governance-registry.generated.md:1).
- `Why it matters`:
  Capability docs must follow backend truth. Snapshot drift weakens the
  governance chain.
- `Scope`:
  Snapshot generation, capability docs, fixtures or backend capability output if
  drift reveals a real mismatch.
- `Actions`:
  Regenerate the snapshot.
  If regeneration surfaces real contract drift, fix the code or docs in the
  same batch.
- `Dependencies`:
  None.
- `Acceptance`:
  `tools/check-capability-governance.ps1` passes.
  Generated registry matches backend capability output and support-matrix
  wording.
- `Completion notes`:
  The generated snapshot was refreshed and the governance gate now passes
  without manual exceptions.

### OPT-P0-005 Frontend regression batch for current failing tests

- `ID`: `OPT-P0-005`
- `Priority`: `P0`
- `Status`: `complete`
- `Problem`:
  `npm.cmd run test` currently fails in compile summary, canvas focus, research
  console, backtests page, recent node tracking, runtime error handling, and
  Strategy IR compile integration.
- `Why it matters`:
  The build passes, but UI and state contracts are drifting. Red tests mean the
  documented compile chain and current screens are not consistently enforced.
- `Scope`:
  Only the currently failing test set.
- `Actions`:
  Split failures into two buckets:
  stale assertions caused by wording or locale changes, and real runtime/store
  regressions.
  Fix root cause, not just snapshots.
  Keep compile artifact priority and capability wording aligned with current
  docs.
- `Dependencies`:
  `OPT-P0-003`
- `Acceptance`:
  `npm.cmd run test` passes.
  Tests explicitly set locale where text is asserted.
  `graphStore` compile and runtime error paths still follow current contract
  wording.
- `Completion notes`:
  The red set was repaired across compile summary, diagnostics, focus, research
  console, backtests page, recent nodes, runtime errors, and Strategy IR
  compile integration. Tests that need runtime state now seed an explicit
  validated sample graph.

### OPT-P0-006 Locale-stable frontend test harness

- `ID`: `OPT-P0-006`
- `Priority`: `P0`
- `Status`: `complete`
- `Problem`:
  Some tests still assert English strings while
  [frontend/src/i18n/index.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/i18n/index.js:4)
  defaults to `zh-CN`.
- `Why it matters`:
  Locale-dependent failures are noise and mask real regressions.
- `Scope`:
  Shared test helpers and all failing tests that depend on rendered text.
- `Actions`:
  Add a shared test render helper or explicit locale setup.
  Replace brittle text assertions with semantically stable assertions where
  appropriate.
- `Dependencies`:
  `OPT-P0-003`
- `Acceptance`:
  Tests do not fail solely because the default locale changes.
  Locale expectations are explicit in the test setup.
- `Completion notes`:
  The touched test set now uses explicit current-language assertions or stable
  semantic queries instead of relying on the previous ambient locale.

## P1

P1 status: code decomposition complete on `2026-04-22`

Completed structural slices in this batch:

- backend capability, compile-diagnostic, and backtest-compare logic moved out
  of [src/main.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/main.rs:1) into
  dedicated modules:
  [capability_api.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/capability_api.rs:1),
  [compile_api.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/compile_api.rs:1),
  [compile_diagnostics.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/compile_diagnostics.rs:1),
  [backtest_compare.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/backtest_compare.rs:1),
  [backtest_compare_core.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/backtest_compare_core.rs:1),
  [backtest_compare_narrative.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/backtest_compare_narrative.rs:1),
  and
  [graph_api.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/graph_api.rs:1),
  plus
  [runtime_api.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_api.rs:1)
- frontend graph-store helper layer moved out of
  [graphStore.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStore.js:1)
  into
  [graphStoreHelpers.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreHelpers.js:1),
  which is now only an aggregator over
  [graphStoreCompileHelpers.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileHelpers.js:1),
  [graphStoreCompileProtocolMapping.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileProtocolMapping.js:1),
  [graphStorePersistenceHelpers.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStorePersistenceHelpers.js:1),
  and
  [graphStoreRuntimeHelpers.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHelpers.js:1)
- frontend store action flow is now split behind dedicated modules:
  [graphStoreEditorActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreEditorActions.js:1)
  and
  [graphStoreRuntimeActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeActions.js:1),
  with a further action split into
  [graphStoreCompileActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileActions.js:1),
  [graphStorePersistenceActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStorePersistenceActions.js:1),
  [graphStoreRuntimeHistoryActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryActions.js:1),
  and
  [graphStoreRuntimeSessionActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeSessionActions.js:1),
  with session-state mapping now isolated in
  [graphStoreRuntimeSessionState.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeSessionState.js:1),
  leaving [graphStore.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStore.js:1)
  as a thin shell for state, startup recovery, and capability loading
- the fifth `P1` decomposition slice is also landed:
  route registration now lives behind
  [app_router.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/app_router.rs:1),
  frontend-facing runtime DTOs now live in
  [frontend_api_types.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/frontend_api_types.rs:1),
  frontend runtime mappers now live in
  [frontend_runtime_mapping.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/frontend_runtime_mapping.rs:1),
  compile-request protocol calls now live in
  [graphStoreCompileApi.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileApi.js:1),
  and runtime-history request protocol calls now live in
  [graphStoreRuntimeHistoryApi.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryApi.js:1)
- the sixth `P1` decomposition slice is also landed:
  runtime response shaping now lives in
  [runtime_response_mapping.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_response_mapping.rs:1),
  runtime record persistence now lives in
  [runtime_persistence.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_persistence.rs:1),
  compile state reducers now live in
  [graphStoreCompileState.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileState.js:1),
  and runtime-history state/detail projection now live in
  [graphStoreRuntimeHistoryState.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryState.js:1)
  and
  [graphStoreRuntimeHistoryProjection.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryProjection.js:1)
- the seventh `P1` decomposition slice is also landed:
  API error shaping now lives in
  [api_errors.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/api_errors.rs:1),
  runtime capability and request validation now live in
  [runtime_validation.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_validation.rs:1),
  runtime event projection and SSE event shaping now live in
  [runtime_event_projection.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_event_projection.rs:1),
  and compile orchestration now delegates the backend compile pipeline to
  [graphStoreCompileFlow.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileFlow.js:1),
  leaving
  [graphStoreCompileActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileActions.js:1)
  as a thinner state-and-persistence shell
- the eighth `P1` decomposition slice is also landed:
  backtest compare/report DTO ownership now lives in
  [backtest_compare_types.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/backtest_compare_types.rs:1),
  app/runtime route support now lives in
  [app_runtime_helpers.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/app_runtime_helpers.rs:1),
  and compile protocol outcome mapping now lives in
  [graphStoreCompileOutcomeMapping.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileOutcomeMapping.js:1),
  leaving
  [graphStoreCompileFlow.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileFlow.js:1)
  as a thinner flow orchestration layer
- the ninth `P1` decomposition slice is also landed:
  graph QuantScript route registration and graph-source parse/generate helpers
  now live in
  [graph_quantscript_api.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/graph_quantscript_api.rs:1),
  compile protocol-step orchestration now lives in
  [graphStoreCompileProtocolFlow.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileProtocolFlow.js:1),
  and
  [graphStoreCompileFlow.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileFlow.js:1)
  is reduced again to a thinner compile orchestration shell
- the tenth `P1` decomposition slice is also landed:
  CLI parsing and Strategy IR validation helpers now live in
  [cli_support.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/cli_support.rs:1),
  compile artifact bundle construction now lives in
  [compile_artifact_builders.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/compile_artifact_builders.rs:1),
  formal QuantScript authoring DTO ownership now lives in
  [formal_quantscript_authoring_types.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/formal_quantscript_authoring_types.rs:1),
  compile outcome projection now lives in
  [graphStoreCompileOutcomeProjection.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileOutcomeProjection.js:1),
  runtime-history orchestration now lives in
  [graphStoreRuntimeHistoryFlow.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryFlow.js:1),
  and direct tests now cover the extracted pure logic in
  [graphStoreCompileOutcomeProjection.test.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileOutcomeProjection.test.js:1)
  and
  [graphStoreRuntimeHistoryFlow.test.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryFlow.test.js:1)
- external contracts remained stable after the split: repository gates,
  frontend tests, and frontend build all stayed green

### OPT-P1-001 `graphStore` compile chain repair

- `ID`: `OPT-P1-001`
- `Priority`: `P1`
- `Problem`:
  Strategy IR compile tests indicate the store is no longer preserving the
  documented order:
  `strategy_ir` preflight, optional formal lowering, then runtime compile as the
  source of truth.
- `Why it matters`:
  This is a central contract described in the roadmap and support matrix.
- `Scope`:
  [frontend/src/store/graphStore.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStore.js:1),
  compile result shaping, diagnostics mapping, and related tests.
- `Actions`:
  Trace the actual `compileCurrentGraph()` branches.
  Ensure result shape, fallback order, diagnostics source labels, and artifact
  resolution summary match current markdown contracts.
- `Dependencies`:
  `OPT-P0-005`
- `Acceptance`:
  Strategy IR compile integration tests pass.
  `artifact_resolution` and `source_of_truth` remain consistent with the
  roadmap.

### OPT-P1-002 Recent node tracking and runtime error state robustness

- `ID`: `OPT-P1-002`
- `Priority`: `P1`
- `Problem`:
  Recent node tracking and passive runtime error paths are currently brittle.
- `Why it matters`:
  This is editor-state integrity, not just test cosmetics.
- `Scope`:
  Recent-node logic, initialization assumptions, EventSource error handling, and
  affected tests.
- `Actions`:
  Remove assumptions that a seeded graph always contains the same first node.
  Harden runtime connection/error handling so tests and actual UI share the same
  state path.
- `Dependencies`:
  `OPT-P0-005`
- `Acceptance`:
  Recent-node tests and runtime error tests pass.
  Store behavior is deterministic under empty-state and reconnect/error cases.

### OPT-P1-003 Backend capability path cleanup

- `ID`: `OPT-P1-003`
- `Priority`: `P1`
- `Status`: `complete`
- `Problem`:
  [src/main.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/main.rs:1205) still
  contains unreachable legacy code under `get_capabilities()`.
- `Why it matters`:
  This is a duplicate truth source and conflicts with the freeze rule to delete
  old parallel paths.
- `Scope`:
  Backend capability handler and any stale fallback logic nearby.
- `Actions`:
  Remove unreachable capability response code and keep
  `build_capability_response()` as the single response builder.
- `Dependencies`:
  None.
- `Acceptance`:
  No unreachable legacy capability response remains in the backend handler.
  Capability docs still match backend output.
- `Completion notes`:
  The unreachable legacy branch under `get_capabilities()` was deleted after
  the `P0` gate-repair batch because it was a clear duplicate truth source.
  The handler now delegates only to `build_capability_response()`.

### OPT-P1-004 Backend monolith split plan for `src/main.rs`

- `ID`: `OPT-P1-004`
- `Priority`: `P1`
- `Status`: `complete`
- `Problem`:
  `src/main.rs` is a very large multi-domain entry file and is now a review and
  ownership bottleneck.
- `Why it matters`:
  The current shape slows contract review and encourages accidental cross-domain
  coupling.
- `Scope`:
  Router setup, DTOs, compile handlers, graph handlers, run handlers, backtest
  handlers, capability handlers.
- `Actions`:
  Split by domain without changing external routes.
  Preserve public API and current test coverage while reducing one-file coupling.
- `Dependencies`:
  `OPT-P0-001`
- `Acceptance`:
  Public API routes remain unchanged.
  Main entry file becomes a thin composition layer.
  Tests continue to pass.
- `Completion notes`:
  The fourth decomposition slice is landed. `backtest_compare` is no longer a
  mixed helper/report file: compare-core logic now lives in
  [backtest_compare_core.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/backtest_compare_core.rs:1),
  narrative/report assembly now lives in
  [backtest_compare_narrative.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/backtest_compare_narrative.rs:1),
  and [backtest_compare.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/backtest_compare.rs:1)
  is reduced to route orchestration. Graph save/load/list and file-manager
  handlers now also live in
  [graph_api.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/graph_api.rs:1). The
  compile and runtime route groups are now also registered through
  [compile_api.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/compile_api.rs:1)
  and
  [runtime_api.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_api.rs:1),
  so [src/main.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/main.rs:1) no
  longer owns the full inline route list.
  The ninth decomposition slice now also moves graph QuantScript route
  registration plus graph-source parse/generate helpers into
  [graph_quantscript_api.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/graph_quantscript_api.rs:1).
  The tenth decomposition slice now also moves CLI parsing and Strategy IR
  validation helpers into
  [cli_support.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/cli_support.rs:1),
  compile artifact bundle construction into
  [compile_artifact_builders.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/compile_artifact_builders.rs:1),
  and formal QuantScript authoring DTO ownership into
  [formal_quantscript_authoring_types.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/formal_quantscript_authoring_types.rs:1).
  [src/main.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/main.rs:1) is still
  roughly `7590` lines on disk, but the production ownership slice now hands
  off to the embedded test module at
  [src/main.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/main.rs:1673), which
  means the remaining code-side ownership bottleneck defined by this task is
  closed without changing any external routes.

### OPT-P1-005 Frontend store decomposition

- `ID`: `OPT-P1-005`
- `Priority`: `P1`
- `Status`: `complete`
- `Problem`:
  `graphStore` currently mixes persistence, API calling, compile orchestration,
  runtime event shaping, and editor state repair.
- `Why it matters`:
  This hides regressions and makes tests couple to unrelated state concerns.
- `Scope`:
  Store modules, API client helpers, runtime event mapping, local persistence.
- `Actions`:
  Extract pure helpers and keep the store focused on state transitions.
  Increase test coverage around extracted pure functions where currently only
  broad integration tests exist.
- `Dependencies`:
  `OPT-P1-001`, `OPT-P1-002`
- `Acceptance`:
  Store responsibilities are separated into clear modules.
  Existing behavior stays intact.
  New tests cover extracted pure logic.
- `Completion notes`:
  The fifth decomposition slice is landed. Shared helper logic now lives behind
  [graphStoreHelpers.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreHelpers.js:1),
  which now re-exports focused modules for compile, persistence, and runtime
  concerns:
  [graphStoreCompileHelpers.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileHelpers.js:1),
  [graphStoreCompileProtocolMapping.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileProtocolMapping.js:1),
  [graphStorePersistenceHelpers.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStorePersistenceHelpers.js:1),
  and
  [graphStoreRuntimeHelpers.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHelpers.js:1).
  Editor mutations and compile/export flow live in
  [graphStoreEditorActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreEditorActions.js:1),
  but compile and persistence responsibilities are now split again into
  [graphStoreCompileActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileActions.js:1)
  and
  [graphStorePersistenceActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStorePersistenceActions.js:1).
  Runtime behavior remains behind
  [graphStoreRuntimeActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeActions.js:1),
  which now composes
  [graphStoreRuntimeHistoryActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryActions.js:1)
  and
  [graphStoreRuntimeSessionActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeSessionActions.js:1),
  with pure session-state shaping isolated in
  [graphStoreRuntimeSessionState.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeSessionState.js:1).
  Route registration is no longer assembled inline in
  [src/main.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/main.rs:1);
  it now composes through
  [app_router.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/app_router.rs:1),
  while frontend DTOs and runtime mapper logic now live in
  [frontend_api_types.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/frontend_api_types.rs:1)
  and
  [frontend_runtime_mapping.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/frontend_runtime_mapping.rs:1).
  Compile and runtime-history actions no longer call backend endpoints
  directly; request protocol wrappers now live in
  [graphStoreCompileApi.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileApi.js:1)
  and
  [graphStoreRuntimeHistoryApi.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryApi.js:1).
  Runtime response DTO shaping and runtime record persistence are no longer
  owned by
  [src/main.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/main.rs:1); they now
  live in
  [runtime_response_mapping.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_response_mapping.rs:1)
  and
  [runtime_persistence.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_persistence.rs:1).
  On the frontend, compile actions now delegate pure state shaping to
  [graphStoreCompileState.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileState.js:1),
  while runtime-history actions now delegate selection/history reducers and
  detail graph projection to
  [graphStoreRuntimeHistoryState.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryState.js:1)
  and
  [graphStoreRuntimeHistoryProjection.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryProjection.js:1).
  The seventh decomposition slice now also moves API error serialization,
  runtime capability/request validation, and runtime event projection into
  [api_errors.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/api_errors.rs:1),
  [runtime_validation.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_validation.rs:1),
  and
  [runtime_event_projection.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/runtime_event_projection.rs:1).
  Compile orchestration now delegates the backend compile pipeline to
  [graphStoreCompileFlow.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileFlow.js:1),
  so
  [graphStoreCompileActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileActions.js:1)
  is now a thinner shell over state updates and persistence.
  The eighth decomposition slice now also moves backtest compare/report DTOs
  into
  [backtest_compare_types.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/backtest_compare_types.rs:1),
  app/runtime route helpers into
  [app_runtime_helpers.rs](/D:/rust-js-pr/QuantPilot/quantpilot/src/app_runtime_helpers.rs:1),
  and compile protocol outcome mapping into
  [graphStoreCompileOutcomeMapping.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileOutcomeMapping.js:1),
  leaving
  [graphStoreCompileFlow.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileFlow.js:1)
  as a thinner orchestration layer.
  The ninth decomposition slice now also moves compile protocol-step
  orchestration into
  [graphStoreCompileProtocolFlow.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileProtocolFlow.js:1),
  leaving
  [graphStoreCompileFlow.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileFlow.js:1)
  at roughly `51` lines.
  [graphStore.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStore.js:1)
  is now roughly `279` lines. The tenth decomposition slice now also moves
  compile outcome projection into
  [graphStoreCompileOutcomeProjection.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileOutcomeProjection.js:1)
  and runtime-history orchestration into
  [graphStoreRuntimeHistoryFlow.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryFlow.js:1),
  leaving
  [graphStoreCompileOutcomeMapping.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileOutcomeMapping.js:1)
  at roughly `55` lines,
  [graphStoreCompileProtocolFlow.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileProtocolFlow.js:1)
  at roughly `94` lines,
  [graphStoreRuntimeHistoryActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryActions.js:1)
  at roughly `48` lines, and
  [graphStoreCompileActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileActions.js:1)
  at roughly `148` lines. Direct tests now cover the new pure modules in
  [graphStoreCompileOutcomeProjection.test.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreCompileOutcomeProjection.test.js:1)
  and
  [graphStoreRuntimeHistoryFlow.test.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStoreRuntimeHistoryFlow.test.js:1),
  so this decomposition task is closed under the current queue definition.

## P2

P2 status: complete on `2026-04-23`

### OPT-P2-001 CSS and page-level bundle hygiene

- `ID`: `OPT-P2-001`
- `Priority`: `P2`
- `Status`: `complete`
- `Problem`:
  Large page and style files continue to grow, especially
  `frontend/src/styles.css` and workspace/backtest pages.
- `Why it matters`:
  This increases regression surface and makes UI review harder.
- `Scope`:
  Styling organization and route-level code splitting only.
- `Actions`:
  Split styles and heavy pages into domain-owned sections where it reduces
  coupling without changing the visual contract.
- `Dependencies`:
  `OPT-P0-005`
- `Acceptance`:
  No user-facing regressions.
  Styling and route ownership are easier to review.
- `Completion notes`:
  The first `P2` slice is landed. Strategy-workspace-specific styles now live
  in
  [strategy-workspace.css](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/strategy-workspace.css:1)
  and are imported by
  [StrategyWorkspacePage.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyWorkspacePage.jsx:1)
  rather than being kept inside the global
  [styles.css](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/styles.css:1)
  bundle. The next slice is also landed: workspace-only presentation ownership
  now lives in
  [StrategyWorkspacePageSections.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyWorkspacePageSections.jsx:1),
  shared backtest-page formatting and metric helpers now live in
  [backtestAnalysisShared.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/backtestAnalysisShared.jsx:1),
  and core backtest-analysis styling now lives in
  [backtest-analysis.css](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/backtest-analysis.css:1)
  through
  [BacktestAnalysisLayout.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/BacktestAnalysisLayout.jsx:1).
  The latest slice also removed duplicated workspace helper ownership from
  [StrategyWorkspacePage.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyWorkspacePage.jsx:1),
  restored the page-local Chinese copy contract, and moved another layer of
  analysis responsive styling out of the global
  [styles.css](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/styles.css:1)
  into
  [backtest-analysis.css](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/backtest-analysis.css:1).
  The newest slice also extracted issue-queue ownership into
  [StrategyWorkspaceIssueQueueCard.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyWorkspaceIssueQueueCard.jsx:1)
  and converted the heaviest workspace-only components
  (`ModuleSidebar`, `StrategyCanvas`, `StrategyCodePanel`,
  `StrategyDiagnosticsPanel`, `StrategyParamsPanel`, and `DiagnosticsPanel`)
  into lazy-loaded route children owned by
  [StrategyWorkspacePage.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyWorkspacePage.jsx:1).
  The latest slice then pushed the remaining workspace derived data and tab
  orchestration into
  [useStrategyWorkspacePageData.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/hooks/useStrategyWorkspacePageData.js:1),
  [StrategyWorkspaceOverviewTab.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyWorkspaceOverviewTab.jsx:1),
  [StrategyWorkspaceCodeTab.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyWorkspaceCodeTab.jsx:1),
  [StrategyWorkspaceDiagnosticsTab.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyWorkspaceDiagnosticsTab.jsx:1),
  and
  [StrategyWorkspaceResearchTab.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyWorkspaceResearchTab.jsx:1),
  leaving
  [StrategyWorkspacePage.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyWorkspacePage.jsx:1)
  as a thin route shell over tab selection and shared state wiring.
  After this slice, the production build emits route-owned CSS chunks for both
  workspace and backtest analysis pages, the global `index.css` bundle is down
  to roughly `79.48 KB`, the route-owned backtest-analysis CSS chunk is about
  `10.72 KB`, and `StrategyWorkspacePage` JavaScript ownership is now split
  into a roughly `23.24 KB` route shell plus lazy-loaded tab chunks such as
  `StrategyWorkspaceOverviewTab` (`7.27 KB`),
  `StrategyWorkspaceDiagnosticsTab` (`7.66 KB`),
  `StrategyWorkspaceCodeTab` (`5.53 KB`),
  `StrategyCanvas` (`19.11 KB`), and `ModuleSidebar` (`9.05 KB`). The page
  source itself is down to roughly `222` lines while
  [styles.css](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/styles.css:1)
  is down to roughly `4350` lines. The remaining bottleneck is no longer a
  single `130 KB` route file, but page-local logic and CSS that still
  deserve further route ownership cleanup.
  The newest follow-on slice then consolidated issue-queue pure ownership into
  [strategyWorkspaceIssueQueue.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyWorkspaceIssueQueue.js:1),
  left
  [StrategyWorkspaceIssueQueueCard.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyWorkspaceIssueQueueCard.jsx:1)
  as a render-only card, removed duplicated filter/order logic from
  [useStrategyWorkspaceUiState.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/hooks/useStrategyWorkspaceUiState.js:1),
  and moved the remaining workspace loading and backtest-analysis presentation
  styles out of the global stylesheet into
  [strategy-workspace.css](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/strategy-workspace.css:1)
  and
  [backtest-analysis.css](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/backtest-analysis.css:1).
  After this slice, the production build reports the global `index.css`
  bundle at about `78.80 KB`, the route-owned backtest-analysis CSS chunk at
  about `11.10 KB`, the route-owned workspace CSS chunk at about `18.04 KB`,
  and the `StrategyWorkspacePage` route shell chunk at about `24.26 KB`.
  [StrategyWorkspacePage.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyWorkspacePage.jsx:1)
  is now about `235` lines; the remaining `P2` work is no longer duplicated
  issue-queue ownership, but continued route-level CSS and bundle cleanup.
  The closing `P2-001` slice is also landed: strategy-hub-specific styles now
  live in
  [strategy-hub.css](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/strategy-hub.css:1)
  and are imported by
  [StrategyHubPage.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubPage.jsx:1)
  instead of remaining in the global stylesheet. After this slice, the build
  emits a dedicated `StrategyHubPage` CSS chunk at about `10.75 KB`, the
  global `index.css` bundle is down again to about `66.48 KB`, the
  `StrategyWorkspacePage` CSS chunk is about `18.38 KB`, the
  backtest-analysis CSS chunk is about `11.10 KB`, and
  [styles.css](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/styles.css:1)
  is down to about `4299` lines. This brings route-owned styling for the heavy
  strategy hub, workspace, and backtest pages under reviewable page-local
  ownership. The final follow-on slice also moves hub-page logic and chunk
  ownership behind route-local lazy sections:
  [StrategyHubHeroSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubHeroSection.jsx:1),
  [StrategyHubBodySection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubBodySection.jsx:1),
  and
  [StrategyHubInlineNote.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubInlineNote.jsx:1),
  leaving
  [StrategyHubPage.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubPage.jsx:1)
  as a route shell of roughly `22` lines. The latest build reports the
  `StrategyHubPage` route shell chunk at about `9.38 KB`, with the extracted
  hero and body shell chunks at about `6.56 KB` and `3.28 KB`. The follow-on body
  cleanup slice then pushes body-level derived data into
  [useStrategyHubBodyData.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/hooks/useStrategyHubBodyData.js:1),
  leaving
  [StrategyHubBodySection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubBodySection.jsx:1)
  as a roughly `33` line shell that lazy-loads
  [StrategyHubRosterSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRosterSection.jsx:1)
  and
  [StrategyHubInspectorSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubInspectorSection.jsx:1).
  The latest follow-on slice also pushes inspector-derived data into
  [useStrategyHubInspectorData.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/hooks/useStrategyHubInspectorData.js:1)
  and
  [strategyHubInspectorProjection.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubInspectorProjection.js:1),
  while inspector-owned rendering is split across
  [StrategyHubRecentBacktestsSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRecentBacktestsSection.jsx:1),
  [StrategyHubRecentRunsSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRecentRunsSection.jsx:1),
  and
  [StrategyHubCompareQueueSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubCompareQueueSection.jsx:1).
  The next roster slice also moves roster-only projection into
  [useStrategyHubRosterData.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/hooks/useStrategyHubRosterData.js:1)
  and
  [strategyHubRosterProjection.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubRosterProjection.js:1),
  while directory and activity ownership now live in
  [StrategyHubRosterDirectorySection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRosterDirectorySection.jsx:1)
  and
  [StrategyHubActivityPanelsSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubActivityPanelsSection.jsx:1).
  The next directory slice then pushes toolbar and table ownership into
  [StrategyHubRosterToolbar.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRosterToolbar.jsx:1)
  and
  [StrategyHubRosterTableSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRosterTableSection.jsx:1),
  while activity-card ownership now lives in
  [StrategyHubBacktestActivityCard.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubBacktestActivityCard.jsx:1)
  and
  [StrategyHubRunActivityCard.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRunActivityCard.jsx:1),
  leaving
  [StrategyHubRosterSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRosterSection.jsx:1)
  as a roughly `30` line shell.
  The next row/action slice then pushes row and row-action ownership into
  [StrategyHubRosterTableRow.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRosterTableRow.jsx:1)
  and
  [StrategyHubRosterRowActions.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRosterRowActions.jsx:1),
  leaving
  [StrategyHubRosterTableSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRosterTableSection.jsx:1)
  as a roughly `27` line shell.
  The next action-semantics slice then pushes grouped action projection and
  row-action dispatch semantics into
  [strategyHubRosterRowActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubRosterRowActions.js:1),
  leaving
  [StrategyHubRosterRowActions.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRosterRowActions.jsx:1)
  as a roughly `34` line render shell and keeping
  [StrategyHubRosterTableSection.test.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRosterTableSection.test.jsx:1)
  on semantic button labels rather than DOM-order assumptions, while
  [strategyHubRosterRowActions.test.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubRosterRowActions.test.js:1)
  now locks the extracted pure module directly.
  The next inspector-overview slice then pushes header, summary, metrics, next
  move, and inspector action semantics into
  [StrategyHubInspectorOverviewSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubInspectorOverviewSection.jsx:1)
  and
  [strategyHubInspectorActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubInspectorActions.js:1),
  leaving
  [StrategyHubInspectorSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubInspectorSection.jsx:1)
  as a roughly `45` line shell while
  [strategyHubInspectorProjection.test.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubInspectorProjection.test.js:1)
  and
  [strategyHubInspectorActions.test.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubInspectorActions.test.js:1)
  directly lock the extracted pure modules.
  The next inspector-interaction slice then pushes recent-backtests and
  compare-queue action semantics into
  [strategyHubRecentBacktestsActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubRecentBacktestsActions.js:1)
  and
  [strategyHubCompareQueueActions.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubCompareQueueActions.js:1),
  leaving
  [StrategyHubRecentBacktestsSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRecentBacktestsSection.jsx:1)
  and
  [StrategyHubCompareQueueSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubCompareQueueSection.jsx:1)
  as roughly `59` and `52` line render shells while
  [strategyHubRecentBacktestsActions.test.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubRecentBacktestsActions.test.js:1)
  and
  [strategyHubCompareQueueActions.test.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubCompareQueueActions.test.js:1)
  directly lock the extracted interaction modules.
  The next recent-runs slice then pushes section copy and run-row ownership into
  [strategyHubRecentRunsView.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubRecentRunsView.js:1)
  and
  [StrategyHubRecentRunItem.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRecentRunItem.jsx:1),
  leaving
  [StrategyHubRecentRunsSection.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubRecentRunsSection.jsx:1)
  as a roughly `18` line shell while
  [strategyHubRecentRunsView.test.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/utils/strategyHubRecentRunsView.test.js:1)
  now directly locks the extracted view module. This same slice also rewrites
  [StrategyHubSharedComponents.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/StrategyHubSharedComponents.jsx:1)
  as clean UTF-8 shared presentation ownership so active hub routes no longer
  carry mojibake through the reusable card/task wrappers.
  The current build reports the `StrategyHubPage` route shell chunk at about
  `9.38 KB`, the `StrategyHubBodySection` shell chunk at about `2.95 KB`, the
  `StrategyHubRosterSection` shell chunk at about `2.56 KB`, the
  `StrategyHubRosterDirectorySection` chunk at about `1.57 KB`, the
  `StrategyHubRosterToolbar` chunk at about `1.01 KB`, the
  `StrategyHubRosterTableSection` chunk at about `3.06 KB`, the
  `StrategyHubRosterTableRow` chunk at about `0.34 KB`, the
  `StrategyHubActivityPanelsSection` chunk at about `1.23 KB`, the
  `StrategyHubBacktestActivityCard` chunk at about `0.82 KB`, the
  `StrategyHubRunActivityCard` chunk at about `0.25 KB`, the
  `StrategyHubInspectorSection` chunk at about `5.88 KB`, and extracted
  recent-backtests / recent-runs / compare-queue chunks at about `1.60 KB`,
  `0.83 KB`, and `1.50 KB`, so `OPT-P2-001` now stays complete across both
  route-owned CSS and finer-grained hub-page JavaScript ownership.

### OPT-P2-002 Repo release hygiene and baseline commit prep

- `ID`: `OPT-P2-002`
- `Priority`: `P2`
- `Status`: `complete`
- `Problem`:
  The repository still lacks a first commit and still has unresolved
  release-hygiene items such as `LICENSE`.
- `Why it matters`:
  Release readiness and auditability remain weak even after test fixes.
- `Scope`:
  Release documents, license decision placeholder, baseline repo hygiene only.
- `Actions`:
  Keep the readiness checklist explicit.
  Prepare the repository for a clean baseline commit once the quality gates are
  green.
- `Dependencies`:
  All `P0` items.
- `Acceptance`:
  Release-readiness docs reflect the actual remaining owner decisions.
  The repo is ready for a stable baseline commit once code gates are green.
- `Completion notes`:
  A placeholder
  [LICENSE](/D:/rust-js-pr/QuantPilot/quantpilot/LICENSE)
  now makes the current legal state explicit without guessing a public license.
  Release readiness is documented in
  [implementation-first-release-readiness.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/planning/implementation-first-release-readiness.md:1),
  and the planning index now separates active queue docs from historical
  background notes in
  [planning/README.md](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/planning/README.md:1).
  The repository is now prepared for a baseline first commit once the owner
  chooses whether to keep the placeholder license state or replace it with the
  final outbound license text.

## Execution order

1. `P0` is complete and the repository gate set is green.
2. Start `P1` structural cleanup without reopening closed `P0` scope.
3. Keep `P2` strictly behind continued quality-gate stability.

## Gate set for this queue

Run these checks after each meaningful batch:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1
cargo test --workspace
cd frontend; npm.cmd run test
cd frontend; npm.cmd run build
```
