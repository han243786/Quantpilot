# v4.16.0 backend.ops_governance.chaos.experiment_creation.report_projection actual extraction complete

> Batch: BE-001OO-02
> Node: `backend.ops_governance.chaos.experiment_creation.report_projection`
> Parent: `backend.ops_governance.chaos.experiment_creation`
> Stage: `extract_closeout`
> Movement: Chaos report projection moved into a private child module.

---

## Summary

`backend.ops_governance.chaos.experiment_creation.report_projection` now owns pure metrics projection, pass criteria, alert/action assembly, and report construction.

The experiment_creation parent keeps local `baseline_metrics` and `build_experiment_report` bridges and continues to own orchestration, side-effect timing, persistence, and memory commit.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/chaos/handlers/experiment_creation.rs` | `src/backend/ops_governance/chaos/handlers/experiment_creation/report_projection.rs` | Baseline metrics, during/after metrics, pass criteria, alert/action assembly, and report construction moved. |
| `src/backend/ops_governance/chaos/handlers/experiment_creation.rs` | `src/backend/ops_governance/chaos/handlers/experiment_creation.rs` | Parent declares the private child and keeps local projection bridges. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| Baseline metrics | Before metrics remain `120.0` freshness and `4.0` planned rate. |
| During metrics | All four chaos types keep the same during-metric projection. |
| After metrics | After metrics still add `5.0` freshness and subtract `0.1` planned rate. |
| Pass criteria | Each chaos type keeps the same pass/fail expression. |
| Alerts/actions | Alert and degradation action vectors keep the same values. |
| Report fields | ID, type, executed_at conversion, injection, metrics, recovery duration, passed flag, and notes are unchanged. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- experiment_creation parent create flow -> experiment_creation parent report projection bridge;
- experiment_creation parent report projection bridge -> private `report_projection::*`;
- report_projection child -> pure schema value construction only.

The following remain outside this child:

- route bridge;
- experiment ID generation;
- chaos mode lifecycle;
- evidence metric sampling;
- closed perturbation_execution;
- persistence and memory commit;
- closed report_persistence, read routes, route facade, closed ops siblings, AppState owner, frontend caller, and release transition logic.

## Proof

- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`

## Next Step

BE-001OO-03 backend.ops_governance.chaos.experiment_creation.report_projection single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
