# Frontend Module Tree

Status: initialized empty from frontend-local truth.

This is the frontend-only module tree for parallel refactor work. It is not copied from the global tree and must not be treated as merged global truth.

## Root

- `root.frontend`

## Active Parent

- `frontend.routing`
  - Status: parent baseline established.
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0012-frontend-routing-baseline.md`
  - Current owned files:
    - `frontend/src/router.js`
    - `frontend/src/router.test.js`
  - Important consumers:
    - `frontend/src/app/useAppRoute.js`
    - `frontend/src/App.jsx`
    - `frontend/src/components/LeftSidebar.jsx`
    - `frontend/src/components/CommandPalette.jsx`
    - `frontend/src/components/BacktestHistorySection.jsx`
    - `frontend/src/components/EventStreamPanel.jsx`
    - `frontend/src/hooks/useStrategyDirectoryModel.js`
    - `frontend/src/pages/*`
    - `frontend/src/utils/*Actions.js`
    - `frontend/src/test/testBridge.js`
  - Child queue:
    - `frontend.routing.route_contract`
    - `frontend.routing.navigation_dispatch`
    - `frontend.routing.shell_navigation`
  - Closed child leaves: none.

## Last Closed Parent

- `frontend.app_shell`
  - Status: parent closed.
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0002-frontend-app-shell-baseline.md`
  - Closeout: `markdown/00-frontend-refactor-governance/records/FE-0011-frontend-app-shell-parent-closeout.md`
  - Current owned files:
    - `frontend/src/main.jsx`
    - `frontend/src/App.jsx`
    - `frontend/src/app/AppGlobalOverlays.jsx`
    - `frontend/src/app/AppGlobalOverlays.test.jsx`
    - `frontend/src/app/DesktopTitleBar.jsx`
    - `frontend/src/app/DesktopTitleBar.test.jsx`
    - `frontend/src/app/AppRouteHost.jsx`
    - `frontend/src/app/AppRouteHost.test.jsx`
    - `frontend/src/app/AppRoot.jsx`
    - `frontend/src/app/AppRoot.test.jsx`
    - `frontend/src/app/AppShellFallback.jsx`
    - `frontend/src/app/AppShellFallback.test.jsx`
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
  - Child queue: closed.
  - Closed child leaves:
    - `frontend.app_shell.bootstrap_root`
    - `frontend.app_shell.startup_readiness`
    - `frontend.app_shell.environment_events`
    - `frontend.app_shell.desktop_window_chrome`
    - `frontend.app_shell.route_host`
    - `frontend.app_shell.global_overlays`

## Pending Parent Queue

- `frontend.api_client`
- `frontend.capabilities`
- `frontend.strategy_workspace`
- `frontend.strategy_hub`
- `frontend.graph_editor`
- `frontend.runtime_panels`
- `frontend.backtest_views`
- `frontend.store`
- `frontend.design_system_styles`
- `frontend.test_support`

## Closed Nodes

- `frontend.app_shell`

## Deferred Merge Notes

Global module tree merge is deferred until frontend refactor closeout.
