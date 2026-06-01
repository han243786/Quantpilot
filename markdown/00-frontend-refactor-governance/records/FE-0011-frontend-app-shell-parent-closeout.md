# FE-0011 Frontend App Shell Parent Closeout

Status: closed.

## Parent Node

`frontend.app_shell`

## Closed Leaves

- `frontend.app_shell.bootstrap_root`
- `frontend.app_shell.startup_readiness`
- `frontend.app_shell.environment_events`
- `frontend.app_shell.desktop_window_chrome`
- `frontend.app_shell.route_host`
- `frontend.app_shell.global_overlays`

## Final Parent Boundary

`frontend.app_shell` now owns the React app root, shell initialization, browser/desktop shell effects, route hosting, and root overlay hosting. `frontend/src/App.jsx` remains a thin composer for parent-level shell glue:

- initialization fallback and manual skip state
- desktop title bar mount
- left sidebar mount
- offline and storage-quota shell banners
- skip link and main content container
- route host mount
- command palette shortcut state and overlay handoff

## Whitebox Contract

### Public Inputs

- Browser location and navigation events.
- Desktop window API when running inside Tauri.
- Browser online/offline, storage quota, and shell keyboard events.
- Tutorial state, command palette state, and route object.

### Public Outputs

- Root React app subtree.
- Desktop/browser shell chrome.
- Shell warning banners.
- Routed page content host.
- Global overlay host.

### Parent-Owned Files

- `frontend/src/main.jsx`
- `frontend/src/App.jsx`
- `frontend/src/app/AppGlobalOverlays.jsx`
- `frontend/src/app/AppGlobalOverlays.test.jsx`
- `frontend/src/app/AppRouteHost.jsx`
- `frontend/src/app/AppRouteHost.test.jsx`
- `frontend/src/app/AppRoot.jsx`
- `frontend/src/app/AppRoot.test.jsx`
- `frontend/src/app/AppShellFallback.jsx`
- `frontend/src/app/AppShellFallback.test.jsx`
- `frontend/src/app/DesktopTitleBar.jsx`
- `frontend/src/app/DesktopTitleBar.test.jsx`
- `frontend/src/app/installGlobalErrorHandlers.js`
- `frontend/src/app/installGlobalErrorHandlers.test.js`
- `frontend/src/app/useAppEnvironmentEvents.js`
- `frontend/src/app/useAppEnvironmentEvents.test.jsx`
- `frontend/src/app/useAppInitialization.js`
- `frontend/src/app/useAppInitialization.test.jsx`
- `frontend/src/app/useAppRoute.js`
- `frontend/src/app/useAppRoute.test.jsx`
- `frontend/src/app/useDesktopWindowChrome.js`
- `frontend/src/app/useDesktopWindowChrome.test.jsx`

## Preserved Behavior

- React still mounts through the same root and keeps global error handling.
- Store initialization, startup fallback, and manual skip behavior are preserved.
- Desktop title bar controls still delegate to the desktop window API.
- Offline, storage quota, and command palette events still run from the app shell.
- Route parsing and routed page rendering remain equivalent.
- Tutorial, command palette, and toast overlays remain mounted at the root shell.

## Further-Split Decision

No further split is useful inside `frontend.app_shell` now. Remaining code in `App.jsx` is parent-level composition glue, not a hidden domain feature. Deeper work belongs to later parent modules:

- `frontend.routing` for route contract, navigation links, and route helpers.
- `frontend.design_system_styles` for global shell layout and styling.
- `frontend.strategy_hub`, `frontend.strategy_workspace`, and runtime parents for routed feature internals.

## Verification

- Commit `5965198` pre-commit: full feature tree check passed.
- Commit `5965198` pre-commit: frontend build passed, 1001 modules transformed.
- Commit `5965198` pre-commit: Vitest passed, 109 test files and 317 tests.

## Next Parent Candidate

`frontend.routing`
