# FE-0040 Frontend Strategy Workspace Route Shell Closeout

Status: closed.

## Leaf Node

`frontend.strategy_workspace.route_shell`

## Code Changes

- Added `frontend/src/pages/strategyWorkspaceRouteShell.js`.
- Added `frontend/src/pages/strategyWorkspaceRouteShell.test.js`.
- Updated `frontend/src/pages/StrategyWorkspacePage.jsx` to consume route-shell helpers for workspace tab definitions, capability visibility, visited-tab mounting, and tab panel props.

## Preserved Behavior

- Workspace tab labels, kicker labels, and inspector panel definitions stay unchanged.
- Capability-hidden workspace surfaces still do not render tabs.
- Disabled-but-visible workspace surfaces still render disabled tabs with their capability state.
- Previously visited workspace tabs still stay mounted while inactive.
- Tab panels still use the same class name, display style, and `aria-hidden` behavior.

## Public Inputs

- Capability projection view.
- Active workspace tab id.
- Visited workspace tab set.
- Requested surface key.

## Public Outputs

- `WORKSPACE_TAB_DEFS`.
- `CODE_INSPECTOR_DEFS`.
- `buildWorkspaceTabs(capabilityView)`.
- `isWorkspaceSurfaceVisible(capabilityView, surfaceKey)`.
- `shouldMountWorkspaceTab({ capabilityView, activeTab, visitedTabs, surfaceKey })`.
- `buildWorkspaceTabPanelProps(activeTab, surfaceKey)`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/pages/strategyWorkspaceRouteShell.test.js src/pages/StrategyWorkspacePage.codeMode.test.jsx src/components/TopToolbar.capabilities.test.jsx`: passed, 3 test files and 15 tests.

## Further-Split Decision

`frontend.strategy_workspace.route_shell` does not need further split now. The route shell now exposes compact pure helpers while `StrategyWorkspacePage.jsx` remains the public route gateway.

## Residuals

- Continue with `frontend.strategy_workspace.shared_model_and_page_data`.
