# FE-0170 Frontend Store Runtime History Refresh Flow Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.runtime_history`
- Closed leaf: `frontend.store.runtime_history.history_refresh_flow`
- Code surfaces:
  - `frontend/src/store/graphStoreRuntimeHistoryRefreshFlow.js`
  - `frontend/src/store/graphStoreRuntimeHistoryFailure.js`
  - `frontend/src/store/graphStoreRuntimeHistoryFlow.js`
  - `frontend/src/store/graphStoreRuntimeHistoryApi.js`
  - `frontend/src/store/graphStoreRuntimeHistoryState.js`
  - `frontend/src/store/graphStoreRuntimeHistoryFlow.test.js`
  - `frontend/src/components/RunHistorySection.test.jsx`
  - `frontend/src/components/BacktestHistorySection.test.jsx`
  - `frontend/src/pages/StrategyWorkspaceExperimentCard.test.jsx`

## Change

- Extracted runtime sidebar warmup and run/backtest/experiment list refresh into `graphStoreRuntimeHistoryRefreshFlow.js`.
- Extracted shared runtime history failure-message formatting into `graphStoreRuntimeHistoryFailure.js`.
- Kept `graphStoreRuntimeHistoryFlow.js` as a compatibility composer for existing imports and for save/detail/discard flows that still reuse refresh helpers.

## Whitebox Boundary

- Inputs:
  - Runtime list readiness state and the run/backtest/experiment history APIs.
  - Experiment list normalization and history list state projectors.
- Processing:
  - Warm only missing sidebar datasets while skipping lists already ready or loading.
  - Load run, backtest, and experiment history lists.
  - Normalize experiment list responses before state projection.
  - Convert refresh failures into runtime history error states through the shared failure formatter.
- Outputs:
  - Updated runtime history lists and status fields.
  - Empty arrays on refresh failure, matching the prior public contract.
  - Stable backend error text with API reason preservation.
- Parent communication:
  - `graphStoreRuntimeHistoryActions.js` continues to call the parent flow facade.
  - `graphStoreRuntimeHistoryFlow.js` imports this leaf for save-after-refresh behavior.
  - Detail and artifact persistence children may reuse refresh helpers only through this leaf or the parent flow facade.

## Recursive Split Decision

- No further split is required now.
- Run, backtest, and experiment refreshes share one list-loading protocol and are small enough to stay together.
- Failure formatting is shared by refresh, detail, save, and discard flows, so it is a helper surface rather than its own recursive child.
- Continue the parent queue through `frontend.store.runtime_history.detail_selection_flow`.

## Equivalence Baseline

- `warmRuntimeSidebarDataFlow` still calls only missing refresh actions and returns `[]` when no sidebar dataset needs loading.
- `refreshRunHistoryFlow`, `refreshBacktestHistoryFlow`, and `refreshExperimentHistoryFlow` still set loading state, call the same APIs, project ready state, and return list data.
- Refresh failures still project the matching history error state and return `[]`.
- Save flows still refresh history before reloading persisted detail.
- Existing imports from `graphStoreRuntimeHistoryFlow.js` still work through re-exports.

## Verification

- `npm.cmd test -- --run src/store/graphStoreRuntimeHistoryFlow.test.js src/components/RunHistorySection.test.jsx src/components/BacktestHistorySection.test.jsx src/pages/StrategyWorkspaceExperimentCard.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
