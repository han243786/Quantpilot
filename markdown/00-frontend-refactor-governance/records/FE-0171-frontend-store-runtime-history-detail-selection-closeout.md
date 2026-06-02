# FE-0171 Frontend Store Runtime History Detail Selection Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.runtime_history`
- Closed leaf: `frontend.store.runtime_history.detail_selection_flow`
- Code surfaces:
  - `frontend/src/store/graphStoreRuntimeHistoryDetailFlow.js`
  - `frontend/src/store/graphStoreRuntimeHistoryFlow.js`
  - `frontend/src/store/graphStoreRuntimeHistoryApi.js`
  - `frontend/src/store/graphStoreRuntimeHistoryProjection.js`
  - `frontend/src/store/graphStoreRuntimeHistoryState.js`
  - `frontend/src/store/graphStoreRuntimeHistoryFlow.test.js`
  - `frontend/src/store/graphStore.backtestArtifacts.test.js`
  - `frontend/src/store/graphStore.detailLoadErrors.test.js`
  - `frontend/src/pages/BacktestDetailPage.test.jsx`
  - `frontend/src/pages/StrategyWorkspaceExperimentCard.test.jsx`

## Change

- Extracted run, backtest, and experiment detail loading into `graphStoreRuntimeHistoryDetailFlow.js`.
- Kept graph resolution, graph projection, storage persistence, and selected runtime state projection inside the detail leaf.
- Kept `graphStoreRuntimeHistoryFlow.js` as a compatibility composer for save flows and existing public imports.

## Whitebox Boundary

- Inputs:
  - Run/backtest/experiment detail ids, current graph, registry, detail APIs, mutation API, and graph projection helpers.
- Processing:
  - Load detail records and parameter mutations where applicable.
  - Resolve the source graph for detail replay.
  - Project run/backtest detail onto graph runtime state and persist the projected graph.
  - Normalize experiment detail before selected experiment projection.
  - Convert detail-load failures into runtime history error states.
- Outputs:
  - Selected run, backtest, or experiment state.
  - Updated graph runtime binding and highlighted node ids for run/backtest detail.
  - Stable `null` return on detail-load failure.
- Parent communication:
  - `graphStoreRuntimeHistoryActions.js` reaches this leaf through the parent flow facade.
  - Save flows call this leaf after refreshing the relevant history list.
  - This leaf may call shared projection/state/failure helpers, but must not call refresh or persistence children directly.

## Recursive Split Decision

- No further split is required now.
- Run and backtest detail share graph resolution/projection/storage semantics, while experiment detail is small and part of the same selected-detail contract.
- The projection helpers and state projectors remain separate helper surfaces for the final API/projection/state contract closeout.
- Continue the parent queue through `frontend.store.runtime_history.artifact_persistence_flow`.

## Equivalence Baseline

- Run detail still sets selected-run loading status, loads parameter mutations, projects the detail graph, stores the graph, then sets ready status.
- Backtest detail still loads backtest detail, projects events/highlights, stores the graph, and returns the detail.
- Experiment detail still sets selected experiment loading state, normalizes the response, and projects ready/error state.
- Detail failures still set backend error state and return `null`.
- Save flows still reload persisted detail through the same public function names.

## Verification

- `npm.cmd test -- --run src/store/graphStoreRuntimeHistoryFlow.test.js src/store/graphStore.backtestArtifacts.test.js src/store/graphStore.detailLoadErrors.test.js src/pages/BacktestDetailPage.test.jsx src/pages/StrategyWorkspaceExperimentCard.test.jsx`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
