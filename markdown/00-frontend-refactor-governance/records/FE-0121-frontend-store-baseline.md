# FE-0121 - Frontend Store Baseline

Status: closed.

## Parent Node

`frontend.store`

## Baseline Scope

- Current owned file count: 57 files under `frontend/src/store`.
- Scope includes the Zustand graph store root, graph persistence helpers/actions, capability refresh, editor action facades, compile actions/flows, runtime session actions/state, runtime history actions/API/projections, runtime transport, runtime selection state, and associated store tests.

## Owned Files

- `frontend/src/store/graphStore.js`
- `frontend/src/store/graphStoreHelpers.js`
- `frontend/src/store/graphStorePersistenceHelpers.js`
- `frontend/src/store/graphStorePersistenceActions.js`
- `frontend/src/store/graphStoreCapabilityRefresh.js`
- `frontend/src/store/graphStoreEditorActions.js`
- `frontend/src/store/graphStoreEditorDraftActions.js`
- `frontend/src/store/graphStoreEditorEdgeActions.js`
- `frontend/src/store/graphStoreEditorNodeActions.js`
- `frontend/src/store/graphStoreEditorSelectionActions.js`
- `frontend/src/store/graphStoreEditorTemplateActions.js`
- `frontend/src/store/graphStoreCompileActions.js`
- `frontend/src/store/graphStoreCompileApi.js`
- `frontend/src/store/graphStoreCompileFlow.js`
- `frontend/src/store/graphStoreCompileHelpers.js`
- `frontend/src/store/graphStoreCompileOutcomeMapping.js`
- `frontend/src/store/graphStoreCompileOutcomeProjection.js`
- `frontend/src/store/graphStoreCompileProtocolFlow.js`
- `frontend/src/store/graphStoreCompileProtocolMapping.js`
- `frontend/src/store/graphStoreCompileState.js`
- `frontend/src/store/graphStoreRuntimeActions.js`
- `frontend/src/store/graphStoreRuntimeHelpers.js`
- `frontend/src/store/graphStoreRuntimeHistoryActions.js`
- `frontend/src/store/graphStoreRuntimeHistoryApi.js`
- `frontend/src/store/graphStoreRuntimeHistoryFlow.js`
- `frontend/src/store/graphStoreRuntimeHistoryProjection.js`
- `frontend/src/store/graphStoreRuntimeHistoryState.js`
- `frontend/src/store/graphStoreRuntimeSelectionState.js`
- `frontend/src/store/graphStoreRuntimeSessionActions.js`
- `frontend/src/store/graphStoreRuntimeSessionState.js`
- `frontend/src/store/graphStoreRuntimeTransport.js`
- Store tests under `frontend/src/store`.

## Important Consumers

- App initialization and global shell code.
- Graph editor canvas, toolbar, module sidebar, and property panels.
- Runtime panels and persisted runtime/backtest history sections.
- Strategy workspace and strategy hub route surfaces.
- Backtest views that consume persisted backtest detail/history state.

## Candidate Child Queue

- `frontend.store.root_shell`
- `frontend.store.persistence_startup`
- `frontend.store.capability_refresh`
- `frontend.store.editor_actions`
- `frontend.store.compile_flow`
- `frontend.store.runtime_session`
- `frontend.store.runtime_history`
- `frontend.store.runtime_transport_selection`

## Boundary Notes

- `frontend.store` is a shared state parent, so extraction must avoid introducing direct child-to-child shortcuts.
- Existing store modules already contain partial splits; the recursive pass should first verify each file family as a whitebox leaf before moving code.
- API transport implementation remains with `frontend.api_client`; store modules may call API facades but should not absorb API transport ownership.
- UI components remain outside this parent; store children should expose state/actions/projections, not view rendering.

## Verification

- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
