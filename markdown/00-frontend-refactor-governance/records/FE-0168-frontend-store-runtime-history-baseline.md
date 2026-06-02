# FE-0168 Frontend Store Runtime History Baseline

Status: baseline established.

## Scope

- Parent node: `frontend.store`
- Active child parent: `frontend.store.runtime_history`
- This is a docs-only recursive baseline for runtime history extraction.

## Owned Files

- `frontend/src/store/graphStoreRuntimeHistoryActions.js`
- `frontend/src/store/graphStoreRuntimeHistoryFlow.js`
- `frontend/src/store/graphStoreRuntimeHistoryApi.js`
- `frontend/src/store/graphStoreRuntimeHistoryProjection.js`
- `frontend/src/store/graphStoreRuntimeHistoryState.js`
- `frontend/src/store/graphStoreRuntimeHistoryFlow.test.js`
- `frontend/src/store/graphStore.backtestArtifacts.test.js`
- `frontend/src/store/graphStoreRuntimeSelectionState.test.js`
- `frontend/src/components/BacktestHistorySection.jsx`
- `frontend/src/components/BacktestHistorySection.test.jsx`
- `frontend/src/components/RunHistorySection.jsx`
- `frontend/src/pages/StrategyWorkspaceExperimentCard.jsx`
- `frontend/src/pages/StrategyWorkspaceExperimentCard.test.jsx`

## Whitebox Boundary

- Inputs:
  - Runtime history/backtest/experiment list endpoints.
  - Runtime detail/backtest detail/experiment detail endpoints.
  - Save and discard endpoints for runtime artifacts.
  - Current graph, registry, runtime selection state, compare selection, and active transient artifact state.
- Processing:
  - Warm sidebar data by loading missing run, backtest, and experiment histories.
  - Refresh run, backtest, and experiment history lists.
  - Toggle, clear, and replace backtest compare selection.
  - Load run/backtest/experiment detail and project selected runtime graph state.
  - Save or discard current transient runtime/backtest/experiment artifact.
  - Normalize API failures into runtime backend errors.
- Outputs:
  - Runtime history lists, selected run/backtest/experiment detail state, graph runtime binding, highlighted nodes, compare selection, artifact persistence state, and backend error state.
- Parent communication:
  - Public methods are exposed through `graphStore.js` via `createGraphStoreRuntimeHistoryActions`.
  - Runtime history children must communicate through `graphStoreRuntimeHistoryActions.js` or the `frontend.store` parent.

## Recursive Child Queue

- `frontend.store.runtime_history.compare_selection_state`
- `frontend.store.runtime_history.history_refresh_flow`
- `frontend.store.runtime_history.detail_selection_flow`
- `frontend.store.runtime_history.artifact_persistence_flow`
- `frontend.store.runtime_history.api_projection_state_contract`

## Split Decision

- This parent is worth recursive split.
- Hard-rule assessment:
  - The action surface exposes unrelated public contracts: compare selection, list refresh, detail loading, and artifact persistence.
  - `graphStoreRuntimeHistoryFlow.js` mixes list refresh, detail graph projection, save/discard persistence, and failure normalization.
  - Detail flows touch graph projection and storage, while list refresh flows only update list/status state.
  - Artifact persistence changes runtime reset/discard behavior and should stay separate from list/detail fetching.
  - API/projection/state files are already separated but need a final white-box contract closeout after flow extraction.
  - Existing runtime history, backtest artifact, and UI tests provide focused verification gates.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
