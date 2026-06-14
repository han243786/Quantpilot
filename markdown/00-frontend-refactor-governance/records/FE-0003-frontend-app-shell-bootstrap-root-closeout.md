# FE-0003 Frontend App Shell Bootstrap Root Closeout

Status: closed.

## Leaf Node

`frontend.app_shell.bootstrap_root`

## Code Changes

- Added `frontend/src/app/AppRoot.jsx`.
- Added `frontend/src/app/AppRoot.test.jsx`.
- Added `frontend/src/app/installGlobalErrorHandlers.js`.
- Added `frontend/src/app/installGlobalErrorHandlers.test.js`.
- Updated `frontend/src/main.jsx` so the entrypoint now delegates app tree composition and global rejection handling to the app shell leaf.

## Preserved Behavior

- The app still mounts into the DOM element with id `root`.
- The rendered app tree still runs under `React.StrictMode`.
- The app is still wrapped by `I18nProvider`.
- The test bridge is still installed before rendering.
- Global `unhandledrejection` events still log `[UnhandledRejection]` and the rejection reason.
- Global style imports remain in the entrypoint and have not been moved in this leaf.

## Public Inputs

- Root DOM element from `document.getElementById("root")`.
- `App` component.
- `I18nProvider`.
- Browser `window` unhandled rejection event.

## Public Outputs

- `AppRoot` component for root tree composition.
- `installGlobalErrorHandlers` function for entrypoint side effects.
- Production build still emits the same app entry bundle shape from Vite's perspective.

## Verification

- `npm.cmd test -- src/app/AppRoot.test.jsx src/app/installGlobalErrorHandlers.test.js router.test.js`: passed, 7 tests.
- `npm.cmd run build`: passed, 993 modules transformed.

## Further-Split Decision

No further split is useful inside `frontend.app_shell.bootstrap_root` right now. The leaf now contains one root wrapper and one global error handler installer; splitting further would create naming overhead without reducing coupling.

## Residuals

- `frontend.app_shell.startup_readiness` remains in `App.jsx`.
- `frontend.app_shell.environment_events` remains in `App.jsx`.
- `frontend.app_shell.desktop_window_chrome` remains in `App.jsx`.
- `frontend.app_shell.route_host` remains in `App.jsx`.
- `frontend.app_shell.global_overlays` remains in `App.jsx`.
