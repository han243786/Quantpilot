# v4.16.0 backend.ops_governance parent residual judgment selects snapshots

> Batch: BE-001NG-01
> Node: `backend.ops_governance`
> Parent: `backend`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance` returns to parent residual judgment after `alerts` closed as a completed parent.

The next child is fixed as:

`backend.ops_governance.snapshots`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.snapshots` | Delegates snapshot route registration to `crate::snapshot_service`. | Select for next baseline. |
| `backend.ops_governance.runbook` | Delegates runbook route registration to `crate::runbook`. | Keep in parent residual queue. |
| `backend.ops_governance.chaos` | Delegates chaos route registration to `crate::chaos_experiment`. | Keep in parent residual queue. |

Closed children:

- `backend.ops_governance.hotswap`
- `backend.ops_governance.sandbox`
- `backend.ops_governance.alerts`

## Selected Child Boundary

`backend.ops_governance.snapshots` currently contains:

- `MODULE_ID = "backend.ops_governance.snapshots"`;
- `register_routes(router: Router<AppState>) -> Router<AppState>`;
- delegation to `crate::snapshot_service::register_snapshot_routes(router)`.

The handler owner remains `src/snapshot_service.rs`.

Current handler owner includes:

- snapshot route facade;
- create/list/get/restore handlers;
- snapshot persistence;
- disk loading;
- snapshot ID validation;
- restore audit persistence;
- embedded snapshot tests.

## Hard Boundaries

BE-001NH-01/02 must not move:

- runbook route or handler owner;
- chaos route or handler owner;
- closed hotswap, sandbox, or alerts internals;
- AppState owner or lock order;
- runtime/capability/storage security internals;
- DTO schema owner outside the selected snapshot handler boundary;
- release transition policy.

No sibling shortcut is allowed. Snapshots may communicate outward only through its parent route facade or root handler boundary until a dedicated baseline changes ownership.

## Next Step

BE-001NH-01 backend.ops_governance.snapshots baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
