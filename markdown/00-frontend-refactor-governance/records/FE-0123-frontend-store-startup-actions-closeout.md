# FE-0123 - Frontend Store Startup Actions Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Active child parent: `frontend.store.persistence_startup`
- Closed subchild leaf: `frontend.store.persistence_startup.startup_actions`
- Public surface:
  - `frontend/src/store/graphStore.js`
  - `frontend/src/store/graphStoreStartupActions.js`

## Extraction

- Added `frontend/src/store/graphStoreStartupActions.js`.
- Moved `initialize()`, `refreshGraphIndex()`, and `recoverLatestRunnableGraph()` out of `graphStore.js`.
- Updated `graphStore.js` to consume `createGraphStoreStartupActions(set, get)`.

## Whitebox Contract

- Inputs:
  - Store `set` and `get` from the `graphStore.js` Zustand root.
  - Capability refresh, graph version/audit refresh, runtime sidebar warmup, and registry state exposed by the parent store.
  - Startup and persistence helpers from `graphStoreHelpers`.
- Outputs:
  - Startup graph resolution and storage synchronization.
  - Graph index state transitions.
  - Latest runnable graph recovery state transitions.
  - Runtime backend error projection for startup failures.
- Parent communication:
  - `graphStore.js` remains the only store root and action aggregation owner.
  - Startup actions communicate through the root `set/get` interface only.
  - Startup actions do not import editor, runtime, compile, or page modules directly.

## Preserved Behavior

- Startup recovery still prefers runnable latest graph when present in the backend graph index.
- Stored runnable graphs still recover when backend latest fails.
- Stored runnable graphs still remain acceptable as the third fallback even when missing from the backend index.
- Non-runnable latest graph recovery errors still use the existing action-failure wording.

## Further Split Decision

- `frontend.store.persistence_startup` remains worth splitting.
- Remaining queued subchildren:
  - `frontend.store.persistence_startup.graph_lifecycle_actions`
  - `frontend.store.persistence_startup.version_audit_actions`
  - `frontend.store.persistence_startup.persistence_helper_contract`

## Verification

- Targeted Vitest:
  - `npm.cmd test -- --run src/store/graphStore.startupRecovery.test.js src/store/graphStorePersistenceConsistency.test.js src/store/graphStore.capabilities.test.js src/store/graphStoreRootState.test.js`
  - Result: passed, 4 files / 9 tests.
- Full feature tree gate: passed.
- Recursive state JSON parse: passed.
- Frontend build: passed.
- `git diff --check`: passed.
