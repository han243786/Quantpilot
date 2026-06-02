# FE-0036 Frontend Capabilities Store Refresh Projection Closeout

Status: closed.

## Leaf Node

`frontend.capabilities.store_capability_refresh.refresh_state_projection`

## Code Changes

- Added `frontend/src/store/graphStoreCapabilityRefresh.js`.
- Added `frontend/src/store/graphStoreCapabilityRefresh.test.js`.
- Updated `frontend/src/store/graphStore.js` so `refreshCapabilities` delegates registry rebuild, graph revalidation, draft resolution, capability cache writes, and fallback projection to the extracted helper.

## Preserved Behavior

- Successful remote `/capabilities` refresh still enters `ready` / `remote`, rebuilds the registry, revalidates the current graph, updates drafts, and caches the capability snapshot.
- Failed refresh with cached capabilities still enters `degraded` / `cache` and projects the cached snapshot.
- Failed refresh with no cache still enters `error` / `safe_fallback` and uses safe fallback capabilities.
- Module sidebar and toolbar capability gating continue to observe the same store state shape.

## Public Inputs

- Remote capability snapshot.
- Current graph store state.
- Refresh failure error.
- Capability fallback messages supplied by the store action facade.

## Public Outputs

- `buildRemoteCapabilityRefreshState(capabilities, currentState)`.
- `buildCapabilityRefreshFailureState(error, currentState, messages)`.
- Store-ready capability refresh state payload.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/store/graphStoreCapabilityRefresh.test.js src/store/graphStore.capabilities.test.js src/components/ModuleSidebar.test.jsx src/components/TopToolbar.capabilities.test.jsx`: passed, 4 test files and 16 tests.

## Further-Split Decision

`frontend.capabilities.store_capability_refresh` is worth one more closeout split. The state projection is now isolated; the remaining public action facade in `graphStore.js` should stay thin and be closed separately.

## Residuals

- Continue with `frontend.capabilities.store_capability_refresh.public_action_facade`.
