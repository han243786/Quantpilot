# FE-0224 - Frontend Global E2E Preflight Handoff

Status: closed.

## Scope

`root.frontend`

## Change

- Added `frontend-global-merge-back-map.md` as the frozen frontend-local to global-governance merge-back map.
- Added `frontend-e2e-current-state-inventory.md` as the E2E spec/support inventory for post-backend cleanup.
- Added `frontend-backend-main-thread-handoff-prompt.md` as the copy-ready backend main process notification prompt.
- Updated `frontend-full-feature-tree.md` to remove stale active/baseline labels and register `frontend.test_support` as a closed parent.
- Advanced the frontend recursive state to `frontend_global_e2e_preflight_ready`.

## Preserved Behavior

- No source code changed.
- No E2E spec body changed.
- No global governance file changed.
- Global merge-back and E2E cleanup remain deferred until backend refactor closeout.

## Verification

- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`: passed.
- From repo root, `powershell -ExecutionPolicy Bypass -File tools/check-matrix-governance.ps1`: passed.
