# FE-0006 Frontend App Shell Startup Readiness Closeout

Status: closed.

## Leaf Node

`frontend.app_shell.startup_readiness`

## Code Changes

- Added `frontend/src/app/AppShellFallback.jsx`.
- Added `frontend/src/app/AppShellFallback.test.jsx`.
- Added `frontend/src/app/useAppInitialization.js`.
- Added `frontend/src/app/useAppInitialization.test.jsx`.
- Updated `frontend/src/App.jsx` to delegate startup fallback rendering and initialization readiness tracking to app shell leaf modules.

## Preserved Behavior

- The app still waits for `useGraphStore.initialize()` before rendering the main shell.
- The user can still force readiness by skipping the loading wait.
- The loading shell still reflects `capabilityStatus`.
- The delayed skip button still appears after 5 seconds when `onSkip` is provided.
- Route, page, overlay, and desktop window behavior remain unchanged in this leaf.

## Public Inputs

- `useGraphStore.initialize`.
- `useGraphStore.capabilityStatus`.
- `useI18n().t`.
- Optional `onSkip` callback from the parent shell.

## Public Outputs

- `useAppInitialization()` readiness boolean.
- `AppShellFallback` loading shell component.

## Verification

- `npm.cmd test -- src/app/AppShellFallback.test.jsx src/app/useAppInitialization.test.jsx src/app/AppRoot.test.jsx src/app/installGlobalErrorHandlers.test.js router.test.js`: passed, 9 tests.
- `npm.cmd run build`: passed, 995 modules transformed.

## Further-Split Decision

No further split is useful inside this leaf now. The fallback component and initialization hook are independent, small, and directly test-covered.

## Residuals

- `frontend.app_shell.environment_events` remains in `App.jsx`.
- `frontend.app_shell.desktop_window_chrome` remains in `App.jsx`.
- `frontend.app_shell.route_host` remains in `App.jsx`.
- `frontend.app_shell.global_overlays` remains in `App.jsx`.
