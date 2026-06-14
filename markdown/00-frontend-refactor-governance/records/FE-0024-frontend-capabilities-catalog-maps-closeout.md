# FE-0024 Frontend Capabilities Catalog Maps Closeout

Status: closed.

## Leaf Node

`frontend.capabilities.support_matrix_truth.catalog_maps`

## Code Changes

- Added `frontend/src/capabilities/capabilityCatalog.js`.
- Added `frontend/src/capabilities/capabilityCatalog.test.js`.
- Updated `frontend/src/capabilities/supportMatrix.js` into a compatibility gateway for catalog exports plus remaining behavior functions.

## Preserved Behavior

- Existing imports from `frontend/src/capabilities/supportMatrix.js` for catalog constants and maps remain valid.
- Supported indicator, runtime, market, frontend module, workspace surface, and UI action maps remain unchanged.
- `SUPPORT_MATRIX` still exposes the same nested catalog structure.

## Public Inputs

- Static frontend capability catalog definitions.

## Public Outputs

- `DECLARED_INDICATOR_KINDS`.
- `SUPPORTED_INDICATOR_KINDS`.
- `SUPPORTED_RUNTIME_MODES`.
- `SUPPORTED_RUNTIME_EXECUTION_MODULES`.
- `SUPPORTED_EXCHANGES`.
- `SUPPORTED_SYMBOLS`.
- `SUPPORTED_FRONTEND_MODULE_KEYS`.
- `WORKSPACE_SURFACE_MAP`.
- `CAPABILITY_ACTION_MAP`.
- `SUPPORT_MATRIX`.

## Verification

- From `frontend/`, `npm.cmd test -- src/capabilities/capabilityCatalog.test.js src/capabilities/supportMatrix.test.js src/capabilities/capabilityGovernance.test.js src/capabilities/capabilityProjection.test.js src/modules/moduleRegistry.test.js`: passed, 5 test files and 31 tests.

## Further-Split Decision

No further split inside `frontend.capabilities.support_matrix_truth.catalog_maps` now. The file is a static catalog node; behavior belongs to separate boundary and action block leaves.

## Residuals

- `frontend.capabilities.support_matrix_truth.boundary_context` remains in `frontend/src/capabilities/supportMatrix.js`.
- `frontend.capabilities.support_matrix_truth.action_block_reason` remains in `frontend/src/capabilities/supportMatrix.js`.
