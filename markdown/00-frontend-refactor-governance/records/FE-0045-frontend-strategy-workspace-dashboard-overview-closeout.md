# FE-0045 Frontend Strategy Workspace Dashboard Overview Closeout

Status: closed.

## Leaf Node

`frontend.strategy_workspace.dashboard_overview`

## Code Changes

- Added `frontend/src/pages/strategyWorkspaceDashboardOverviewShell.js`.
- Added `frontend/src/pages/strategyWorkspaceDashboardOverviewShell.test.js`.
- Updated `frontend/src/pages/StrategyWorkspaceDashboard.jsx` to delegate latest runtime selection, backtest counting, quick-action class names, surface blocking state, and surface blocking title projection to the extracted shell module.
- Updated `frontend/src/pages/StrategyWorkspaceOverviewTab.jsx` to delegate overview action-card projection to the extracted shell module while keeping route and tab callback binding inside the page.

## Preserved Behavior

- Dashboard still prefers store runtime over the fallback runtime prop.
- Dashboard backtest count still reads `runtime.backtestHistory.length` with a zero fallback.
- Dashboard quick-action buttons still route to `code`, `research`, `monitor`, and `source` through the parent-owned `onNavigate` callback.
- Surface blocking still disables dashboard quick-action buttons and keeps `blockReason` ahead of `reason` as the button title.
- Overview action cards still show the same build, diagnostics, and research entry points and preserve the existing tab/route targets.

## Public Inputs

- Store runtime and fallback runtime.
- Workspace surface capability map.
- Workspace graph node and edge counts.
- Compile diagnostic counts.
- Recent run and backtest lists.

## Public Outputs

- `WORKSPACE_DASHBOARD_QUICK_ACTIONS`.
- `resolveWorkspaceDashboardRuntime(storeRuntime, fallbackRuntime)`.
- `countWorkspaceDashboardBacktests(runtime)`.
- `canNavigateWorkspaceSurface(workspaceSurfaces, surfaceKey)`.
- `getWorkspaceSurfaceNavigationTitle(workspaceSurfaces, surfaceKey)`.
- `buildWorkspaceDashboardQuickActions(workspaceSurfaces)`.
- `buildWorkspaceOverviewActionCards({ graph, compileCounts, recentRuns, recentBacktests })`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/pages/strategyWorkspaceDashboardOverviewShell.test.js src/pages/StrategyWorkspacePage.codeMode.test.jsx`: passed, 2 test files and 10 tests.

## Further-Split Decision

`frontend.strategy_workspace.dashboard_overview` does not need a deeper split yet. The dashboard and overview entry projections are now isolated from React rendering, while hero copy, metric rendering, issue queue rendering, and side-card composition remain page-owned or delegated to already separated child leaves.

## Residuals

- Continue with `frontend.strategy_workspace.monitor_research_source_tabs`.
