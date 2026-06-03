# v4.16.0 backend.ops_governance.chaos.experiment_creation parent residual judgment selects memory_commit

> Batch: BE-001OP-01
> Node: `backend.ops_governance.chaos.experiment_creation`
> Parent: `backend.ops_governance.chaos`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.experiment_creation` returns to parent residual judgment after `report_projection` closed as the pure report assembly boundary.

The next child is fixed as:

`backend.ops_governance.chaos.experiment_creation.memory_commit`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.chaos.experiment_creation.memory_commit` | Scoped in-memory insert after persistence succeeds. | Select for next baseline. |

## Selected Child Boundary

`backend.ops_governance.chaos.experiment_creation.memory_commit` currently contains:

- `auth::scoped_key(user_id, experiment_id)` construction;
- write lock acquisition on `state.chaos_experiments`;
- insertion of a cloned report into the scoped in-memory map.

## Hard Boundaries

BE-001OQ-01/02 must not move:

- create-flow route bridge;
- experiment ID generation;
- chaos mode lifecycle;
- evidence metric sampling;
- closed `perturbation_execution` internals;
- closed `report_projection` internals;
- parent-mediated persistence;
- closed `report_persistence` internals;
- list/detail read handlers;
- route facade;
- chaos schema type definitions;
- closed ops siblings, AppState owner, frontend caller, and release transition logic.

No sibling shortcut is allowed. The memory_commit child must remain private to the experiment_creation parent.

## Next Step

BE-001OQ-01 backend.ops_governance.chaos.experiment_creation.memory_commit baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
