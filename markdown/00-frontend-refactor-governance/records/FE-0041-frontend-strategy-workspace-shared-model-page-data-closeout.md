# FE-0041 Frontend Strategy Workspace Shared Model And Page Data Closeout

Status: closed.

## Leaf Node

`frontend.strategy_workspace.shared_model_and_page_data`

## Code Changes

- Added `frontend/src/hooks/strategyWorkspacePageDataProjection.js`.
- Added `frontend/src/hooks/strategyWorkspacePageDataProjection.test.js`.
- Updated `frontend/src/hooks/useStrategyWorkspacePageData.js` to delegate primitive formatting, readiness projection, recent activity selection, overview metrics, preview rows, status highlights, diagnostics highlights, and compare selection resolution to the projection module.

## Preserved Behavior

- `useStrategyWorkspacePageData` still returns the same public page data fields consumed by `StrategyWorkspacePage.jsx` and its tab children.
- Recent runs and backtests remain graph-scoped, sorted newest-first, and limited to four items by default.
- Readiness labels and tones still prioritize active validation issues before runnable and compilable states.
- Compile output, diagnostics count, run preview, backtest preview, overview highlight, diagnostics highlight, and compare selection fallbacks remain equivalent.
- Canvas recommendation and repair path state stay inside the hook because they still depend on React memoization and selected graph state.

## Public Inputs

- Strategy graph.
- Runtime history, backtest history, and compare selection state.
- Compile summary diagnostics and outputs.
- Validation issue counts and runnable state.
- Workspace issue queue counts and source lanes.
- Active tab, selected node, selected edge, and inspector panel state.

## Public Outputs

- `formatWorkspaceTime(value)`.
- `formatWorkspaceCount(value)`.
- `formatWorkspacePercent(value)`.
- `compileWorkspaceOutputsText(outputs)`.
- `countWorkspaceDiagnostics(diagnostics)`.
- `resolveWorkspaceReadiness({ isRunnable, isCompilable, issueCount })`.
- `selectRecentWorkspaceActivity(items, currentGraphId, limit)`.
- `resolveWorkspaceCompareSelection(runtime, graphId)`.
- `buildWorkspaceOverviewMetrics(input)`.
- `buildWorkspaceRunPreviewItems(recentRuns)`.
- `buildWorkspaceBacktestPreviewItems(recentBacktests)`.
- `buildWorkspaceOverviewStatusHighlights(input)`.
- `buildWorkspaceDiagnosticsStatusHighlights(input)`.
- `useStrategyWorkspacePageData(input)` as the React facade.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/hooks/strategyWorkspacePageDataProjection.test.js src/pages/StrategyWorkspacePage.codeMode.test.jsx src/pages/StrategyWorkspaceVersionHistoryCard.test.jsx`: passed, 3 test files and 14 tests.

## Further-Split Decision

`frontend.strategy_workspace.shared_model_and_page_data` does not need a deeper split yet. The pure projection boundary is now separated from the hook facade, while the shared model selector remains intentionally small and reused by workspace and backtest pages.

## Residuals

- Continue with `frontend.strategy_workspace.issue_queue_state`.
