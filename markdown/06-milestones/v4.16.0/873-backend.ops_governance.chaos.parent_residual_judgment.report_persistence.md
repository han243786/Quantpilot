# v4.16.0 backend.ops_governance.chaos parent residual judgment selects report_persistence

> Batch: BE-001OH-01
> Node: `backend.ops_governance.chaos`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos` returns to parent residual judgment after the first chaos handler extraction stayed open for internal split.

The next child is fixed as:

`backend.ops_governance.chaos.report_persistence`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.chaos.report_persistence` | `persist_chaos_report`, `load_chaos_report_from_disk`, and `validate_experiment_id`. | Select for next baseline. |
| `backend.ops_governance.chaos.experiment_creation` | Create handler, chaos mode toggling, perturbation execution, metrics, and report assembly. | Keep in parent residual queue. |
| `backend.ops_governance.chaos.read_routes` | List/detail read handlers after persistence separates. | Keep in parent residual queue. |
| `backend.ops_governance.chaos.route_facade` | Route registration for chaos endpoints. | Keep in parent residual queue. |

## Selected Child Boundary

`backend.ops_governance.chaos.report_persistence` currently contains:

- report storage quota check;
- report directory creation;
- atomic JSON report write;
- experiment ID validation;
- disk read fallback;
- report JSON deserialization and error mapping.

## Hard Boundaries

BE-001OI-01/02 must not move:

- create experiment handler;
- list/detail route handlers;
- route facade;
- closed hotswap, sandbox, alerts, snapshots, and runbook internals;
- AppState owner or lock order;
- chaos schema type definitions;
- frontend caller;
- release transition logic.

No sibling shortcut is allowed. Create/detail handlers must consume persistence through the chaos parent bridge until their own baseline changes ownership.

## Next Step

BE-001OI-01 backend.ops_governance.chaos.report_persistence baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
