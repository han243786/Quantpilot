# v4.16.0 backend.ops_governance parent residual judgment selects alerts

> Batch: BE-001MQ-01
> Node: `backend.ops_governance`
> Parent: `backend`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance` returns to parent residual judgment after `sandbox` closed as a completed parent.

The next child is fixed as:

`backend.ops_governance.alerts`

Selection reasons:

- It is the next real child facade in the parent route order after closed hotswap and sandbox.
- It owns the ops alert route registration facade.
- Its current implementation delegates to the root `alert_engine` handler owner, which must be frozen before any movement decision.
- It is independent from snapshots, runbook, and chaos.

BE-001MR-01 must establish the alerts equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.alerts` | Delegates alert route registration to `crate::alert_engine`. | Select for next baseline. |
| `backend.ops_governance.snapshots` | Delegates snapshot route registration to `crate::snapshot_service`. | Keep in parent residual queue. |
| `backend.ops_governance.runbook` | Delegates runbook route registration to `crate::runbook`. | Keep in parent residual queue. |
| `backend.ops_governance.chaos` | Delegates chaos route registration to `crate::chaos_experiment`. | Keep in parent residual queue. |

Closed children:

- `backend.ops_governance.hotswap`
- `backend.ops_governance.sandbox`

## Selected Child Boundary

`backend.ops_governance.alerts` currently contains:

- `MODULE_ID = "backend.ops_governance.alerts"`;
- `register_routes(router: Router<AppState>) -> Router<AppState>`;
- delegation to `crate::alert_engine::register_alert_routes(router)`.

The child is only a route facade today. The handler owner remains `src/alert_engine.rs`.

## Hard Boundaries

BE-001MR-01/02 must not move:

- snapshots route or handler owner;
- runbook route or handler owner;
- chaos route or handler owner;
- closed hotswap internals;
- closed sandbox internals;
- AppState owner or lock order;
- runtime/capability/storage security internals;
- test support internals;
- release transition policy.

No sibling shortcut is allowed. Alerts may communicate outward only through its parent route facade or root handler boundary until a dedicated baseline changes ownership.

## Next Step

BE-001MR-01 backend.ops_governance.alerts baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
