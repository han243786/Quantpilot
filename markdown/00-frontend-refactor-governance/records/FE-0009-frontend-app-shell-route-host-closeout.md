# FE-0009 Frontend App Shell Route Host Closeout

Status: closed.

## Leaf Node

`frontend.app_shell.route_host`

## Code Changes

- Added `frontend/src/app/AppRouteHost.jsx`.
- Added `frontend/src/app/AppRouteHost.test.jsx`.
- Added `frontend/src/app/useAppRoute.js`.
- Added `frontend/src/app/useAppRoute.test.jsx`.
- Updated `frontend/src/App.jsx` to delegate route state, root redirect, popstate handling, lazy page loading, Suspense fallback, and per-route ErrorBoundary wrapping.

## Preserved Behavior

- Initial route still comes from `window.location.pathname` and `window.location.search`.
- Browser `popstate` still refreshes the app route.
- Root path `/` still redirects to `/strategies`.
- Strategy workspace, strategy backtests, backtest detail, backtest compare, approval, utility, settings, and not-found routes still render through lazy page components.
- Each routed page is still wrapped by an `ErrorBoundary`.
- Suspense still uses `AppShellFallback` while lazy pages load.

## Public Inputs

- Browser location and `popstate`.
- `parseRoute` and `strategiesPath` from `frontend/src/router.js`.
- Route object passed to `AppRouteHost`.
- Lazy page modules.

## Public Outputs

- `useAppRoute()` current route object.
- `AppRouteHost` routed content subtree.

## Verification

- `npm.cmd test -- src/app/AppRouteHost.test.jsx src/app/useAppRoute.test.jsx src/app/DesktopTitleBar.test.jsx src/app/useDesktopWindowChrome.test.jsx src/app/useAppEnvironmentEvents.test.jsx src/app/AppShellFallback.test.jsx src/app/useAppInitialization.test.jsx src/app/AppRoot.test.jsx src/app/installGlobalErrorHandlers.test.js router.test.js`: passed, 22 tests.
- `npm.cmd run build`: passed, 1000 modules transformed.

## Further-Split Decision

No further split inside `frontend.app_shell.route_host` now. `useAppRoute` owns route state and browser navigation wiring; `AppRouteHost` owns route rendering. Deeper route parsing remains owned by `frontend.routing`.

## Residuals

- `frontend.app_shell.global_overlays` remains in `App.jsx`.
