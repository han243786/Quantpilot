# Frontend Full Feature Tree

Status: initialized empty from frontend-local truth.

This file is the frontend-only full feature tree. It starts blank by design and will be filled only by frontend extraction evidence.

## Root

- `frontend`

## Feature Areas

- `frontend.app_shell`
  - Status: parent closed.
  - User-visible behavior: React root bootstraps the application, initializes the graph store, renders route content behind the app shell, and hosts desktop/browser shell affordances.
  - Active frontend-local paths:
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
  - Evidence:
    - `markdown/00-frontend-refactor-governance/records/FE-0002-frontend-app-shell-baseline.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0003-frontend-app-shell-bootstrap-root-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0004-frontend-isolated-coverage-gate.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0006-frontend-app-shell-startup-readiness-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0007-frontend-app-shell-environment-events-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0008-frontend-app-shell-desktop-window-chrome-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0009-frontend-app-shell-route-host-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0010-frontend-app-shell-global-overlays-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0011-frontend-app-shell-parent-closeout.md`

- `frontend.routing`
  - Status: parent closed.
  - User-visible behavior: path builders, route parsing, and navigation dispatch keep shell navigation and routed feature entry stable.
  - Active frontend-local paths:
    - `frontend/src/router.js`
    - `frontend/src/router.test.js`
    - `frontend/src/routing/navigationDispatch.js`
    - `frontend/src/routing/navigationDispatch.test.js`
    - `frontend/src/routing/routeContract.js`
    - `frontend/src/routing/routeContract.test.js`
    - `frontend/src/routing/shellNavigation.js`
    - `frontend/src/routing/shellNavigation.test.js`
  - Evidence:
    - `markdown/00-frontend-refactor-governance/records/FE-0012-frontend-routing-baseline.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0013-frontend-routing-route-contract-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0014-frontend-routing-navigation-dispatch-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0015-frontend-routing-shell-navigation-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0016-frontend-routing-parent-closeout.md`

- `frontend.api_client`
  - Status: parent closed.
  - User-visible behavior: frontend HTTP requests resolve the API base, apply timeout handling, send JSON requests, and surface server errors consistently.
  - Active frontend-local paths:
    - `frontend/src/api/apiBase.js`
    - `frontend/src/api/apiBase.test.js`
    - `frontend/src/api/fetchHelpers.js`
    - `frontend/src/api/fetchHelpers.test.js`
    - `frontend/src/api/apiTransport.js`
    - `frontend/src/api/apiTransport.test.js`
    - `frontend/src/api/client.js`
    - `frontend/src/utils/api.js`
  - Evidence:
    - `markdown/00-frontend-refactor-governance/records/FE-0017-frontend-api-client-baseline.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0018-frontend-api-client-base-resolution-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0019-frontend-api-client-request-transport-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0020-frontend-api-client-compat-fetch-helpers-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0021-frontend-api-client-parent-closeout.md`

- `frontend.capabilities`
  - Status: parent baseline established.
  - User-visible behavior: backend capability snapshots gate frontend modules, workspace surfaces, toolbar/runtime actions, and safe fallback messaging.
  - Active frontend-local paths:
    - `frontend/src/capabilities/supportMatrix.js`
    - `frontend/src/capabilities/supportMatrix.test.js`
    - `frontend/src/capabilities/capabilityActionBlocks.js`
    - `frontend/src/capabilities/capabilityActionBlocks.test.js`
    - `frontend/src/capabilities/capabilityCatalog.js`
    - `frontend/src/capabilities/capabilityCatalog.test.js`
    - `frontend/src/capabilities/capabilityBoundary.js`
    - `frontend/src/capabilities/capabilityBoundary.test.js`
    - `frontend/src/capabilities/capabilitySync.js`
    - `frontend/src/capabilities/capabilitySync.test.js`
    - `frontend/src/capabilities/capabilityProjection.js`
    - `frontend/src/capabilities/capabilityProjection.test.js`
    - `frontend/src/capabilities/capabilityGovernanceCore.js`
    - `frontend/src/capabilities/capabilityGovernanceCore.test.js`
    - `frontend/src/capabilities/capabilityGovernanceRegistry.js`
    - `frontend/src/capabilities/capabilityGovernanceRegistry.test.js`
    - `frontend/src/capabilities/capabilityGovernance.js`
    - `frontend/src/capabilities/capabilityGovernance.test.js`
    - `frontend/src/modules/moduleRegistry.js`
    - `frontend/src/modules/moduleRegistry.test.js`
    - `frontend/src/modules/builtinModules.js`
    - `frontend/src/store/graphStore.capabilities.test.js`
  - Evidence:
    - `markdown/00-frontend-refactor-governance/records/FE-0022-frontend-capabilities-baseline.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0023-frontend-capabilities-sync-block-gate-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0024-frontend-capabilities-catalog-maps-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0025-frontend-capabilities-boundary-context-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0026-frontend-capabilities-action-block-reason-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0027-frontend-capabilities-projection-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0028-frontend-capabilities-governance-core-contract-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0029-frontend-capabilities-governance-registry-entries-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0030-frontend-capabilities-governance-public-facade-closeout.md`

## Evidence Rules

Each landed feature area should link to:

- Owning module node.
- User-visible behavior preserved.
- Source files owned by the feature area.
- Equivalence baseline or closeout record.

## Deferred Merge Notes

Do not mirror this file into `markdown/10-overview/overview-full-feature-tree.md` until frontend refactor is fully closed and merge-back is explicitly started.
