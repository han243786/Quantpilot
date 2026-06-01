# FE-0013 Frontend Routing Route Contract Closeout

Status: closed.

## Leaf Node

`frontend.routing.route_contract`

## Code Changes

- Added `frontend/src/routing/routeContract.js`.
- Added `frontend/src/routing/routeContract.test.js`.
- Updated `frontend/src/router.js` to keep compatibility exports for path builders and `parseRoute`.

## Preserved Behavior

- All existing route path builder names remain exported from `frontend/src/router.js`.
- `parseRoute()` continues to return the same route object shapes for strategies, strategy backtests, backtest detail, backtest compare, static shell routes, and not-found routes.
- Strategy and backtest IDs still use the same URL encoding and decoding behavior.
- Invalid strategy IDs still redirect to the strategies route object with the same error payload.
- `navigateTo()` remains in `frontend/src/router.js` for the next routing leaf.

## Public Inputs

- Route pathname.
- Route search string.
- Strategy IDs and backtest IDs for path builders.

## Public Outputs

- Public path builder functions.
- Public `parseRoute(pathname, search)` route object.
- Compatibility re-exports from `frontend/src/router.js`.

## Verification

- From `frontend/`, `npm.cmd test -- src/routing/routeContract.test.js src/router.test.js src/app/useAppRoute.test.jsx src/components/LeftSidebar.test.jsx src/app/AppRouteHost.test.jsx`: passed, 5 test files and 21 tests.
- From `frontend/`, `npm.cmd run build`: passed, 1002 modules transformed.

## Further-Split Decision

No further split inside `frontend.routing.route_contract` now. Path builders and route parsing are closely coupled by route names and path shape, and a direct test suite now protects the public contract.

## Residuals

- `frontend.routing.navigation_dispatch` remains in `frontend/src/router.js`.
- `frontend.routing.shell_navigation` remains in `frontend/src/components/LeftSidebar.jsx` and command palette page commands.
