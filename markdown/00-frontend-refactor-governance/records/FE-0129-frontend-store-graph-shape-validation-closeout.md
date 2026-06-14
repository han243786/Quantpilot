# FE-0129 - Frontend Store Graph Shape Validation Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Active child parent: `frontend.store.persistence_startup`
- Active nested child parent: `frontend.store.persistence_startup.persistence_helper_contract`
- Closed subchild leaf: `frontend.store.persistence_startup.persistence_helper_contract.graph_shape_validation_contract`
- Public surface:
  - `frontend/src/store/graphStorePersistenceHelpers.js`
  - `frontend/src/store/graphStoreGraphShapeValidation.js`

## Extraction

- Added `frontend/src/store/graphStoreGraphShapeValidation.js`.
- Moved graph shape, registry, recent-node, and validation helpers out of `graphStorePersistenceHelpers.js`:
  - `defaultCapabilities`
  - `defaultRegistry`
  - `createSafeFallbackCapabilities`
  - `buildRegistryFromCapabilities()`
  - `hasUsableGraphShape()`
  - `isDeprecatedBuiltinSampleGraph()`
  - `normalizeGraphShape()`
  - `recordRecentNodeIds()`
  - `withRecentNodeIds()`
  - `attachValidationWithRegistry()`
  - `fallbackRunnableGraph()`
  - `resolveLoadedGraph()`
  - `resolveLoadedGraphWithRegistry()`
- Kept `resolveGraphForDetail()` in `graphStorePersistenceHelpers.js` because it composes transport with loaded-graph resolution.
- Updated `graphStorePersistenceHelpers.js` to re-export the graph shape/validation API for compatibility.

## Whitebox Contract

- Inputs:
  - Built-in capabilities and module registry factories.
  - Raw graph payloads from storage, API transport, imports, or store actions.
  - Actor collaboration normalizer from `frontend/src/store/graphStoreActorCollaboration.js`.
  - Graph validation and QuantScript artifact helpers.
- Outputs:
  - Default capabilities and default registry.
  - Normalized graph shape with metadata, nodes, edges, editor state, runtime state, and validation state.
  - Validated runnable graph fallback.
  - Loaded graph resolution for persisted/API payloads.
  - Recent-node metadata update helpers.
- Parent communication:
  - Store actions continue importing through `graphStoreHelpers`.
  - The graph shape leaf imports actor collaboration directly and does not import API transport, storage, runtime actions, editor actions, or page modules.

## Preserved Behavior

- Invalid or empty graph payloads still fall back to `createEmptyGraph(defaultRegistry)`.
- Deprecated built-in sample graphs are still replaced with a fresh empty graph.
- Recent node ids are still limited to valid nodes and capped at eight.
- `attachValidationWithRegistry()` still attaches QuantScript artifacts before validation.
- `resolveLoadedGraphWithRegistry()` still returns `null` for unusable graph shapes.

## Further Split Decision

- `frontend.store.persistence_startup.persistence_helper_contract` remains active.
- Remaining queued subchildren:
  - `frontend.store.persistence_startup.persistence_helper_contract.version_audit_normalizers`

## Verification

- Targeted Vitest:
  - `npm.cmd test -- --run src/store/graphStoreRootState.test.js src/store/graphStoreCapabilityRefresh.test.js src/store/graphStore.startupRecovery.test.js src/store/graphStore.saveGraphRollback.test.js src/store/graphStoreEditorNodeActions.test.js src/store/graphStoreEditorEdgeActions.test.js src/store/graphStoreEditorDraftActions.test.js src/store/graphStoreEditorTemplateActions.test.js src/store/graphStore.versionHistory.test.js src/store/graphStorePersistenceConsistency.test.js src/store/graphStoreCompileOutcomeProjection.test.js src/store/graphStore.strategyIrCompile.test.js`
  - Result: passed, 12 files / 31 tests.
- Full feature tree gate: passed.
- Recursive state JSON parse: passed.
- Frontend build: passed.
- `git diff --check`: passed.
