# FE-0008 Frontend App Shell Desktop Window Chrome Closeout

Status: closed.

## Leaf Node

`frontend.app_shell.desktop_window_chrome`

## Code Changes

- Added `frontend/src/app/DesktopTitleBar.jsx`.
- Added `frontend/src/app/DesktopTitleBar.test.jsx`.
- Added `frontend/src/app/useDesktopWindowChrome.js`.
- Added `frontend/src/app/useDesktopWindowChrome.test.jsx`.
- Updated `frontend/src/App.jsx` to delegate Tauri window resolution, maximized-state tracking, and titlebar rendering to app shell leaf modules.

## Preserved Behavior

- Browser runtime still renders no desktop titlebar.
- Tauri runtime still resolves the current window through `getCurrentWindow()`.
- Titlebar controls still call `minimize`, `toggleMaximize`, and `close` on the desktop window.
- Maximized state is still refreshed on resize.
- Main content still uses the desktop titlebar offset when `appWindow` exists.

## Public Inputs

- Optional `window.__TAURI_INTERNALS__` marker.
- `getCurrentWindow()` from `@tauri-apps/api/window`.
- Desktop window methods: `isMaximized`, `onResized`, `minimize`, `toggleMaximize`, and `close`.
- `useI18n().t` for accessible labels.

## Public Outputs

- `useDesktopWindowChrome()` returns `{ appWindow, isMaximized }`.
- `DesktopTitleBar` renders desktop controls only when an app window exists.

## Verification

- `npm.cmd test -- src/app/DesktopTitleBar.test.jsx src/app/useDesktopWindowChrome.test.jsx src/app/useAppEnvironmentEvents.test.jsx src/app/AppShellFallback.test.jsx src/app/useAppInitialization.test.jsx src/app/AppRoot.test.jsx src/app/installGlobalErrorHandlers.test.js router.test.js`: passed, 15 tests.
- `npm.cmd run build`: passed, 998 modules transformed.

## Further-Split Decision

No further split is useful now. The leaf separates runtime detection/state from the render component, and both sides are covered by focused tests.

## Residuals

- `frontend.app_shell.route_host` remains in `App.jsx`.
- `frontend.app_shell.global_overlays` remains in `App.jsx`.
