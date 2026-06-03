# v4.16.0 backend.ops_governance.chaos parent residual judgment selects route_facade

> Batch: BE-001OU-01
> Node: `backend.ops_governance.chaos`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos` returns to parent residual judgment after `read_routes` closed as the read handler boundary.

The next child is fixed as:

`backend.ops_governance.chaos.route_facade`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.chaos.route_facade` | Route registration for create/list/detail chaos endpoints. | Select for next baseline. |

## Selected Child Boundary

`backend.ops_governance.chaos.route_facade` currently contains:

- `POST /api/v1/chaos/experiments`;
- `GET /api/v1/chaos/experiments`;
- `GET /api/v1/chaos/experiments/:experiment_id`;
- binding of those routes to parent-owned create/list/detail handler bridges.

## Hard Boundaries

BE-001OV-01/02 must not move:

- closed `experiment_creation` internals;
- closed `read_routes` internals;
- closed `report_persistence` internals;
- closed hotswap, sandbox, alerts, snapshots, and runbook internals;
- AppState owner or lock order;
- chaos schema type definitions;
- frontend caller;
- release transition logic.

No sibling shortcut is allowed. Route facade must bind only to chaos parent handler bridges.

## Next Step

BE-001OV-01 backend.ops_governance.chaos.route_facade baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
