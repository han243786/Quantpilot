# FE-0004 Frontend Isolated Coverage Gate

Status: closed.

## Problem

After `frontend.app_shell.bootstrap_root` created new code under `frontend/src/app`, the repository full feature tree gate reported the files as uncovered because the global tree intentionally remains untouched during frontend isolated recursion.

## Decision

Keep the global tree frozen for frontend isolated recursion, and let the full feature tree checker use the frontend-local full feature tree as a coverage supplement.

This preserves both constraints:

- Global governance files remain low-conflict during parallel backend/frontend work.
- New active frontend files still have exact repo-relative path coverage before commit.

## Changed Files

- `tools/check-full-feature-tree.ps1`
- `markdown/00-frontend-refactor-governance/frontend-full-feature-tree.md`
- `markdown/00-frontend-refactor-governance/frontend-recursive-state.json`

## Coverage Supplement

Default supplement:

- `markdown/00-frontend-refactor-governance/frontend-full-feature-tree.md`

The checker still performs version, structure, stale marker, and explicit path checks against the global full feature tree. The supplement only extends active file coverage.

## Verification

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`: passed, including frontend coverage supplement.
- `git diff --check`: passed.
- `frontend-recursive-state.json` JSON parse: passed.

## Further-Split Decision

No further split is useful. This is a governance gate compatibility leaf, not a product module leaf.
