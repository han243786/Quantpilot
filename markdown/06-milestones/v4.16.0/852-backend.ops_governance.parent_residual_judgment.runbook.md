# v4.16.0 backend.ops_governance parent residual judgment selects runbook

> Batch: BE-001NW-01
> Node: `backend.ops_governance`
> Parent: `backend`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance` returns to parent residual judgment after `snapshots` closed as a completed parent.

The next child is fixed as:

`backend.ops_governance.runbook`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.runbook` | Delegates runbook route registration to `crate::runbook`. | Select for next baseline. |
| `backend.ops_governance.chaos` | Delegates chaos route registration to `crate::chaos_experiment`. | Keep in parent residual queue. |

Closed children:

- `backend.ops_governance.hotswap`
- `backend.ops_governance.sandbox`
- `backend.ops_governance.alerts`
- `backend.ops_governance.snapshots`

## Selected Child Boundary

`backend.ops_governance.runbook` currently contains:

- `MODULE_ID = "backend.ops_governance.runbook"`;
- `register_routes(router: Router<AppState>) -> Router<AppState>`;
- delegation to `crate::runbook::register_runbook_routes(router)`.

The handler owner remains `src/runbook.rs`.

Current handler owner includes:

- runbook route facade;
- list scenarios handler;
- get scenario handler;
- default runbook catalog builder;
- embedded runbook catalog tests.

## Hard Boundaries

BE-001NX-01/02 must not move:

- chaos route or handler owner;
- closed hotswap, sandbox, alerts, or snapshots internals;
- AppState owner or lock order;
- runtime/capability/storage security internals;
- DTO schema owner outside the selected runbook handler boundary;
- release transition policy.

No sibling shortcut is allowed. Runbook may communicate outward only through its parent route facade or root handler boundary until a dedicated baseline changes ownership.

## Next Step

BE-001NX-01 backend.ops_governance.runbook baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
