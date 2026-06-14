# FE-0014 Frontend Routing Navigation Dispatch Closeout

Status: closed.

## Leaf Node

`frontend.routing.navigation_dispatch`

## Code Changes

- Added `frontend/src/routing/navigationDispatch.js`.
- Added `frontend/src/routing/navigationDispatch.test.js`.
- Updated `frontend/src/router.js` to keep compatibility export for `navigateTo`.

## Preserved Behavior

- `navigateTo(pathname)` still no-ops outside a browser window.
- Navigation to the current pathname still no-ops.
- Navigation still preserves the current hash.
- Navigation still pushes browser history and dispatches a synthetic `popstate`.
- Duplicate target navigation within 100ms is still ignored.

## Public Inputs

- Target pathname.
- Browser `window.location`, `window.history`, and current hash.
- Current timestamp from `Date.now()`.

## Public Outputs

- Browser history mutation.
- Synthetic `PopStateEvent`.
- Compatibility `navigateTo` export from `frontend/src/router.js`.

## Verification

- From `frontend/`, `npm.cmd test -- src/routing/navigationDispatch.test.js src/routing/routeContract.test.js src/router.test.js src/app/useAppRoute.test.jsx src/components/LeftSidebar.test.jsx`: passed, 5 test files and 20 tests.
- From `frontend/`, `npm.cmd run build`: passed, 1003 modules transformed.

## Further-Split Decision

No further split inside `frontend.routing.navigation_dispatch` now. The leaf has one public operation, one small state cache for duplicate suppression, and direct unit coverage for the browser side effects.

## Residuals

- `frontend.routing.shell_navigation` remains in `frontend/src/components/LeftSidebar.jsx` and command palette page commands.
