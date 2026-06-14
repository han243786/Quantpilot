# FE-0010 Frontend App Shell Global Overlays Closeout

Status: closed.

## Leaf Node

`frontend.app_shell.global_overlays`

## Code Changes

- Added `frontend/src/app/AppGlobalOverlays.jsx`.
- Added `frontend/src/app/AppGlobalOverlays.test.jsx`.
- Updated `frontend/src/App.jsx` to delegate tutorial overlay rendering, command palette hosting, and toast hosting.

## Preserved Behavior

- Tutorial overlay still opens from `useTutorial()` and closes through the tutorial hook.
- Tutorial steps still come from `createTutorialSteps(t)`.
- Command palette visibility is still controlled by the app shell shortcut state.
- Command palette close still clears the app shell command palette flag.
- Toast host still remains mounted at the app shell root.

## Public Inputs

- `commandPaletteOpen` boolean from `App.jsx`.
- `onCloseCommandPalette` callback from `App.jsx`.
- `useTutorial()` state and close callback.
- `useI18n().t` translation function.

## Public Outputs

- Tutorial overlay subtree when tutorial state is open.
- Command palette subtree.
- Toast container subtree.

## Verification

- From `frontend/`, `npm.cmd test -- src/app/AppGlobalOverlays.test.jsx src/app/AppRouteHost.test.jsx src/app/useAppRoute.test.jsx src/app/DesktopTitleBar.test.jsx src/app/useDesktopWindowChrome.test.jsx src/app/useAppEnvironmentEvents.test.jsx src/app/AppShellFallback.test.jsx src/app/useAppInitialization.test.jsx src/app/AppRoot.test.jsx src/app/installGlobalErrorHandlers.test.js src/router.test.js`: passed, 11 test files and 24 tests.
- From `frontend/`, `npm.cmd run build`: passed, 1001 modules transformed.

## Further-Split Decision

No further split inside `frontend.app_shell.global_overlays` now. Tutorial behavior, command palette visibility, and toast mounting are distinct dependencies, but the leaf is only a root-level overlay host with one small public surface. Splitting further would add wrapper churn before the owning feature modules are reached.

## Residuals

- `frontend.app_shell` parent closeout remains next.
