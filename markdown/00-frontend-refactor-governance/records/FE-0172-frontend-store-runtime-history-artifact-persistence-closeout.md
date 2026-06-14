# FE-0172 Frontend Store Runtime History Artifact Persistence Closeout

Status: closed.

## Scope

- Parent node: `frontend.store.runtime_history`
- Closed leaf: `frontend.store.runtime_history.artifact_persistence_flow`
- Code surfaces:
  - `frontend/src/store/graphStoreRuntimeHistoryArtifactFlow.js`
  - `frontend/src/store/graphStoreRuntimeHistoryFlow.js`
  - `frontend/src/store/graphStoreRuntimeHistoryApi.js`
  - `frontend/src/store/graphStoreRuntimeHistoryRefreshFlow.js`
  - `frontend/src/store/graphStoreRuntimeHistoryDetailFlow.js`
  - `frontend/src/store/graphStoreRuntimeHistoryState.js`
  - `frontend/src/store/graphStoreRuntimeHistoryFlow.test.js`
  - `frontend/src/components/EventStreamPanel.runtimeArtifactActions.test.jsx`
  - `frontend/src/hooks/useStrategyResearchActions.test.js`

## Change

- Extracted save and discard runtime artifact flows into `graphStoreRuntimeHistoryArtifactFlow.js`.
- Kept save flows responsible for API save, history refresh, and persisted detail reload.
- Kept discard flows responsible for API discard and runtime reset or selected experiment cleanup.
- Reduced `graphStoreRuntimeHistoryFlow.js` to a pure re-export facade for existing imports.

## Whitebox Boundary

- Inputs:
  - Run/backtest/experiment record ids, save/discard APIs, refresh helpers, detail loaders, reset state projector, and shared failure formatter.
- Processing:
  - Save records, refresh the matching history list, then reload persisted detail.
  - Discard run/backtest records and reset runtime state.
  - Discard experiment records and clear selected experiment state.
  - Convert save/discard failures into runtime history backend error state.
- Outputs:
  - Persisted detail data after save.
  - Discard API responses after discard.
  - `null` on save/discard failure.
  - Parent flow facade preserving existing import paths.
- Parent communication:
  - `graphStoreRuntimeHistoryActions.js` continues to choose the current artifact kind from runtime state.
  - This leaf performs the concrete persistence side effects once the parent action selects the target kind.
  - This leaf may call refresh and detail leaves through their public helper functions.

## Recursive Split Decision

- No further split is required now.
- Save and discard both operate on the same artifact persistence protocol and share failure handling.
- Splitting by artifact kind would create three small leaves with repeated API-refresh-detail choreography.
- Continue the parent queue through `frontend.store.runtime_history.api_projection_state_contract`.

## Equivalence Baseline

- Saving a run still calls save, refreshes run history, reloads run detail, and returns the detail.
- Saving a backtest still calls save, refreshes backtest history, reloads backtest detail, and returns the detail.
- Saving an experiment still calls save, refreshes experiment history, reloads experiment detail, and returns the detail.
- Discarding a run or backtest still resets runtime state after the discard API succeeds.
- Discarding an experiment still clears selected experiment state after the discard API succeeds.
- Existing imports from `graphStoreRuntimeHistoryFlow.js` still work through facade re-exports.

## Verification

- `npm.cmd test -- --run src/store/graphStoreRuntimeHistoryFlow.test.js src/components/EventStreamPanel.runtimeArtifactActions.test.jsx src/hooks/useStrategyResearchActions.test.js src/store/graphStore.backtestArtifacts.test.js`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
- `npm.cmd run build`
