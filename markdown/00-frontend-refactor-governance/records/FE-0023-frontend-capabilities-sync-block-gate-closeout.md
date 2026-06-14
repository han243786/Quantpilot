# FE-0023 Frontend Capabilities Sync Block Gate Closeout

Status: closed.

## Leaf Node

`frontend.capabilities.support_matrix_truth.sync_block_gate`

## Code Changes

- Added `frontend/src/capabilities/capabilitySync.js`.
- Added `frontend/src/capabilities/capabilitySync.test.js`.
- Updated `frontend/src/capabilities/supportMatrix.js` to re-export `isCapabilitySyncBlocked` from the new whitebox node.

## Preserved Behavior

- `isCapabilitySyncBlocked("loading", "remote")` remains `true`.
- `isCapabilitySyncBlocked("error", "safe_fallback")` remains `true`.
- Cache/degraded states remain usable and return `false`.
- Existing imports from `frontend/src/capabilities/supportMatrix.js` continue to work.

## Public Inputs

- Capability status.
- Capability source.

## Public Outputs

- Boolean sync-block decision for UI and action gates.

## Verification

- From `frontend/`, `npm.cmd test -- src/capabilities/capabilitySync.test.js src/capabilities/supportMatrix.test.js src/components/ModuleSidebar.test.jsx src/pages/StrategyWorkspaceExperimentCard.test.jsx`: passed, 4 test files and 22 tests.

## Further-Split Decision

`frontend.capabilities.support_matrix_truth` is worth further recursive split. The sync block gate is now closed; the remaining support matrix concerns should continue as catalog maps, boundary context, and action block reason leaves.

## Residuals

- `frontend.capabilities.support_matrix_truth.catalog_maps` remains in `frontend/src/capabilities/supportMatrix.js`.
- `frontend.capabilities.support_matrix_truth.boundary_context` remains in `frontend/src/capabilities/supportMatrix.js`.
- `frontend.capabilities.support_matrix_truth.action_block_reason` remains in `frontend/src/capabilities/supportMatrix.js`.
