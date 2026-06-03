# v4.16.0 backend.ops_governance.chaos.experiment_creation parent residual judgment selects report_projection

> Batch: BE-001ON-01
> Node: `backend.ops_governance.chaos.experiment_creation`
> Parent: `backend.ops_governance.chaos`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.experiment_creation` returns to parent residual judgment after `perturbation_execution` closed as the side-effect execution boundary.

The next child is fixed as:

`backend.ops_governance.chaos.experiment_creation.report_projection`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.chaos.experiment_creation.report_projection` | Metrics before/during/after, pass criteria, alert/degradation actions, and report assembly. | Select for next baseline. |
| `backend.ops_governance.chaos.experiment_creation.memory_commit` | Scoped in-memory insert after persistence succeeds. | Keep in parent residual queue. |

## Selected Child Boundary

`backend.ops_governance.chaos.experiment_creation.report_projection` currently contains:

- baseline steady-state metrics;
- during-experiment metric projection for each chaos type;
- after-experiment metric projection;
- pass/fail criteria for each chaos type;
- alert vector assembly;
- degradation action vector assembly;
- `ChaosExperimentReport` assembly.

## Hard Boundaries

BE-001OO-01/02 must not move:

- create-flow route bridge;
- experiment ID generation;
- chaos mode lifecycle;
- evidence metric sampling;
- closed `perturbation_execution` internals;
- parent-mediated persistence;
- memory commit;
- closed `report_persistence` internals;
- list/detail read handlers;
- route facade;
- chaos schema type definitions;
- closed ops siblings, AppState owner, frontend caller, and release transition logic.

No sibling shortcut is allowed. The report_projection child must remain private to the experiment_creation parent.

## Next Step

BE-001OO-01 backend.ops_governance.chaos.experiment_creation.report_projection baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
