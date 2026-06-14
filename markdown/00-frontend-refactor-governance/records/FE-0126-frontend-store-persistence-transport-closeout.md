# FE-0126 - Frontend Store Persistence Transport Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Active child parent: `frontend.store.persistence_startup`
- Active nested child parent: `frontend.store.persistence_startup.persistence_helper_contract`
- Closed subchild leaf: `frontend.store.persistence_startup.persistence_helper_contract.transport_contract`
- Public surface:
  - `frontend/src/store/graphStorePersistenceHelpers.js`
  - `frontend/src/store/graphStorePersistenceTransport.js`

## Extraction

- Added `frontend/src/store/graphStorePersistenceTransport.js`.
- Moved transport helpers out of `graphStorePersistenceHelpers.js`:
  - `API_BASE`
  - `fetchJson()`
  - `unwrapPage()`
  - `postJson()`
  - `deleteJson()`
- Updated `graphStorePersistenceHelpers.js` to import `fetchJson()` for internal detail resolution and re-export the transport API for compatibility.

## Whitebox Contract

- Inputs:
  - Shared frontend API base URL from `frontend/src/api/client.js`.
  - Timeout-aware fetch helper from `frontend/src/utils/api.js`.
  - Humanized error formatting from `frontend/src/utils/errorText.js`.
- Outputs:
  - JSON GET/POST/DELETE transport primitives.
  - Paginated response unwrapping.
  - Structured failure metadata for POST and DELETE errors.
- Parent communication:
  - `graphStorePersistenceHelpers.js` remains the compatibility export surface.
  - Store action leaves can keep importing through `graphStoreHelpers` or `graphStorePersistenceHelpers`.
  - Transport does not import store state, graph normalization, runtime, editor, or page modules.

## Preserved Behavior

- `fetchJson()` still throws humanized status errors.
- `postJson()` still carries status, error, details, and partial artifact metadata on failures.
- `deleteJson()` still carries status, error, and details metadata on failures.
- `unwrapPage()` still accepts the `{ data, total }` page shape and otherwise returns the original payload.

## Further Split Decision

- `frontend.store.persistence_startup.persistence_helper_contract` remains active.
- Remaining queued subchildren:
  - `frontend.store.persistence_startup.persistence_helper_contract.storage_cache_contract`
  - `frontend.store.persistence_startup.persistence_helper_contract.graph_shape_validation_contract`
  - `frontend.store.persistence_startup.persistence_helper_contract.version_audit_normalizers`
  - `frontend.store.persistence_startup.persistence_helper_contract.actor_collaboration_contract`

## Verification

- Targeted Vitest:
  - `npm.cmd test -- --run src/api/apiTransport.test.js src/api/fetchHelpers.test.js src/store/graphStore.versionHistory.test.js src/store/graphStore.saveGraphRollback.test.js src/store/graphStoreCapabilityRefresh.test.js src/store/graphStore.startupRecovery.test.js`
  - Result: passed, 6 files / 18 tests.
- Full feature tree gate: passed.
- Recursive state JSON parse: passed.
- Frontend build: passed.
- `git diff --check`: passed.
