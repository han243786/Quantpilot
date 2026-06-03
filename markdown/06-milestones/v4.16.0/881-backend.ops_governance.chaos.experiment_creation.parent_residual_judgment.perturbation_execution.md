# v4.16.0 backend.ops_governance.chaos.experiment_creation parent residual judgment selects perturbation_execution

> Batch: BE-001OL-01
> Node: `backend.ops_governance.chaos.experiment_creation`
> Parent: `backend.ops_governance.chaos`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.experiment_creation` returns to parent residual judgment after the first create-flow extraction stayed open for internal split.

The next child is fixed as:

`backend.ops_governance.chaos.experiment_creation.perturbation_execution`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.chaos.experiment_creation.perturbation_execution` | Max-duration resolution and side-effect execution for disk pressure, data latency, event loss, and clock skew. | Select for next baseline. |
| `backend.ops_governance.chaos.experiment_creation.report_projection` | Metrics before/during/after, pass criteria, alerts, degradation actions, and report assembly. | Keep in parent residual queue. |
| `backend.ops_governance.chaos.experiment_creation.memory_commit` | Scoped in-memory insert after persistence succeeds. | Keep in parent residual queue. |

## Selected Child Boundary

`backend.ops_governance.chaos.experiment_creation.perturbation_execution` currently contains:

- default max-duration constant;
- `QUANTPILOT_CHAOS_MAX_DURATION_MS` parsing;
- duration clamp;
- disk pressure temp directory creation;
- 10 MiB pressure file write loop;
- perturbation sleep;
- disk pressure temp directory cleanup;
- latency, event loss, and clock skew sleep behavior.

## Hard Boundaries

BE-001OM-01/02 must not move:

- create-flow route bridge;
- chaos mode lifecycle;
- evidence metric sampling;
- metric projection, pass criteria, alert/action assembly, or report assembly;
- parent-mediated persistence;
- memory commit;
- closed `report_persistence` internals;
- list/detail read handlers;
- route facade;
- chaos schema type definitions;
- closed ops siblings, AppState owner, frontend caller, and release transition logic.

No sibling shortcut is allowed. The perturbation child must remain private to the experiment_creation parent.

## Next Step

BE-001OM-01 backend.ops_governance.chaos.experiment_creation.perturbation_execution baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
