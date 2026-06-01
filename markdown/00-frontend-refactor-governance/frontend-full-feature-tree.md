# Frontend Full Feature Tree

Status: initialized empty from frontend-local truth.

This file is the frontend-only full feature tree. It starts blank by design and will be filled only by frontend extraction evidence.

## Root

- `frontend`

## Feature Areas

- `frontend.app_shell`
  - Status: first child leaf extracted; parent remains active.
  - User-visible behavior: React root bootstraps the application, initializes the graph store, renders route content behind the app shell, and hosts desktop/browser shell affordances.
  - Evidence:
    - `markdown/00-frontend-refactor-governance/records/FE-0002-frontend-app-shell-baseline.md`
    - `markdown/00-frontend-refactor-governance/records/FE-0003-frontend-app-shell-bootstrap-root-closeout.md`

## Evidence Rules

Each landed feature area should link to:

- Owning module node.
- User-visible behavior preserved.
- Source files owned by the feature area.
- Equivalence baseline or closeout record.

## Deferred Merge Notes

Do not mirror this file into `markdown/10-overview/overview-full-feature-tree.md` until frontend refactor is fully closed and merge-back is explicitly started.
