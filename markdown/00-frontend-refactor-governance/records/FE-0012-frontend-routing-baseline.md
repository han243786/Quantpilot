# FE-0012 Frontend Routing Baseline

Status: baseline established.

## Parent Node

`frontend.routing`

## Current Scope

Frontend routing currently centers on `frontend/src/router.js`, with broad consumers across the app shell, shell navigation, command palette, strategy pages, backtest pages, utility actions, and test support.

## Initial Child Queue

- `frontend.routing.route_contract`
- `frontend.routing.navigation_dispatch`
- `frontend.routing.shell_navigation`

## Current Owned Files

- `frontend/src/router.js`
- `frontend/src/router.test.js`

## Important Consumers

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

## Whitebox Contract

### Public Inputs

- Browser pathname and search string.
- Strategy IDs and backtest IDs that need safe URL encoding.
- User navigation actions from shell and feature modules.
- Browser history state and current hash during navigation.

### Public Outputs

- Stable path builders for strategy, backtest, approval, alert, snapshot, runbook, chaos, settings, and QuantScript routes.
- Parsed route objects consumed by `AppRouteHost`.
- `navigateTo(pathname)` history update and synthetic `popstate` event.

## Equivalence Anchors

- `frontend/src/router.test.js`.
- `frontend/src/app/useAppRoute.test.jsx`.
- `frontend/src/components/LeftSidebar.test.jsx`.
- Existing page tests that mock or consume `../router`.
- Frontend build.

## Split Rules

- Keep compatibility exports at `frontend/src/router.js` until all consumers are migrated intentionally.
- Do not change route names, path shapes, query keys, dedup timing, or synthetic `popstate` behavior during extraction.
- Do not mix feature page refactors into routing unless a consumer import must move for the current leaf.

## First Leaf

`frontend.routing.route_contract`
