# Implementation Optimization Blueprint

This blueprint defines how the current remediation queue should be executed.
It is active for the present repository cleanup phase.

## Objective

Close the current phase with one honest, testable, and UTF-8-clean beta
surface.

The immediate optimization target is not feature growth.
It is repository convergence:

- green repository quality gates
- one authoritative capability truth path
- no mojibake or BOM in active frontend/docs surfaces
- compile/runtime/docs/test wording aligned with the retained `V1` boundary
- reduced duplicate truth sources in code and planning docs

## Non-goals

Do not use this blueprint to reopen deferred scope such as:

- research-grade backtest widening
- generic risk or execution DSL growth
- live trading
- wider spread semantics
- third-party plugin marketplace exposure

## Governing principles

This queue must be executed under the following rules:

- follow the main chain:
  `Data -> Intent -> Agent -> Risk -> Execution`
- do not add or preserve code paths that bypass the current contract wording
- prefer deleting duplicate truth sources over maintaining legacy parallel paths
- treat text encoding defects as product defects
- keep backend `/api/capabilities` authoritative when capability truth drifts
- do not widen `V1` just to make stale tests pass

Primary references:

- [QuantPilot Design Principles](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/principles/principles-quantpilot-design.md)
- [Data And Intent Layer Principles](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/principles/principles-data-and-intent-layer.md)
- [Current Status And Roadmap](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/overview/overview-current-status-and-roadmap.md)
- [V1 Freeze / De-scope Checklist](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/guides/quantscript/guide-v1-freeze-descope-checklist.md)

## Current blockers

The gate-repair portion of this queue is now closed.

Closed blockers in the completed `P0` batch:

- workspace Rust tests were restored by moving the report strategy sample into
  `tests/fixtures/`
- frontend unit tests were restored across compile, locale-sensitive UI text,
  runtime error handling, and editor-state paths
- UTF-8 and user-facing-text checks are green
- capability-governance snapshot drift was cleared

Remaining optimization blockers:

- none inside the current remediation queue
- release-hygiene prep is now explicit through the placeholder `LICENSE` file
  and the first-release-readiness checklist, so the remaining first-commit
  decision is an owner action rather than an open code/document cleanup blocker

## Execution strategy

### Phase 1: restore gate integrity

- fix missing test fixtures
- normalize touched files to `UTF-8` without BOM
- repair mojibake in active frontend source
- regenerate capability-governance snapshot
- restore all currently failing frontend tests

Phase 1 status: complete on `2026-04-22`

### Phase 2: converge contract paths

- repair `graphStore` compile-chain behavior against documented artifact
  priority
- remove stale capability-response branches
- align diagnostics and property-panel wording with current docs
- document remaining restricted paths instead of hiding them behind stale copy

### Phase 3: structural cleanup

- split oversized backend and frontend ownership bottlenecks
- move pure logic out of stateful orchestration files
- compress planning docs so they describe only active work

## Deletion rule

When a code path is both stale and clearly outside the retained development
direction, prefer deletion over narrowing shims.

Examples:

- unreachable legacy response builders
- duplicate capability truth paths
- tests that preserve outdated product wording after the current wording contract
  has already changed

## Required outputs of this phase

Before this phase can be called complete, all of the following must be true:

- `cargo test --workspace` passes
- `frontend` unit tests pass
- UTF-8 and mojibake checks pass
- capability-governance snapshot matches current backend truth
- active markdown docs point to the same current queue and acceptance rules
- no active doc still implies that the current priority is new feature widening

Current status:

- all phase outputs above are now satisfied for the full optimization queue
- the optimization queue is now closed; future work should reopen from the
  roadmap lanes or a new written remediation queue instead of pretending this
  cleanup batch is still in progress
- the first `P1` slices are already landed:
  capability output, compile diagnostics, and backtest compare/report logic are
  no longer embedded directly inside `src/main.rs`, and shared graph-store
  helpers are no longer embedded directly inside `graphStore.js`
- the second `P1` structure slice is also landed:
  `backtest_compare` is now split into compare-core and narrative/report layers,
  and `graphStore` now delegates editor actions and runtime actions to dedicated
  modules while keeping repository gates green
- the third `P1` structure slice is also landed:
  graph save/load/list and file-manager handlers now live in `graph_api.rs`,
  `graphStoreHelpers.js` is reduced to a thin aggregator over compile,
  persistence, and runtime helper modules, and runtime/editor actions are split
  again into compile, persistence, runtime-history, and runtime-session modules
  while keeping repository gates green
- the fourth `P1` structure slice is also landed:
  `main.rs` now registers compile/runtime/graph route groups through dedicated
  route modules, compile protocol mapping now lives in
  `graphStoreCompileProtocolMapping.js`, and runtime session state shaping now
  lives in `graphStoreRuntimeSessionState.js` while repository gates remain
  green
- the fifth `P1` structure slice is also landed:
  route registration now composes through `app_router.rs`, frontend DTO and
  mapper ownership now lives in `frontend_api_types.rs` and
  `frontend_runtime_mapping.rs`, and compile/runtime-history request protocol
  calls now live in `graphStoreCompileApi.js` and
  `graphStoreRuntimeHistoryApi.js` while repository gates remain green
- the sixth `P1` structure slice is also landed:
  runtime response shaping now lives in `runtime_response_mapping.rs`, runtime
  persistence helpers now live in `runtime_persistence.rs`, compile-state
  reducers now live in `graphStoreCompileState.js`, and runtime-history
  reducers/detail projection now live in `graphStoreRuntimeHistoryState.js` and
  `graphStoreRuntimeHistoryProjection.js` while repository gates remain green
- the seventh `P1` structure slice is also landed:
  API error shaping now lives in `api_errors.rs`, runtime capability/request
  validation now lives in `runtime_validation.rs`, runtime event projection now
  lives in `runtime_event_projection.rs`, and compile orchestration now
  delegates the backend compile pipeline to `graphStoreCompileFlow.js` while
  repository gates remain green
- the eighth `P1` structure slice is also landed:
  backtest compare/report DTO ownership now lives in
  `backtest_compare_types.rs`, app/runtime route support now lives in
  `app_runtime_helpers.rs`, and compile protocol outcome mapping now lives in
  `graphStoreCompileOutcomeMapping.js` while repository gates remain green
- the ninth `P1` structure slice is also landed:
  graph QuantScript route registration and graph-source parse/generate helpers
  now live in `graph_quantscript_api.rs`, compile protocol-step orchestration
  now lives in `graphStoreCompileProtocolFlow.js`, and
  `graphStoreCompileFlow.js` is reduced again to a thinner orchestration shell
  while repository gates remain green
- the tenth `P1` structure slice is also landed:
  CLI parsing and Strategy IR validation helpers now live in
  `cli_support.rs`, compile artifact bundle construction now lives in
  `compile_artifact_builders.rs`, formal QuantScript authoring DTO ownership
  now lives in `formal_quantscript_authoring_types.rs`, compile outcome
  projection now lives in `graphStoreCompileOutcomeProjection.js`,
  runtime-history orchestration now lives in `graphStoreRuntimeHistoryFlow.js`,
  and new direct tests cover those extracted pure modules while repository
  gates remain green
- the first `P2` hygiene slice is also landed:
  strategy-workspace-specific styles now live in
  `frontend/src/pages/strategy-workspace.css` and are imported by
  `StrategyWorkspacePage.jsx` instead of staying in the global stylesheet, so
  route-owned CSS splitting is now active without changing the workspace visual
  contract
- the second `P2` hygiene slice is also landed:
  workspace-only presentation components now live in
  `frontend/src/pages/StrategyWorkspacePageSections.jsx`, shared backtest-page
  formatting now lives in `frontend/src/pages/backtestAnalysisShared.jsx`, and
  core backtest-analysis styling now lives in
  `frontend/src/pages/backtest-analysis.css` through
  `BacktestAnalysisLayout.jsx`; the latest follow-on slice also removed
  duplicated helper ownership from `StrategyWorkspacePage.jsx`, restored the
  page-local Chinese copy contract, and moved another layer of analysis
  responsive styling out of `frontend/src/styles.css`; the global CSS bundle is
  now about `78.80 KB`
- the third `P2` hygiene slice is now landed:
  issue-queue ownership now lives in
  `frontend/src/pages/StrategyWorkspaceIssueQueueCard.jsx`, heavy
  workspace-only components now load lazily under
  `StrategyWorkspacePage.jsx`, the route chunk is down to roughly `59.44 KB`,
  and the previously monolithic workspace child weight now sits in lazy-loaded
  chunks such as `StrategyCanvas` (`19.11 KB`) and `ModuleSidebar`
  (`9.05 KB`); `StrategyWorkspacePage.jsx` itself is now roughly `984` lines
  while `frontend/src/styles.css` is down to roughly `4350` lines, so the next
  `P2` work is no longer emergency thinning but continued page-ownership
  cleanup
- the fourth `P2` hygiene slice is now landed:
  workspace-derived data now lives in
  `frontend/src/hooks/useStrategyWorkspacePageData.js`, tab-level orchestration
  now lives in `StrategyWorkspaceOverviewTab.jsx`,
  `StrategyWorkspaceCodeTab.jsx`, `StrategyWorkspaceDiagnosticsTab.jsx`, and
  `StrategyWorkspaceResearchTab.jsx`, and
  `StrategyWorkspacePage.jsx` is now a route shell of roughly `222` lines; the
  route shell chunk is down again to about `23.24 KB` while the tab-level
  weight remains split into lazy-loaded chunks such as
  `StrategyWorkspaceOverviewTab` (`7.27 KB`),
  `StrategyWorkspaceDiagnosticsTab` (`7.66 KB`), and
  `StrategyWorkspaceCodeTab` (`5.53 KB`)
- the fifth `P2` hygiene slice is now landed:
  issue-queue pure logic now lives in
  `frontend/src/utils/strategyWorkspaceIssueQueue.js`, the render shell now
  lives in `frontend/src/pages/StrategyWorkspaceIssueQueueCard.jsx`, duplicated
  issue-queue filtering/order logic has been removed from
  `frontend/src/hooks/useStrategyWorkspaceUiState.js`, and the remaining
  workspace loading plus backtest-analysis surface styles have been moved out
  of `frontend/src/styles.css` into `strategy-workspace.css` and
  `backtest-analysis.css`; the current build reports the global `index.css`
  bundle at about `78.80 KB`, the route-owned backtest-analysis CSS chunk at
  about `11.10 KB`, the route-owned workspace CSS chunk at about `18.04 KB`,
  and the `StrategyWorkspacePage` route shell chunk at about `24.26 KB` while
  `StrategyWorkspacePage.jsx` itself is now roughly `235` lines
- the closing `P2` hygiene slice is also landed:
  strategy-hub-specific styles now live in
  `frontend/src/pages/strategy-hub.css` and are imported by
  `StrategyHubPage.jsx` instead of remaining in the global stylesheet. The
  current build now emits a dedicated `StrategyHubPage` CSS chunk at about
  `10.75 KB`, the global `index.css` bundle is down to about `66.48 KB`, the
  route-owned workspace CSS chunk is about `18.38 KB`, the route-owned
  backtest-analysis CSS chunk remains about `11.10 KB`, and
  `frontend/src/styles.css` is down to roughly `4299` lines. This closes the
  first `P2` route-owned CSS and page-bundle hygiene target for the heavy
  hub/workspace/backtest pages while keeping repository gates green
- the follow-on hub ownership slice is also landed:
  `StrategyHubPage.jsx` now lazy-loads
  `StrategyHubHeroSection.jsx` and `StrategyHubBodySection.jsx`, with inline
  note behavior isolated in `StrategyHubInlineNote.jsx` and shared formatting
  isolated in `strategyHubFormatters.js`. The route shell itself is now about
  `22` lines, and the current build reports a `StrategyHubPage` route shell
  chunk at about `9.38 KB` with extracted hero/body shell chunks at about
  `6.56 KB` and `3.28 KB`
- the follow-on hub body slice is also landed:
  `StrategyHubBodySection.jsx` now delegates derived-data shaping to
  `useStrategyHubBodyData.js` and lazy-loads
  `StrategyHubRosterSection.jsx` plus `StrategyHubInspectorSection.jsx`. The
  body shell itself is now about `33` lines. The next inspector slice then
  moves inspector-only projection into `useStrategyHubInspectorData.js` and
  `strategyHubInspectorProjection.js`, while recent backtests, recent runs, and
  compare queue now render through dedicated section modules. The current build
  reports the inspector shell chunk at about `5.43 KB`, and dedicated
  recent-backtests / recent-runs / compare-queue chunks at about `1.25 KB`,
  `0.65 KB`, and `1.03 KB`
- the next inspector-overview slice is also landed:
  inspector header, summary, metrics, next-move copy, and action semantics now
  live in `StrategyHubInspectorOverviewSection.jsx` and
  `strategyHubInspectorActions.js`, leaving `StrategyHubInspectorSection.jsx`
  itself at about `45` lines as a shell; direct coverage now lives in
  `strategyHubInspectorProjection.test.js` and
  `strategyHubInspectorActions.test.js`, and the current build reports the
  inspector chunk at about `5.88 KB`
- the next inspector-interaction slice is also landed:
  recent-backtests and compare-queue action semantics now live in
  `strategyHubRecentBacktestsActions.js` and
  `strategyHubCompareQueueActions.js`, leaving
  `StrategyHubRecentBacktestsSection.jsx` and
  `StrategyHubCompareQueueSection.jsx` as roughly `59` and `52` line render
  shells; direct coverage now lives in
  `strategyHubRecentBacktestsActions.test.js` and
  `strategyHubCompareQueueActions.test.js`, and the current build reports
  recent-backtests / compare-queue chunks at about `1.60 KB` and `1.50 KB`
- the next recent-runs slice is also landed:
  recent-runs section copy and run-row ownership now live in
  `strategyHubRecentRunsView.js` and `StrategyHubRecentRunItem.jsx`, leaving
  `StrategyHubRecentRunsSection.jsx` itself at about `18` lines as a shell;
  direct coverage now lives in `strategyHubRecentRunsView.test.js`, the current
  build reports the recent-runs chunk at about `0.83 KB`, and
  `StrategyHubSharedComponents.jsx` has been rewritten as clean UTF-8 shared
  presentation ownership so active hub pages no longer depend on mojibake-laden
  reusable wrappers
- the next roster slice is also landed:
  roster-only projection now lives in `useStrategyHubRosterData.js` and
  `strategyHubRosterProjection.js`, while directory and activity ownership now
  live in `StrategyHubRosterDirectorySection.jsx` and
  `StrategyHubActivityPanelsSection.jsx`. The current build reports the body
  shell chunk at about `2.95 KB`, the roster shell chunk at about `2.56 KB`,
  the roster-directory chunk at about `1.57 KB`, and the roster-activity chunk
  at about `1.23 KB`
- the next directory slice is also landed:
  toolbar and table ownership now live in `StrategyHubRosterToolbar.jsx` and
  `StrategyHubRosterTableSection.jsx`, while activity-card ownership now lives
  in `StrategyHubBacktestActivityCard.jsx` and `StrategyHubRunActivityCard.jsx`.
  The roster shell itself is now about `30` lines, and the current build
  reports toolbar / table / backtest-activity / run-activity chunks at about
  `1.01 KB`, `2.84 KB`, `0.82 KB`, and `0.25 KB`
- the next row/action slice is also landed:
  row and row-action ownership now live in `StrategyHubRosterTableRow.jsx` and
  `StrategyHubRosterRowActions.jsx`, leaving the table shell itself at about
  `27` lines. The current build reports the row chunk at about `0.34 KB`
- the next action-semantics slice is also landed:
  grouped action projection and row-action dispatch semantics now live in
  `strategyHubRosterRowActions.js`, leaving `StrategyHubRosterRowActions.jsx`
  itself at about `34` lines as a render shell while direct coverage now lives
  in `strategyHubRosterRowActions.test.js`; the current build reports the table
  chunk at about `3.06 KB`
- the closing release-hygiene slice is also landed:
  the repository now carries an explicit placeholder `LICENSE`, release
  readiness now lives in `implementation-first-release-readiness.md`, and the
  planning index now separates active optimization docs from historical
  background notes so planning-doc compression is no longer an open queue item

## Active references

- [Implementation Optimization Task List](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/archive/planning-retired/implementation-optimization-task-list.md)
- [Implementation Optimization Acceptance Matrix](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/archive/planning-retired/implementation-optimization-acceptance-matrix.md)
