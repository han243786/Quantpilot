# FE-0042 Frontend Strategy Workspace Issue Queue State Closeout

Status: closed.

## Leaf Node

`frontend.strategy_workspace.issue_queue_state`

## Code Changes

- Added `frontend/src/hooks/strategyWorkspaceIssueQueueState.js`.
- Added `frontend/src/hooks/strategyWorkspaceIssueQueueState.test.js`.
- Updated `frontend/src/hooks/useStrategyWorkspaceUiState.js` to delegate issue-filter storage scope, read/write, normalization, and summary text to the extracted state module.
- Updated `frontend/src/pages/StrategyWorkspaceIssueQueueCard.jsx` to consume `buildWorkspaceIssueQueueFilterModel(items, filters)` instead of rebuilding filter state in several local memo blocks.

## Preserved Behavior

- Issue queue filters still persist by strategy id, then graph id, then `draft_graph`.
- Invalid or missing persisted filters still fall back to default issue filters.
- Stale source and node-type filters still reset when the active issue queue no longer supports them.
- Diagnostics queue scope text still summarizes severity, actionable-only, source, and node-type filters.
- Issue queue cards still show the same counts, source lanes, node-type lanes, filtered items, and reset behavior.

## Public Inputs

- Strategy id and graph id for storage scoping.
- Workspace issue queue items.
- Current filter patch or persisted filter payload.
- Severity, actionable, source, and node-type filter values.

## Public Outputs

- `workspaceIssueFiltersStorageScope(strategyId, graphId)`.
- `readStoredWorkspaceIssueFilters(scope)`.
- `persistWorkspaceIssueFilters(scope, filters)`.
- `normalizeWorkspaceIssueFilters(filters, items)`.
- `workspaceIssueFiltersSummary(filters)`.
- `buildWorkspaceIssueQueueFilterModel(items, filters)`.
- `WORKSPACE_ISSUE_FILTERS_STORAGE_KEY`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/hooks/strategyWorkspaceIssueQueueState.test.js src/utils/strategyWorkspaceIssueQueue.test.js src/pages/StrategyWorkspacePage.codeMode.test.jsx`: passed, 3 test files and 15 tests.

## Further-Split Decision

`frontend.strategy_workspace.issue_queue_state` does not need a deeper split yet. The pure state and filter model boundary is now separated from the React hook and presentational card; the lower-level issue queue builder remains in `frontend/src/utils/strategyWorkspaceIssueQueue.js`.

## Residuals

- Continue with `frontend.strategy_workspace.workspace_toolbar_bridge`.
