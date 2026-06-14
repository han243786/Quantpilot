# FE-0032 Frontend Capabilities Normalization Closeout

Status: closed.

## Leaf Node

`frontend.capabilities.builtin_capability_snapshot.normalization`

## Code Changes

- Added `frontend/src/capabilities/capabilityNormalization.js`.
- Added `frontend/src/capabilities/capabilityNormalization.test.js`.
- Updated `frontend/src/modules/builtinModules.js` to delegate `normalizeCapabilities` through a compatibility wrapper that supplies builtin module keys.

## Preserved Behavior

- Existing imports from `frontend/src/modules/builtinModules.js` still work.
- Invalid capability input still returns `DEFAULT_CAPABILITIES`.
- Unsafe permission-boundary values still normalize to restrictive behavior.
- Frontend module support still derives declared module keys from builtin modules, supported keys, and unsupported reason maps.

## Public Inputs

- Backend capability snapshot.
- Optional `knownModuleKeys` for frontend module support derivation.

## Public Outputs

- `normalizeCapabilities(capabilities, { knownModuleKeys })`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/capabilities/capabilityNormalization.test.js src/capabilities/supportMatrix.test.js src/store/graphStore.capabilities.test.js src/store/graphStore.startupRecovery.test.js`: passed, 4 test files and 21 tests.

## Further-Split Decision

No further split inside normalization now. It is a single sanitization and compatibility boundary for backend capability snapshots.

## Residuals

- `frontend.capabilities.builtin_capability_snapshot` is closed.
- Continue with `frontend.capabilities.module_registry_gate`.
