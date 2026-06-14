# FE-0174 Frontend Store Runtime History Parent Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Closed child parent: `frontend.store.runtime_history`
- This is a docs-only parent closeout. No frontend source files changed in this step.

## Closed Leaves

- `frontend.store.runtime_history.compare_selection_state`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0169-frontend-store-runtime-history-compare-selection-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreRuntimeHistoryCompareSelection.js`
    - `frontend/src/store/graphStoreRuntimeHistoryActions.js`
    - `frontend/src/store/graphStoreRuntimeHistoryState.js`
- `frontend.store.runtime_history.history_refresh_flow`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0170-frontend-store-runtime-history-refresh-flow-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreRuntimeHistoryRefreshFlow.js`
    - `frontend/src/store/graphStoreRuntimeHistoryFailure.js`
    - `frontend/src/store/graphStoreRuntimeHistoryFlow.js`
- `frontend.store.runtime_history.detail_selection_flow`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0171-frontend-store-runtime-history-detail-selection-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreRuntimeHistoryDetailFlow.js`
    - `frontend/src/store/graphStoreRuntimeHistoryProjection.js`
    - `frontend/src/store/graphStoreRuntimeHistoryFlow.js`
- `frontend.store.runtime_history.artifact_persistence_flow`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0172-frontend-store-runtime-history-artifact-persistence-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreRuntimeHistoryArtifactFlow.js`
    - `frontend/src/store/graphStoreRuntimeHistoryFlow.js`
    - `frontend/src/store/graphStoreRuntimeHistoryActions.js`
- `frontend.store.runtime_history.api_projection_state_contract`
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0173-frontend-store-runtime-history-api-projection-state-contract-closeout.md`
  - Public surfaces:
    - `frontend/src/store/graphStoreRuntimeHistoryApi.js`
    - `frontend/src/store/graphStoreRuntimeHistoryProjection.js`
    - `frontend/src/store/graphStoreRuntimeHistoryState.js`
    - `frontend/src/store/graphStoreRuntimeHistoryFlow.js`

## Public Surfaces Preserved

- `frontend/src/store/graphStoreRuntimeHistoryActions.js`
- `frontend/src/store/graphStoreRuntimeHistoryFlow.js`
- `frontend/src/store/graphStoreRuntimeHistoryApi.js`
- `frontend/src/store/graphStoreRuntimeHistoryProjection.js`
- `frontend/src/store/graphStoreRuntimeHistoryState.js`
- `frontend/src/store/graphStoreRuntimeHistoryFailure.js`
- `frontend/src/store/graphStoreRuntimeHistoryCompareSelection.js`
- `frontend/src/store/graphStoreRuntimeHistoryRefreshFlow.js`
- `frontend/src/store/graphStoreRuntimeHistoryDetailFlow.js`
- `frontend/src/store/graphStoreRuntimeHistoryArtifactFlow.js`

## Recursive Decision

- The runtime history child queue is closed.
- Runtime history is now split into compare selection, refresh, detail, artifact persistence, and API/projection/state contracts.
- `graphStoreRuntimeHistoryActions.js` remains the parent public store action composer.
- `graphStoreRuntimeHistoryFlow.js` remains a compatibility facade for already-known imports.
- No runtime history child needs another recursive split at this stage.
- The parent returns control to `frontend.store`.
- Next queued store child: `frontend.store.runtime_transport_selection`.

## Equivalence Evidence

- FE-0169 through FE-0172 each landed with targeted tests, build verification, and full frontend pre-commit verification.
- FE-0173 closed the helper contract after docs-only governance gates.
- This parent closeout only changes frontend-local governance files and records the already-verified child queue completion.

## Verification

- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
