# Implementation Optimization Acceptance Matrix

This matrix records the current acceptance rules for the remediation queue and
the latest audit result against those rules.

Audit date: `2026-04-23`

## Acceptance status legend

- `pass`: currently aligned
- `attention`: aligned in principle but still needs cleanup
- `fail`: currently blocks phase close

## Matrix

| Area | Rule | Current status | Evidence | Required action |
|---|---|---|---|---|
| Rust workspace gate | `cargo test --workspace` must pass | pass | stable fixture now lives in `tests/fixtures/report_sma20_100_daily_btc.qs` and workspace tests pass on `2026-04-22` | keep tests out of runtime artifact directories |
| Frontend unit gate | `frontend` unit tests must pass | pass | `frontend` `npm.cmd run test` passed on `2026-04-22` after repairing the previous red set | keep new tests explicit about locale and runtime state |
| Frontend build gate | production build must pass | pass | `npm.cmd run build` succeeded on `2026-04-22` | keep as smoke guard during `P1` cleanup |
| UTF-8 rule | active docs and frontend files must be `UTF-8` without BOM | pass | `tools/check-utf8.ps1` passed on `2026-04-22` after BOM cleanup | keep touched markdown and frontend files BOM-free |
| User-facing text rule | no mojibake in active UI copy | pass | `tools/check-user-facing-text.ps1` passed on `2026-04-22` after restoring active frontend copy | keep Chinese UI strings clean and capability-honest |
| Capability governance | generated snapshot must match backend truth | pass | `tools/check-capability-governance.ps1` passed on `2026-04-22` after snapshot regeneration | re-run snapshot generation whenever capability truth changes |
| Capability source of truth | backend `/api/capabilities` stays authoritative | pass | backend handler now delegates only to `build_capability_response()`, and docs plus generated snapshot are aligned | keep capability output on one response-builder path |
| Compile artifact priority | runtime compile remains source of truth after preflight/fallback | pass | repaired tests now pass across compile summary, diagnostics, and Strategy IR integration | keep docs, tests, and compile summary wording aligned |
| Main-chain discipline | no bypass around `Data -> Intent -> Agent -> Risk -> Execution` | pass | current audit found no direct bypass in touched paths | preserve during refactors |
| Backend entry decomposition | oversized pure helper clusters should move out of `src/main.rs` without changing routes | pass | route/domain handlers, CLI helpers, compile artifact builders, formal QuantScript authoring DTOs, frontend DTO/runtime mappers, runtime response/persistence helpers, API error/validation/event projection helpers, graph QuantScript helpers, and compare/report ownership now live in dedicated modules outside `src/main.rs`; the production ownership slice now hands off to the embedded test module at `src/main.rs:1673` even though the file still contains in-file tests | keep any further thinning focused on test relocation or future route groups rather than reintroducing cross-domain ownership |
| Frontend store decomposition | helper logic should not live in the zustand store shell | pass | `graphStore.js` remains about `279` lines, compile/history API calls live in dedicated protocol modules, compile/history state shaping lives in dedicated reducer/projection modules, compile outcome projection now lives in `graphStoreCompileOutcomeProjection.js`, runtime-history orchestration now lives in `graphStoreRuntimeHistoryFlow.js`, and new direct tests cover both extracted pure modules | keep new logic out of the store shell and add direct tests whenever a new pure mapper or flow module is introduced |
| Planning-doc compression | active docs should show only current unfinished work | pass | `markdown/implementation/planning/README.md` now separates active optimization docs from historical background notes, and the normalized legacy planning files already declare themselves background-only | keep new cleanup work in the active queue instead of reopening historical planning branches |
| CSS and page-level bundle hygiene | route-owned styles should not remain trapped in the global stylesheet when they only serve one heavy page | pass | strategy-workspace-specific styles now live in `frontend/src/pages/strategy-workspace.css`, core backtest-analysis styles now live in `frontend/src/pages/backtest-analysis.css` through `BacktestAnalysisLayout.jsx`, strategy-hub-specific styles now live in `frontend/src/pages/strategy-hub.css` through `StrategyHubPage.jsx`, shared backtest-page formatting now lives in `frontend/src/pages/backtestAnalysisShared.jsx`, issue-queue pure ownership now lives in `frontend/src/utils/strategyWorkspaceIssueQueue.js`, render ownership now lives in `frontend/src/pages/StrategyWorkspaceIssueQueueCard.jsx`, duplicated issue-queue filter/order logic has been removed from `frontend/src/hooks/useStrategyWorkspaceUiState.js`, workspace-derived data now lives in `frontend/src/hooks/useStrategyWorkspacePageData.js`, tab-level orchestration now lives in `StrategyWorkspaceOverviewTab.jsx`, `StrategyWorkspaceCodeTab.jsx`, `StrategyWorkspaceDiagnosticsTab.jsx`, and `StrategyWorkspaceResearchTab.jsx`, heavy workspace-only components now lazy-load from those route-owned tab modules, hub-page logic now lazy-loads through `StrategyHubHeroSection.jsx` and `StrategyHubBodySection.jsx`, hub-body derived data now lives in `useStrategyHubBodyData.js` while roster and inspector ownership now live in `StrategyHubRosterSection.jsx` and `StrategyHubInspectorSection.jsx`, inspector-only projection plus section ownership now live in `useStrategyHubInspectorData.js`, `strategyHubInspectorProjection.js`, `StrategyHubRecentBacktestsSection.jsx`, `StrategyHubRecentRunsSection.jsx`, and `StrategyHubCompareQueueSection.jsx`, the next inspector-overview slice now pushes header, summary, metrics, next-move copy, and action semantics into `StrategyHubInspectorOverviewSection.jsx` and `strategyHubInspectorActions.js` with direct coverage in `strategyHubInspectorProjection.test.js` and `strategyHubInspectorActions.test.js`, the next inspector-interaction slice now pushes recent-backtests and compare-queue action semantics into `strategyHubRecentBacktestsActions.js` and `strategyHubCompareQueueActions.js` with direct coverage in `strategyHubRecentBacktestsActions.test.js` and `strategyHubCompareQueueActions.test.js`, the next recent-runs slice now pushes section copy and run-row ownership into `strategyHubRecentRunsView.js` and `StrategyHubRecentRunItem.jsx` with direct coverage in `strategyHubRecentRunsView.test.js`, while `StrategyHubSharedComponents.jsx` has been rewritten as clean UTF-8 shared presentation ownership for active hub routes, roster-only projection plus section ownership now live in `useStrategyHubRosterData.js`, `strategyHubRosterProjection.js`, `StrategyHubRosterDirectorySection.jsx`, and `StrategyHubActivityPanelsSection.jsx`, the next directory/activity slice now pushes finer ownership into `StrategyHubRosterToolbar.jsx`, `StrategyHubRosterTableSection.jsx`, `StrategyHubBacktestActivityCard.jsx`, and `StrategyHubRunActivityCard.jsx`, the next row/action slice now pushes ownership into `StrategyHubRosterTableRow.jsx` and `StrategyHubRosterRowActions.jsx`, and the next action-semantics slice now pushes grouped action projection and row-action dispatch semantics into `strategyHubRosterRowActions.js` with direct coverage in `strategyHubRosterRowActions.test.js` while `StrategyHubRosterTableSection.test.jsx` now asserts semantic button labels instead of DOM-order assumptions; the current build reports the global `index.css` bundle at about `66.48 KB`, the route-owned strategy-hub CSS chunk at about `10.75 KB`, the route-owned backtest-analysis CSS chunk at about `11.10 KB`, the route-owned workspace CSS chunk at about `18.38 KB`, the `StrategyWorkspacePage` route shell chunk at about `24.26 KB`, the `StrategyHubPage` route shell chunk at about `9.38 KB`, the `StrategyHubBodySection` shell chunk at about `2.95 KB`, the `StrategyHubRosterSection` shell chunk at about `2.56 KB`, the `StrategyHubRosterDirectorySection` chunk at about `1.57 KB`, the `StrategyHubRosterToolbar` chunk at about `1.01 KB`, the `StrategyHubRosterTableSection` chunk at about `3.06 KB`, the `StrategyHubRosterTableRow` chunk at about `0.34 KB`, the `StrategyHubActivityPanelsSection` chunk at about `1.23 KB`, the `StrategyHubBacktestActivityCard` chunk at about `0.82 KB`, the `StrategyHubRunActivityCard` chunk at about `0.25 KB`, the `StrategyHubInspectorSection` chunk at about `5.88 KB`, and extracted hub hero / recent-backtests / recent-runs / compare-queue chunks at about `6.56 KB`, `1.60 KB`, `0.83 KB`, and `1.50 KB`, while the formerly inlined heavy child weight now sits in lazy-loaded chunks such as `StrategyCanvas` (`19.11 KB`) and `ModuleSidebar` (`9.05 KB`) and `StrategyWorkspacePage.jsx` itself is about `235` lines | keep future page-only style and logic ownership local and avoid regressing heavy route code back into the global shell |
| Release hygiene and baseline commit prep | legal and release-readiness docs should make the current first-release state explicit without guessing owner policy | pass | the repository now carries a placeholder `LICENSE`, `implementation-first-release-readiness.md` records the remaining owner decisions, and the README now points at replacing the placeholder before any public first release | keep baseline-commit prep explicit and replace the placeholder license text only through an owner decision |
| Markdown coverage | active implementation changes should be documented in active markdown | pass | queue, acceptance rules, and roadmap now record the completed `P0` batch and green gate state | keep active docs current when phase state changes |

## Principle audit

### Matches current development thought

- backend capability output is already documented as authoritative
- main-chain runtime semantics remain consistent with the current principles
- current docs already reject overclaiming unsupported beta capability
- current roadmap already prefers cleanup and wording honesty over new growth

### Deviations that must be corrected

- the code-side ownership bottlenecks tracked by `OPT-P1-004` and
  `OPT-P1-005` are now closed, but `src/main.rs` still carries a large embedded
  test block and golden-view coverage that could be moved later if file-size
  ergonomics become the next bottleneck
- the touched frontend tests are now stable, but the wider test suite still
  mixes exact rendered prose assertions with semantic assertions and remains
  costly to maintain when copy changes

## Testing landmines

These items are not theoretical; they are active failure sources or near-term
risk points.

- fixture paths under `storage/` are brittle because `storage/` is a runtime
  artifact domain, not a stable source fixture domain. This specific red path is
  fixed now, but the rule should be kept explicit for new tests
- locale-dependent text assertions fail when the default locale changes. The
  repaired tests now avoid this, but the pattern is still a repo-wide risk
- EventSource-related tests are fragile when store connection setup changes
- store tests that assume a specific seeded node ordering break easily when
  initialization changes. The new validated-graph helper reduces this risk for
  touched tests
- compile-path tests can silently drift if artifact-resolution wording changes in
  code but not in docs or fixtures
- backend entry tests still live inside `src/main.rs`, so large golden-view
  updates will continue to create noisy diffs until that test block is split

## Compatibility check

### Compatible areas

- frontend build still succeeds with current bundle split
- backend API route layout still matches the current beta documentation
- full workspace Rust tests now pass, which indicates the domain crates and
  integration layers remain coherent after the fixture repair
- frontend compile, diagnostics, backtests, and runtime-error paths are again
  aligned with current tests and docs
- extracted Rust modules, including `app_router.rs`, `compile_api.rs`,
  `runtime_api.rs`, `graph_api.rs`, `frontend_api_types.rs`,
  `frontend_runtime_mapping.rs`, `runtime_response_mapping.rs`,
  `runtime_persistence.rs`, `api_errors.rs`, `runtime_validation.rs`, and
  `runtime_event_projection.rs`, `backtest_compare_types.rs`,
  `app_runtime_helpers.rs`, `graph_quantscript_api.rs`, `cli_support.rs`,
  `compile_artifact_builders.rs`, and
  `formal_quantscript_authoring_types.rs`, and graph-store
  helper/action/API/flow/protocol-flow/outcome-mapping/outcome-projection/state/projection/runtime-history-flow
  submodules preserved the existing public contracts and test expectations

### Compatibility risks

- frontend compile summary and diagnostics wording may no longer match backend
  contract wording
- locale and text corruption can make frontend behavior appear incompatible even
  when underlying data flow is correct
- stale capability snapshot content can mislead other modules or docs about what
  is actually supported

## Markdown coverage check

Current implementation areas already described in markdown:

- beta boundary and current roadmap
- capability governance and support matrix
- QuantScript trunk and freeze rules
- runtime and backtest implementation boundaries
- core design principles

Still required after this audit:

- keep the active queue docs current if a new remediation batch is opened
- refresh roadmap wording again if a future queue changes the current blocker set

## Phase-close rule

The optimization queue phase is now complete because the previous `fail` and
`attention` entries are closed.

Open a new queue only when a new red gate, contract drift, or release-hygiene
gap appears.
