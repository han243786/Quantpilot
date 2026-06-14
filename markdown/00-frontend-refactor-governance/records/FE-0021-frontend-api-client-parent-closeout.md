# FE-0021 Frontend API Client Parent Closeout

Status: closed.

## Parent Node

`frontend.api_client`

## Closed Leaves

- `frontend.api_client.base_resolution`
- `frontend.api_client.request_transport`
- `frontend.api_client.compat_fetch_helpers`

## Final Parent Boundary

`frontend.api_client` now owns frontend API base resolution, auth header extension, JSON request transport, method-specific API client helpers, timeout fetch helpers, and the compatibility `utils/api.js` gateway for legacy imports.

## Whitebox Contract

### Public Inputs

- Optional `VITE_API_BASE_URL`.
- Browser/window availability when resolving the API base.
- Request method, path, body, headers, timeout, API base, and fetch implementation.
- URL, fetch options, and timeout for compatibility fetch helpers.

### Public Outputs

- Stable `API_BASE`, `resolveApiBase`, and `getAuthHeaders`.
- `request(method, path, body, options)`.
- `createApiClient(options)` and `apiClient`.
- `withPagination(path, options)`.
- `fetchWithTimeout(url, options, timeoutMs)`.
- Compatibility exports from `frontend/src/utils/api.js`.

### Parent-Owned Files

- `frontend/src/api/apiBase.js`
- `frontend/src/api/apiBase.test.js`
- `frontend/src/api/apiTransport.js`
- `frontend/src/api/apiTransport.test.js`
- `frontend/src/api/client.js`
- `frontend/src/api/fetchHelpers.js`
- `frontend/src/api/fetchHelpers.test.js`
- `frontend/src/utils/api.js`

## Preserved Behavior

- API base resolution still trims configured bases, falls back to `/api` in browsers, and uses `http://127.0.0.1:3000/api` outside window contexts.
- JSON request helpers still set content type, serialize provided bodies, surface non-OK response text with status, and parse JSON responses by content type.
- `apiClient.get`, `apiClient.post`, and `apiClient.del` remain stable.
- `fetchWithTimeout` still attaches an abort signal, preserves caller fetch options, and clears its timer after settle.
- Existing imports from both `frontend/src/api/client.js` and `frontend/src/utils/api.js` continue to work.

## Further-Split Decision

No further split is useful inside `frontend.api_client` now. The parent has three compact leaf boundaries: base resolution, request transport, and compatibility fetch helpers. Broader fetch consumer migration should happen inside their owning feature parents, not by forcing cross-parent churn here.

## Verification

- Commit `1979f49` pre-commit: full feature tree check passed.
- Commit `1979f49` pre-commit: frontend build passed, 1007 modules transformed.
- Commit `1979f49` pre-commit: Vitest passed, 115 test files and 337 tests.

## Next Parent Candidate

`frontend.capabilities`
