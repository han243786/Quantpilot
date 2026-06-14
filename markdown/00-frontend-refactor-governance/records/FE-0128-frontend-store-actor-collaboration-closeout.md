# FE-0128 - Frontend Store Actor Collaboration Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Active child parent: `frontend.store.persistence_startup`
- Active nested child parent: `frontend.store.persistence_startup.persistence_helper_contract`
- Closed subchild leaf: `frontend.store.persistence_startup.persistence_helper_contract.actor_collaboration_contract`
- Public surface:
  - `frontend/src/store/graphStorePersistenceHelpers.js`
  - `frontend/src/store/graphStoreActorCollaboration.js`

## Extraction

- Added `frontend/src/store/graphStoreActorCollaboration.js`.
- Moved actor/collaboration helpers out of `graphStorePersistenceHelpers.js`:
  - `DEFAULT_LOCAL_ACTOR`
  - `normalizeActorIdentity()`
  - `normalizeCollaborationMetadata()`
  - `resolveGraphActor()`
  - `withGraphActorMetadata()`
- Updated `graphStorePersistenceHelpers.js` to import the actor normalizers used by graph shape and audit normalization.
- Re-exported the existing actor-facing public helpers from `graphStorePersistenceHelpers.js` for compatibility.

## Ordering Note

- This leaf was closed before `graph_shape_validation_contract`.
- Reason: `normalizeGraphShape()` depends on `normalizeCollaborationMetadata()`. Extracting actor/collaboration first avoids a reverse dependency from the future graph-shape leaf back into the aggregate helper.

## Whitebox Contract

- Inputs:
  - Graph metadata collaboration payloads.
  - Optional actor identity payloads from runtime, save, or audit flows.
  - Text sanitizer from `frontend/src/utils/errorText.js`.
- Outputs:
  - Normalized actor identities.
  - Normalized collaboration metadata.
  - Resolved graph actor fallback.
  - Graph metadata with owner/last_saved_by actor fields applied.
- Parent communication:
  - Store actions continue importing `resolveGraphActor()` and `withGraphActorMetadata()` through `graphStoreHelpers`.
  - Graph shape and audit normalizers can import actor helpers directly without touching API transport, storage, runtime, editor, or page modules.

## Preserved Behavior

- Missing actor still falls back to `local_operator`.
- Owner remains the first priority actor, then first editor, then local fallback.
- `withGraphActorMetadata()` still preserves existing metadata and fills owner when absent.
- Audit normalization can keep using the same local actor fallback.

## Further Split Decision

- `frontend.store.persistence_startup.persistence_helper_contract` remains active.
- Remaining queued subchildren:
  - `frontend.store.persistence_startup.persistence_helper_contract.graph_shape_validation_contract`
  - `frontend.store.persistence_startup.persistence_helper_contract.version_audit_normalizers`

## Verification

- Targeted Vitest:
  - `npm.cmd test -- --run src/store/graphStore.versionHistory.test.js src/store/graphStore.startupRecovery.test.js src/store/graphStore.saveGraphRollback.test.js src/store/graphStore.runtimeErrors.test.js src/store/graphStoreCapabilityRefresh.test.js src/store/graphStorePersistenceConsistency.test.js`
  - Result: passed, 6 files / 21 tests.
- Full feature tree gate: passed.
- Recursive state JSON parse: passed.
- Frontend build: passed.
- `git diff --check`: passed.
