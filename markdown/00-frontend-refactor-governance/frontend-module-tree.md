# Frontend Module Tree

Status: initialized empty from frontend-local truth.

This is the frontend-only module tree for parallel refactor work. It is not copied from the global tree and must not be treated as merged global truth.

## Root

- `root.frontend`

## Active Parent

- `frontend.app_shell`
  - Status: parent baseline established.
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0002-frontend-app-shell-baseline.md`
  - Current owned files:
    - `frontend/src/main.jsx`
    - `frontend/src/App.jsx`
    - `frontend/src/app/AppRoot.jsx`
    - `frontend/src/app/AppRoot.test.jsx`
    - `frontend/src/app/installGlobalErrorHandlers.js`
    - `frontend/src/app/installGlobalErrorHandlers.test.js`
  - Child queue:
    - `frontend.app_shell.startup_readiness`
    - `frontend.app_shell.environment_events`
    - `frontend.app_shell.desktop_window_chrome`
    - `frontend.app_shell.route_host`
    - `frontend.app_shell.global_overlays`
  - Closed child leaves:
    - `frontend.app_shell.bootstrap_root`

## Pending Parent Queue

- `frontend.routing`
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

None yet.

## Deferred Merge Notes

Global module tree merge is deferred until frontend refactor closeout.
