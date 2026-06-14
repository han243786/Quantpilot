# FE-0124 - Frontend Store Graph Lifecycle Actions Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Active child parent: `frontend.store.persistence_startup`
- Closed subchild leaf: `frontend.store.persistence_startup.graph_lifecycle_actions`
- Public surface:
  - `frontend/src/store/graphStorePersistenceActions.js`
  - `frontend/src/store/graphStoreGraphLifecycleActions.js`

## Extraction

- Added `frontend/src/store/graphStoreGraphLifecycleActions.js`.
- Moved graph lifecycle public methods out of `graphStorePersistenceActions.js`:
  - `resetGraph()`
  - `saveGraph()`
  - `loadLatestGraph()`
  - `deleteGraph()`
  - `loadGraphById()`
  - `importStrategyPackage()`
  - `revealGraphFile()`
- Updated `graphStorePersistenceActions.js` to aggregate graph lifecycle actions and retain version/audit actions for the next leaf.

## Whitebox Contract

- Inputs:
  - Store `set` and `get` from the `graphStore.js` Zustand root through `graphStorePersistenceActions.js`.
  - Compile, validation, graph storage, graph CRUD transport, and actor metadata helpers.
- Outputs:
  - Graph reset, save, load, delete, import, reveal, and rollback state transitions.
  - Local storage synchronization for current graph state.
  - Version/audit refresh calls delegated back through the parent store API.
- Parent communication:
  - `graphStorePersistenceActions.js` remains the persistence parent aggregator.
  - Lifecycle actions communicate through root `set/get` and do not import startup, runtime session, editor, or page modules directly.

## Preserved Behavior

- Save rollback still restores graph, compile result, version preview, version compare, quant script draft, and strategy IR draft.
- Loading latest or by id still stops a running runtime before graph replacement.
- Import still creates an `imported_` graph id and clears compare selection.
- Delete still resets the current graph when deleting the active graph id.

## Further Split Decision

- `frontend.store.persistence_startup` remains active.
- Remaining queued subchildren:
  - `frontend.store.persistence_startup.version_audit_actions`
  - `frontend.store.persistence_startup.persistence_helper_contract`

## Verification

- Targeted Vitest:
  - `npm.cmd test -- --run src/store/graphStore.saveGraphRollback.test.js src/store/graphStore.versionHistory.test.js src/store/graphStore.detailLoadErrors.test.js src/store/graphStore.export.test.js src/store/graphStorePersistenceConsistency.test.js`
  - Result: passed, 5 files / 10 tests.
- Full feature tree gate: passed.
- Recursive state JSON parse: passed.
- Frontend build: passed.
- `git diff --check`: passed.
