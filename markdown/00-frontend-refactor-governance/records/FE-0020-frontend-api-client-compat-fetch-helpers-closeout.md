# FE-0020 Frontend API Client Compat Fetch Helpers Closeout

Status: closed.

## Leaf Node

`frontend.api_client.compat_fetch_helpers`

## Code Changes

- Added `frontend/src/api/fetchHelpers.js`.
- Added `frontend/src/api/fetchHelpers.test.js`.
- Updated `frontend/src/api/client.js` to export `fetchWithTimeout` from the API client boundary.
- Updated `frontend/src/utils/api.js` into a compatibility re-export layer.

## Preserved Behavior

- `fetchWithTimeout(url, options, timeoutMs)` still forwards the URL and caller options to `fetch`.
- The default timeout remains 30000ms.
- A fresh `AbortSignal` is still attached to each request.
- The timeout timer is still cleared when the request settles.
- Existing `../utils/api` imports for `API_BASE`, `getAuthHeaders`, and `fetchWithTimeout` remain valid.

## Public Inputs

- URL, fetch options, timeout in milliseconds.
- Existing API base and auth header exports from the API client boundary.

## Public Outputs

- `fetchWithTimeout(url, options, timeoutMs)`.
- Compatibility exports from `frontend/src/utils/api.js`.

## Verification

- From `frontend/`, `npm.cmd test -- src/api/fetchHelpers.test.js src/api/apiBase.test.js src/api/apiTransport.test.js`: passed, 3 test files and 10 tests.
- From `frontend/`, `npm.cmd run build`: passed, 1007 modules transformed.

## Further-Split Decision

No further split inside `frontend.api_client.compat_fetch_helpers` now. The leaf is a compact compatibility adapter with one timeout helper and legacy re-export surface.

## Residuals

- No open child leaves remain under `frontend.api_client`.
