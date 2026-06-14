# FE-0037 Frontend Capabilities Store Refresh Facade Closeout

Status: closed.

## Leaf Node

`frontend.capabilities.store_capability_refresh.public_action_facade`

## Code Changes

- No production code changes in this step.
- Confirmed `refreshCapabilities` in `frontend/src/store/graphStore.js` is now a thin public action facade over `graphStoreCapabilityRefresh`.

## Preserved Behavior

- `refreshCapabilities` still sets loading state before fetch.
- Remote capability refresh still returns the remote snapshot after applying store state.
- Failure with cache and failure without cache still return the projected capability snapshot from the helper.
- Public callers still use `useGraphStore.getState().refreshCapabilities()` without changing imports or call shape.

## Public Inputs

- `/capabilities` response through `fetchJson`.
- Current graph store state through `get()`.
- Capability fallback messages owned by the public action.

## Public Outputs

- Updated Zustand store state via `set(refresh.state)`.
- Returned capability snapshot from the selected refresh path.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/store/graphStoreCapabilityRefresh.test.js src/store/graphStore.capabilities.test.js src/pages/StrategyHubPage.test.jsx`: passed, 3 test files and 10 tests.

## Further-Split Decision

`frontend.capabilities.store_capability_refresh.public_action_facade` does not need further split now. It is intentionally a thin public action boundary.

## Parent Closeout

`frontend.capabilities.store_capability_refresh` is closed for this pass.

Closed leaves:

- `frontend.capabilities.store_capability_refresh.refresh_state_projection`
- `frontend.capabilities.store_capability_refresh.public_action_facade`

## Residuals

- Evaluate whether `frontend.capabilities` can close as a parent.
