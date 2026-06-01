# Frontend Full Feature Tree

Status: initialized empty from frontend-local truth.

This file is the frontend-only full feature tree. It starts blank by design and will be filled only by frontend extraction evidence.

## Root

- `frontend`

## Feature Areas

- `frontend.app_shell`
  - Status: first child leaf extracted; parent remains active.
  - User-visible behavior: React root bootstraps the application, initializes the graph store, renders route content behind the app shell, and hosts desktop/browser shell affordances.
  - Active frontend-local paths:
    - `frontend/src/main.jsx`
    - `frontend/src/App.jsx`
    - `frontend/src/app/AppRoot.jsx`
    - `frontend/src/app/AppRoot.test.jsx`
    - `frontend/src/app/AppShellFallback.jsx`
    - `frontend/src/app/AppShellFallback.test.jsx`
    - `frontend/src/app/installGlobalErrorHandlers.js`
    - `frontend/src/app/installGlobalErrorHandlers.test.js`
    - `frontend/src/app/useAppInitialization.js`
    - `frontend/src/app/useAppInitialization.test.jsx`
  - Evidence:
    - `markdown/00-frontend-refactor-governance/records/FE-0002-frontend-app-shell-baseline.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0003-frontend-app-shell-bootstrap-root-closeout.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0004-frontend-isolated-coverage-gate.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0006-frontend-app-shell-startup-readiness-closeout.md`

## Evidence Rules

Each landed feature area should link to:

- Owning module node.
- User-visible behavior preserved.
- Source files owned by the feature area.
- Equivalence baseline or closeout record.

## Deferred Merge Notes

Do not mirror this file into `markdown/10-overview/overview-full-feature-tree.md` until frontend refactor is fully closed and merge-back is explicitly started.
