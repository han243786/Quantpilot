# FE-0017 Frontend API Client Baseline

Status: baseline established.

## Parent Node

`frontend.api_client`

## Current Scope

The current frontend API client has one primary gateway in `frontend/src/api/client.js` and one compatibility utility module in `frontend/src/utils/api.js`. Several feature and store modules still use direct `fetch` or local JSON helpers; those consumers are recorded as boundaries but should not be mass-migrated during this parent.

## Initial Child Queue

- `frontend.api_client.base_resolution`
- `frontend.api_client.request_transport`
- `frontend.api_client.compat_fetch_helpers`

## Current Owned Files

- `frontend/src/api/client.js`
- `frontend/src/utils/api.js`

## Important Consumers

- `frontend/src/components/DeployButton.jsx`
- `frontend/src/pages/StrategyConfigCockpit.jsx`
- `frontend/src/store/graphStorePersistenceHelpers.js`
- `frontend/src/pages/AlertsPage.jsx`
- `frontend/src/pages/ChaosPage.jsx`
- `frontend/src/pages/RunbookPage.jsx`
- `frontend/src/pages/SnapshotsPage.jsx`
- `frontend/src/pages/StrategyWorkspaceSourceTab.jsx`
- `frontend/src/components/TopToolbar.jsx`
- `frontend/src/utils/runtimeApproval.js`

## Whitebox Contract

### Public Inputs

- `import.meta.env.VITE_API_BASE_URL`.
- Browser or non-browser runtime environment.
- Request path, method, body, headers, and timeout.
- Native `fetch`, `AbortController`, and response body APIs.

### Public Outputs

- `API_BASE`.
- `apiClient.get`, `apiClient.post`, and `apiClient.del`.
- `withPagination(path, pagination)`.
- `getAuthHeaders()`.
- Compatibility `fetchWithTimeout(url, options, timeoutMs)` from `frontend/src/utils/api.js`.

## Equivalence Anchors

- New direct API client unit tests should be added during extraction.
- Existing `StrategyConfigCockpit` test mocks `../api/client`.
- Existing store and page tests indirectly cover direct fetch callers.
- Frontend build.

## Split Rules

- Do not change API base resolution order or slash trimming.
- Do not change timeout defaults, JSON serialization, or error status attachment.
- Do not migrate feature-owned direct `fetch` callers unless the current leaf needs a compatibility adapter.
- Keep `frontend/src/api/client.js` and `frontend/src/utils/api.js` compatibility exports stable until all frontend parents are processed.

## First Leaf

`frontend.api_client.base_resolution`
