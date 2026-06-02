# FE-0127 - Frontend Store Persistence Storage Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Active child parent: `frontend.store.persistence_startup`
- Active nested child parent: `frontend.store.persistence_startup.persistence_helper_contract`
- Closed subchild leaf: `frontend.store.persistence_startup.persistence_helper_contract.storage_cache_contract`
- Public surface:
  - `frontend/src/store/graphStorePersistenceHelpers.js`
  - `frontend/src/store/graphStorePersistenceStorage.js`

## Extraction

- Added `frontend/src/store/graphStorePersistenceStorage.js`.
- Moved local storage and cache helpers out of `graphStorePersistenceHelpers.js`:
  - `STORAGE_KEY`
  - `CAPABILITY_CACHE_KEY`
  - `saveGraphToStorage()`
  - `loadGraphFromStorage()`
  - `saveCapabilitiesToCache()`
  - `loadCapabilitiesFromCache()`
- Kept `safeGetItem()` and `safeSetItem()` private inside the storage leaf.
- Updated `graphStorePersistenceHelpers.js` to re-export the storage/cache API for compatibility.

## Whitebox Contract

- Inputs:
  - Browser `window.localStorage` when available.
  - Optional `navigator.storage.estimate()` quota metadata.
  - Graph and capability payloads supplied by parent store actions.
- Outputs:
  - Serialized graph cache under `STORAGE_KEY`.
  - Serialized capability cache under `CAPABILITY_CACHE_KEY`.
  - `null` fallback for unavailable storage, missing cache, parse failure, or schema mismatch.
  - `qp-storage-quota-exceeded` browser event on localStorage quota failures.
- Parent communication:
  - Store action leaves continue importing through `graphStoreHelpers` or `graphStorePersistenceHelpers`.
  - Storage/cache does not import graph normalization, runtime, editor, API transport, or page modules.

## Preserved Behavior

- Graph cache still writes `_schema: 1` with the graph payload.
- Graph cache schema mismatch still drops the cached graph.
- Capability cache still accepts only parsed object payloads.
- Storage access remains no-op/null-safe when `window` is unavailable.
- Quota warnings and quota-exceeded event behavior remain local to the storage leaf.

## Further Split Decision

- `frontend.store.persistence_startup.persistence_helper_contract` remains active.
- Remaining queued subchildren:
  - `frontend.store.persistence_startup.persistence_helper_contract.graph_shape_validation_contract`
  - `frontend.store.persistence_startup.persistence_helper_contract.version_audit_normalizers`
  - `frontend.store.persistence_startup.persistence_helper_contract.actor_collaboration_contract`

## Verification

- Targeted Vitest:
  - `npm.cmd test -- --run src/store/graphStore.startupRecovery.test.js src/store/graphStoreCapabilityRefresh.test.js src/store/graphStore.saveGraphRollback.test.js src/store/graphStore.versionHistory.test.js src/store/graphStorePersistenceConsistency.test.js`
  - Result: passed, 5 files / 13 tests.
- Full feature tree gate: passed.
- Recursive state JSON parse: passed.
- Frontend build: passed.
- `git diff --check`: passed.
