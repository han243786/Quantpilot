# v4.16.0 backend.ops_governance.chaos parent residual judgment selects experiment_creation

> Batch: BE-001OJ-01
> Node: `backend.ops_governance.chaos`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos` returns to parent residual judgment after `report_persistence` closed as the chaos disk persistence and ID validation boundary.

The next child is fixed as:

`backend.ops_governance.chaos.experiment_creation`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.chaos.experiment_creation` | Create handler, chaos mode toggling, perturbation execution, metrics, pass criteria, alert/degradation action assembly, report assembly, and in-memory insert. | Select for next baseline. |
| `backend.ops_governance.chaos.read_routes` | List/detail read handlers after create flow separates. | Keep in parent residual queue. |
| `backend.ops_governance.chaos.route_facade` | Route registration for chaos endpoints. | Keep in parent residual queue. |

## Selected Child Boundary

`backend.ops_governance.chaos.experiment_creation` currently contains:

- experiment ID generation;
- `chaos_mode` enable/disable lifecycle;
- evidence metric sampling before and after the injection;
- max-duration environment clamp;
- perturbation execution for disk pressure, data latency, event loss, and clock skew;
- steady-state metric projection before, during, and after the experiment;
- pass/fail criteria;
- alert and degradation action assembly;
- chaos report assembly;
- persistence through the chaos parent bridge;
- in-memory experiment insertion.

## Hard Boundaries

BE-001OK-01/02 must not move:

- closed `report_persistence` internals;
- list/detail read handlers;
- route facade;
- closed hotswap, sandbox, alerts, snapshots, and runbook internals;
- AppState owner or lock order;
- chaos schema type definitions;
- frontend caller;
- release transition logic.

No sibling shortcut is allowed. The create-flow child must use the chaos parent persistence bridge instead of importing or calling `handlers::report_persistence` directly.

## Next Step

BE-001OK-01 backend.ops_governance.chaos.experiment_creation baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
