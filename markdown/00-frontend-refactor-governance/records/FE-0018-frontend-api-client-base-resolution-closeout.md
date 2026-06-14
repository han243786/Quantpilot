# FE-0018 Frontend API Client Base Resolution Closeout

Status: closed.

## Leaf Node

`frontend.api_client.base_resolution`

## Code Changes

- Added `frontend/src/api/apiBase.js`.
- Added `frontend/src/api/apiBase.test.js`.
- Updated `frontend/src/api/client.js` to re-export `API_BASE`, `resolveApiBase`, and `getAuthHeaders` from the base module.

## Preserved Behavior

- `VITE_API_BASE_URL` still takes precedence when provided.
- Explicit API base values still trim trailing slashes.
- Browser fallback remains `/api`.
- Non-browser fallback remains `http://127.0.0.1:3000/api`.
- `getAuthHeaders()` remains an empty extension point.
- Existing imports from `frontend/src/api/client.js` remain compatible.

## Public Inputs

- `import.meta.env.VITE_API_BASE_URL`.
- Runtime browser availability.

## Public Outputs

- `resolveApiBase(options)`.
- `API_BASE`.
- `getAuthHeaders()`.
- Compatibility re-exports from `frontend/src/api/client.js`.

## Verification

- From `frontend/`, `npm.cmd test -- src/api/apiBase.test.js src/pages/StrategyConfigCockpit.test.jsx`: passed, 2 test files and 6 tests.
- From `frontend/`, `npm.cmd run build`: passed, 1005 modules transformed.

## Further-Split Decision

No further split inside `frontend.api_client.base_resolution` now. The leaf is a compact configuration boundary with one resolver and one auth extension point.

## Residuals

- `frontend.api_client.request_transport` remains in `frontend/src/api/client.js`.
- `frontend.api_client.compat_fetch_helpers` remains in `frontend/src/utils/api.js`.
