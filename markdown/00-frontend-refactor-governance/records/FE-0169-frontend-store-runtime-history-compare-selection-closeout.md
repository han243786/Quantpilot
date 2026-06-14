# FE-0169 Frontend Store Runtime History Compare Selection Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.runtime_history`
- Closed leaf: `frontend.store.runtime_history.compare_selection_state`
- Code surfaces:
  - `frontend/src/store/graphStoreRuntimeHistoryCompareSelection.js`
  - `frontend/src/store/graphStoreRuntimeHistoryActions.js`
  - `frontend/src/store/graphStoreRuntimeHistoryState.js`
  - `frontend/src/store/graphStoreRuntimeHistoryFlow.test.js`
  - `frontend/src/components/BacktestHistorySection.test.jsx`

## Change

- Extracted strategy-scoped backtest compare selection helpers into `graphStoreRuntimeHistoryCompareSelection.js`.
- Kept `graphStoreRuntimeHistoryActions.js` as the public action composer for toggle, clear, and replace operations.
- Kept `buildBacktestHistoryReadyState` responsible for pruning compare selections against the refreshed backtest list.

## Whitebox Boundary

- Inputs:
  - Current graph metadata id, runtime compare selection state, and candidate backtest ids.
- Processing:
  - Resolve the strategy-scoped compare-selection key.
  - Sanitize compare selection to unique non-empty ids with a maximum of two entries.
  - Toggle, clear, replace, and prune selections without changing unrelated strategy selections.
- Outputs:
  - Updated `runtime.backtestCompareSelection` map.
  - Stable public compare-selection behavior for runtime history actions and UI consumers.
- Parent communication:
  - `graphStoreRuntimeHistoryActions.js` imports this leaf and exposes public store actions.
  - `graphStoreRuntimeHistoryState.js` imports this leaf only to prune selections during backtest history ready-state projection.
  - Runtime history children must not call each other for compare behavior.

## Recursive Split Decision

- No further split is required now.
- The leaf is cohesive around one compare-selection state contract and contains no API, persistence, or graph projection behavior.
- Continue the parent queue through `frontend.store.runtime_history.history_refresh_flow`.

## Equivalence Baseline

- Compare selection still falls back to legacy array state when present.
- Compare selection remains strategy scoped by `graph.metadata.graph_id`, with `_global` fallback.
- Selection still toggles selected ids, refuses a third active id, clears to an empty list, and replaces with at most two sanitized ids.
- Backtest history refresh still removes selected ids that no longer exist in the refreshed history list while preserving other strategy selections.

## Verification

- `npm.cmd test -- --run src/store/graphStoreRuntimeHistoryFlow.test.js src/components/BacktestHistorySection.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
