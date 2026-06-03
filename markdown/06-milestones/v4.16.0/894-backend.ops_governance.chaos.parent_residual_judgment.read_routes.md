# v4.16.0 backend.ops_governance.chaos parent residual judgment selects read_routes

> Batch: BE-001OS-01
> Node: `backend.ops_governance.chaos`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos` returns to parent residual judgment after `experiment_creation` closed as the create-flow parent.

The next child is fixed as:

`backend.ops_governance.chaos.read_routes`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.chaos.read_routes` | List/detail read handlers, user scoped filtering, sorting, memory lookup, and disk fallback through parent bridge. | Select for next baseline. |
| `backend.ops_governance.chaos.route_facade` | Route registration for chaos endpoints. | Keep in parent residual queue. |

## Selected Child Boundary

`backend.ops_governance.chaos.read_routes` currently contains:

- `list_experiments`;
- scoped prefix filtering through `auth::scoped_key(user_id, "")`;
- newest-first sort by `executed_at`;
- `get_experiment`;
- scoped in-memory lookup;
- disk fallback through the chaos parent persistence bridge.

## Hard Boundaries

BE-001OT-01/02 must not move:

- closed `experiment_creation` internals;
- closed `report_persistence` internals;
- route facade;
- closed hotswap, sandbox, alerts, snapshots, and runbook internals;
- AppState owner or lock order;
- chaos schema type definitions;
- frontend caller;
- release transition logic.

No sibling shortcut is allowed. Detail reads must use the chaos parent disk-load bridge instead of importing `report_persistence` directly.

## Next Step

BE-001OT-01 backend.ops_governance.chaos.read_routes baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
