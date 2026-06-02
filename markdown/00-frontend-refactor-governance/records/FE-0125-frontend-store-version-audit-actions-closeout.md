# FE-0125 - Frontend Store Version Audit Actions Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Active child parent: `frontend.store.persistence_startup`
- Closed subchild leaf: `frontend.store.persistence_startup.version_audit_actions`
- Public surface:
  - `frontend/src/store/graphStorePersistenceActions.js`
  - `frontend/src/store/graphStoreVersionAuditActions.js`

## Extraction

- Added `frontend/src/store/graphStoreVersionAuditActions.js`.
- Moved graph version and audit public methods out of `graphStorePersistenceActions.js`:
  - `refreshGraphVersions()`
  - `refreshGraphAuditHistory()`
  - `loadGraphVersionPreview()`
  - `clearGraphVersionPreview()`
  - `compareGraphVersions()`
  - `clearGraphVersionCompare()`
  - `restoreGraphVersion()`
- Reduced `graphStorePersistenceActions.js` to a parent aggregator over graph lifecycle and version/audit subleaves.

## Whitebox Contract

- Inputs:
  - Store `set` and `get` from the `graphStore.js` Zustand root through `graphStorePersistenceActions.js`.
  - Graph version transport, graph audit transport, compare normalization, actor metadata, and graph load helpers.
- Outputs:
  - Version list, version preview, version compare, audit history, and restore state transitions.
  - Restore flow delegates graph refresh/load and clear operations through the parent store API.
- Parent communication:
  - `graphStorePersistenceActions.js` remains the only persistence action aggregator imported by the store root.
  - Version/audit actions communicate through root `set/get` only.
  - Version/audit actions do not import graph lifecycle actions directly; restore calls parent methods through `get()`.

## Preserved Behavior

- Version refresh still clears preview, compare, and audit state for draft graphs.
- Version preview still does not overwrite the working draft.
- Restore still refreshes graph index, reloads the active graph, refreshes versions/audit, and clears preview/compare state.
- Compare still preserves structured strategy config and evidence diff payloads.

## Further Split Decision

- `frontend.store.persistence_startup` remains active.
- Remaining queued subchild:
  - `frontend.store.persistence_startup.persistence_helper_contract`

## Verification

- Targeted Vitest:
  - `npm.cmd test -- --run src/store/graphStore.versionHistory.test.js src/store/graphStore.saveGraphRollback.test.js src/store/graphStore.detailLoadErrors.test.js src/store/graphStorePersistenceConsistency.test.js`
  - Result: passed, 4 files / 9 tests.
- Full feature tree gate: passed.
- Recursive state JSON parse: passed.
- Frontend build: passed.
- `git diff --check`: passed.
