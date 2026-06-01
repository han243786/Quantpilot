# FE-0019 Frontend API Client Request Transport Closeout

Status: closed.

## Leaf Node

`frontend.api_client.request_transport`

## Code Changes

- Added `frontend/src/api/apiTransport.js`.
- Added `frontend/src/api/apiTransport.test.js`.
- Updated `frontend/src/api/client.js` to create `apiClient` from the transport module and keep compatibility exports.

## Preserved Behavior

- Requests still use JSON content type by default.
- Request bodies are still JSON serialized only when provided.
- The default timeout remains 30000ms.
- Failed responses still throw an `Error` with `status` attached.
- JSON responses still parse through `response.json()`.
- Non-JSON responses still return `response.text()`.
- `apiClient.get`, `apiClient.post`, and `apiClient.del` remain stable.

## Public Inputs

- Method, path, body, headers, timeout, API base, and fetch implementation.
- Native response `ok`, `status`, `text`, `json`, and `content-type`.

## Public Outputs

- `request(method, path, body, options)`.
- `createApiClient(options)`.
- Compatibility `apiClient` export from `frontend/src/api/client.js`.

## Verification

- From `frontend/`, `npm.cmd test -- src/api/apiTransport.test.js src/api/apiBase.test.js src/pages/StrategyConfigCockpit.test.jsx`: passed, 3 test files and 10 tests.
- From `frontend/`, `npm.cmd run build`: passed, 1006 modules transformed.

## Further-Split Decision

No further split inside `frontend.api_client.request_transport` now. Timeout, JSON body serialization, error conversion, and method helper creation are one cohesive transport contract.

## Residuals

- `frontend.api_client.compat_fetch_helpers` remains in `frontend/src/utils/api.js`.
