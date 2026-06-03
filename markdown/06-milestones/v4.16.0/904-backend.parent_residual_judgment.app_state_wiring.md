# v4.16.0 backend parent residual judgment selects app_state_wiring

> Batch: BE-001OY-01
> Node: `backend`
> Selected child: `backend.app_state_wiring`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend` returns to its top-level residual queue after `backend.ops_governance` closed.

Decision:

`next_child: backend.app_state_wiring`

## Closed Backend Children

Already closed in the current recursive scope:

- `backend.interface_boundary`;
- `backend.runtime`;
- `backend.graph_compile`;
- `backend.capability`;
- `backend.strategy_config`;
- `backend.storage_security`;
- `backend.ops_governance`.

## Open Backend Residuals

| Residual | Status |
| --- | --- |
| `backend.app_state_wiring` | Selected next. Existing facade owns health route bridge, state factory re-export, and `attach_state`. |
| `backend.test_support` | Still queued. Test asset retirement remains out of scope until a later plan. |

## Selection Rationale

`backend.app_state_wiring` is the next safe residual because it is already a thin wiring facade:

- `src/backend/app_state_wiring.rs` exposes `health`, `attach_state`, and `new_app_state`;
- `src/backend/app_state_wiring/health_route.rs` delegates to `app_runtime_helpers::health`;
- `src/backend/app_state_wiring/state_factory.rs` re-exports `app_runtime_helpers::new_app_state`;
- AppState fields, locks, state directory ownership, schema, and startup chain semantics remain outside the child movement scope.

## Hard Boundaries

The next `backend.app_state_wiring` closeout must not move:

- `AppState` field ownership;
- lock order;
- runtime/capability/storage/ops child state;
- health response schema;
- frontend caller;
- release transition logic.

## Next Step

BE-001OZ-01 `backend.app_state_wiring` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
