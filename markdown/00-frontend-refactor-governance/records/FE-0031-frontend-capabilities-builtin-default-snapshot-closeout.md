# FE-0031 Frontend Capabilities Builtin Default Snapshot Closeout

Status: closed.

## Leaf Node

`frontend.capabilities.builtin_capability_snapshot.default_snapshot`

## Code Changes

- Added `frontend/src/capabilities/builtinCapabilitySnapshot.js`.
- Added `frontend/src/capabilities/builtinCapabilitySnapshot.test.js`.
- Updated `frontend/src/modules/builtinModules.js` to import and compatibility re-export `DEFAULT_CAPABILITIES` and `createSafeFallbackCapabilities`.

## Preserved Behavior

- Existing imports from `frontend/src/modules/builtinModules.js` still work.
- The default capability snapshot remains aligned with support-matrix constants.
- Safe fallback capabilities still use `schema_hash: "safe-fallback"`, disabled AI write policy, and declared-only risky surfaces/actions.

## Public Inputs

- Support matrix constants.
- Optional safe fallback reason text.

## Public Outputs

- `DEFAULT_CAPABILITIES`.
- `createSafeFallbackCapabilities(reason)`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/capabilities/builtinCapabilitySnapshot.test.js src/capabilities/supportMatrix.test.js src/store/graphStore.capabilities.test.js`: passed, 3 test files and 17 tests.

## Further-Split Decision

`frontend.capabilities.builtin_capability_snapshot` is worth continuing to split. Remaining capability normalization logic still lives in `frontend/src/modules/builtinModules.js`.

## Residuals

- Continue with `frontend.capabilities.builtin_capability_snapshot.normalization`.
