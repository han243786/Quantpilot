# FE-0016 Frontend Routing Parent Closeout

Status: closed.

## Parent Node

`frontend.routing`

## Closed Leaves

- `frontend.routing.route_contract`
- `frontend.routing.navigation_dispatch`
- `frontend.routing.shell_navigation`

## Final Parent Boundary

`frontend.routing` now owns the core browser route contract, navigation dispatch, and shell navigation route definitions. `frontend/src/router.js` remains as a compatibility gateway for existing imports while the concrete implementations live under `frontend/src/routing/`.

## Whitebox Contract

### Public Inputs

- Browser pathname, search string, hash, and history API.
- Strategy IDs and backtest IDs used by path builders.
- Shell navigation current path.

### Public Outputs

- Stable path builders and route objects.
- `navigateTo(pathname)` browser navigation dispatch.
- Sidebar navigation sections and command palette page navigation entries.
- Shell active-route matcher.

### Parent-Owned Files

- `frontend/src/router.js`
- `frontend/src/router.test.js`
- `frontend/src/routing/navigationDispatch.js`
- `frontend/src/routing/navigationDispatch.test.js`
- `frontend/src/routing/routeContract.js`
- `frontend/src/routing/routeContract.test.js`
- `frontend/src/routing/shellNavigation.js`
- `frontend/src/routing/shellNavigation.test.js`

## Preserved Behavior

- Route path shapes, query keys, and route object names are preserved.
- Navigation still mutates browser history, preserves hash, dispatches `popstate`, and deduplicates repeated target paths within 100ms.
- Existing imports from `frontend/src/router.js` continue to work.
- Sidebar and command palette navigation entries keep their visible behavior.

## Further-Split Decision

No further split is useful inside `frontend.routing` now. The remaining router gateway is intentionally small and exists to prevent a broad consumer migration during this parent. Feature-page navigation adapters should be handled with their feature parents, not by forcing cross-parent churn here.

## Verification

- Commit `c6bafed` pre-commit: full feature tree check passed.
- Commit `c6bafed` pre-commit: frontend build passed, 1004 modules transformed.
- Commit `c6bafed` pre-commit: Vitest passed, 112 test files and 327 tests.

## Next Parent Candidate

`frontend.api_client`
