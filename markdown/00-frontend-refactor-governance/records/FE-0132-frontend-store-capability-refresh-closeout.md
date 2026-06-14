# FE-0132 Frontend Store Capability Refresh Closeout

Status: closed.

## Scope

- Parent node: `frontend.store`
- Closed leaf: `frontend.store.capability_refresh`
- Primary files:
  - `frontend/src/store/graphStore.js`
  - `frontend/src/store/graphStoreCapabilityRefresh.js`
  - `frontend/src/store/graphStoreCapabilityRefresh.test.js`
  - `frontend/src/store/graphStore.capabilities.test.js`

## Whitebox Boundary

- Inputs:
  - `/capabilities` response loaded through `fetchJson`.
  - Existing graph store state from `get()`.
  - Cached capability snapshot from local storage on refresh failure.
- Processing:
  - `refreshCapabilities` owns the async store action boundary and loading/error state transition.
  - `buildRemoteCapabilityRefreshState` projects remote capability snapshots into ready store state.
  - `buildCapabilityRefreshFailureState` chooses cached capability recovery before safe fallback.
  - `buildCachedCapabilityRefreshState` and `buildSafeFallbackCapabilityRefreshState` preserve degraded/error state paths.
- Outputs:
  - Updated registry, capability snapshot, capability status/source/message, graph validation state, `quantScriptDraft`, and `strategyIrDraft`.
  - Persisted capability cache for remote success.
  - Persisted graph storage after validation refresh.
- Parent communication:
  - The public store method remains `useGraphStore.getState().refreshCapabilities()`.
  - Helper methods stay behind `frontend/src/store/graphStoreCapabilityRefresh.js` and communicate through `graphStore.js`.

## Recursive Split Decision

- No further subleaf split is required.
- Hard-rule assessment:
  - The leaf is small enough to audit directly.
  - The public behavior is one cohesive refresh workflow: remote success, cached fallback, safe fallback.
  - Existing tests already isolate the projection helper and the store public method.
  - Splitting the helper into smaller files would add pass-through imports without reducing dependency risk.
- Next queued leaf: `frontend.store.editor_actions`.

## Equivalence Baseline

- Remote success keeps `capabilityStatus=ready`, `capabilitySource=remote`, caches the snapshot, rebuilds the registry, and refreshes graph validation.
- Fetch failure with cached capabilities keeps `capabilityStatus=degraded`, `capabilitySource=cache`, and preserves cached support data.
- Fetch failure without cache keeps `capabilityStatus=error`, `capabilitySource=safe_fallback`, and narrows module/workspace/action support to declared-only fallback.
- Startup recovery remains covered because startup actions can call `refreshCapabilities`.

## Verification

- `npm.cmd test -- --run src/store/graphStoreCapabilityRefresh.test.js src/store/graphStore.capabilities.test.js src/store/graphStore.startupRecovery.test.js`
- `git diff --check`
- `node -e "JSON.parse(require('fs').readFileSync('markdown/00-frontend-refactor-governance/frontend-recursive-state.json','utf8')); console.log('recursive state json ok')"`
- `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`
