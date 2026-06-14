# FE-0015 Frontend Routing Shell Navigation Closeout

Status: closed.

## Leaf Node

`frontend.routing.shell_navigation`

## Code Changes

- Added `frontend/src/routing/shellNavigation.js`.
- Added `frontend/src/routing/shellNavigation.test.js`.
- Updated `frontend/src/components/LeftSidebar.jsx` to consume route-backed shell navigation sections while keeping icon rendering in the UI component.
- Updated `frontend/src/components/CommandPalette.jsx` to consume route-backed command navigation entries while keeping action commands local.

## Preserved Behavior

- Sidebar brand and all existing sidebar links remain rendered.
- Sidebar active-state matching still supports exact, nested, and query path matches.
- Sidebar navigation still calls `navigateTo(path)`.
- Command palette page navigation entries keep the same ids, labels, section keys, and paths.
- Command palette action commands remain local to `CommandPalette`.

## Public Inputs

- Shell route path helpers.
- Current browser pathname.
- Sidebar icon key mapping from `LeftSidebar`.
- Command palette action definitions from `CommandPalette`.

## Public Outputs

- `SHELL_NAV_SECTIONS`.
- `COMMAND_NAVIGATION_DEFS`.
- `isShellNavPathActive(currentPath, itemPath)`.

## Verification

- From `frontend/`, `npm.cmd test -- src/routing/shellNavigation.test.js src/components/LeftSidebar.test.jsx src/app/AppGlobalOverlays.test.jsx src/routing/navigationDispatch.test.js src/routing/routeContract.test.js src/router.test.js`: passed, 6 test files and 22 tests.
- From `frontend/`, `npm.cmd run build`: passed, 1004 modules transformed.

## Further-Split Decision

No further split inside `frontend.routing.shell_navigation` now. Route-backed sidebar sections, command navigation entries, and active-path matching form a compact public shell navigation contract. UI icon rendering and command action execution intentionally stay with their component owners.

## Residuals

- Feature-page navigation adapters remain with their feature parents until those parents are processed.
- `frontend.routing` parent closeout remains next.
