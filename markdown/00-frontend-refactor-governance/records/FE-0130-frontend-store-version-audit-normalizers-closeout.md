# FE-0130 - Frontend Store Version Audit Normalizers Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Active child parent: `frontend.store.persistence_startup`
- Active nested child parent: `frontend.store.persistence_startup.persistence_helper_contract`
- Closed subchild leaf: `frontend.store.persistence_startup.persistence_helper_contract.version_audit_normalizers`
- Public surface:
  - `frontend/src/store/graphStorePersistenceHelpers.js`
  - `frontend/src/store/graphStoreVersionAuditNormalizers.js`

## Extraction

- Added `frontend/src/store/graphStoreVersionAuditNormalizers.js`.
- Moved version, audit, graph index, and compare normalizers out of `graphStorePersistenceHelpers.js`:
  - `normalizeGraphIndex()`
  - `normalizeGraphAuditHistory()`
  - `normalizeGraphVersions()`
  - `normalizeGraphVersionCompare()`
  - `graphExistsInIndex()`
- Kept `resolveGraphForDetail()` in `graphStorePersistenceHelpers.js` because it composes API transport with loaded-graph resolution.
- Updated `graphStorePersistenceHelpers.js` to re-export the version/audit normalizer API for compatibility.

## Whitebox Contract

- Inputs:
  - Raw graph index, audit history, graph version, and graph-version compare API payloads.
  - Actor normalizer from `frontend/src/store/graphStoreActorCollaboration.js`.
  - Display text sanitizer from `frontend/src/utils/errorText.js`.
- Outputs:
  - Sanitized graph index rows.
  - Sanitized audit history rows with normalized actor identity.
  - Sanitized graph version rows.
  - Sanitized version compare payloads, including metadata, node, edge, config, strategy config, and evidence diffs.
  - Graph-index existence check for startup recovery.
- Parent communication:
  - Store actions continue importing through `graphStoreHelpers`.
  - The normalizer leaf does not import storage, API transport, graph validation, runtime actions, editor actions, or page modules.

## Preserved Behavior

- Invalid list payloads still normalize to empty arrays.
- Invalid compare payload still normalizes to `null`.
- Index and version rows are still filtered by required ids.
- Actor data in audit history still uses the local fallback actor.
- Strategy config evidence compare fields keep the same fallback statuses and numeric defaults.

## Further Split Decision

- `frontend.store.persistence_startup.persistence_helper_contract` has no remaining queued subchildren.
- Next recursive step: parent closeout for `frontend.store.persistence_startup.persistence_helper_contract`, then `frontend.store.persistence_startup`.

## Verification

- Targeted Vitest:
  - `npm.cmd test -- --run src/store/graphStore.versionHistory.test.js src/store/graphStore.startupRecovery.test.js src/store/graphStorePersistenceConsistency.test.js src/store/graphStore.saveGraphRollback.test.js src/store/graphStoreCapabilityRefresh.test.js`
  - Result: passed, 5 files / 13 tests.
- Full feature tree gate: passed.
- Recursive state JSON parse: passed.
- Frontend build: passed.
- `git diff --check`: passed.
