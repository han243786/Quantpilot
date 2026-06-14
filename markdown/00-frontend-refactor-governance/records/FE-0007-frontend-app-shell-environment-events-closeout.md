# FE-0007 Frontend App Shell Environment Events Closeout

Status: closed.

## Leaf Node

`frontend.app_shell.environment_events`

## Code Changes

- Added `frontend/src/app/useAppEnvironmentEvents.js`.
- Added `frontend/src/app/useAppEnvironmentEvents.test.jsx`.
- Updated `frontend/src/App.jsx` to delegate global browser environment event wiring to the app shell hook.

## Preserved Behavior

- Stored theme is still applied on startup and on `qp-theme-change`.
- Online/offline browser events still drive the offline banner state.
- `qp-storage-quota-exceeded` still enables the storage warning banner, and the banner can still clear it.
- `visibilitychange` still refreshes the graph index when the page becomes visible.
- `beforeunload` still guards strategy workspace and QuantScript editing routes.
- Ctrl/Cmd+K still toggles the command palette.

## Public Inputs

- Current route object from the app route host.
- `onToggleCommandPalette` callback from the shell.
- Browser `window`, `document`, `navigator`, `localStorage`, and lifecycle events.
- `useGraphStore.getState().refreshGraphIndex`.

## Public Outputs

- `isOffline` state.
- `storageQuotaExceeded` state.
- `setStorageQuotaExceeded` setter for the storage warning banner.

## Verification

- `npm.cmd test -- src/app/useAppEnvironmentEvents.test.jsx src/app/AppShellFallback.test.jsx src/app/useAppInitialization.test.jsx src/app/AppRoot.test.jsx src/app/installGlobalErrorHandlers.test.js router.test.js`: passed, 11 tests.
- `npm.cmd run build`: passed, 996 modules transformed.

## Further-Split Decision

No further split is useful right now. The hook is cohesive around browser environment events and has focused tests for network state, storage quota state, visibility refresh, keyboard shortcut, theme application, and unload guarding.

## Residuals

- `frontend.app_shell.desktop_window_chrome` remains in `App.jsx`.
- `frontend.app_shell.route_host` remains in `App.jsx`.
- `frontend.app_shell.global_overlays` remains in `App.jsx`.
