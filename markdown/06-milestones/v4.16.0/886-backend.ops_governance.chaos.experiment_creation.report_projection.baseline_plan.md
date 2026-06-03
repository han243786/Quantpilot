# v4.16.0 backend.ops_governance.chaos.experiment_creation.report_projection equivalence baseline and extraction plan

> Batch: BE-001OO-01
> Node: `backend.ops_governance.chaos.experiment_creation.report_projection`
> Parent: `backend.ops_governance.chaos.experiment_creation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.experiment_creation.report_projection` is frozen as the pure metrics and report assembly owner for chaos experiment creation.

BE-001OO-01 does not move code. It defines the exact baseline and allowed movement for BE-001OO-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/chaos/handlers/experiment_creation.rs`

Current selected boundary:

- baseline `ChaosSteadyStateMetrics`;
- during-experiment metric projection;
- after-experiment metric projection;
- pass/fail criteria;
- alert vector assembly;
- degradation action vector assembly;
- `ChaosExperimentReport` assembly.

The parent bridge must remain:

- create-flow parent requests baseline metrics before perturbation;
- create-flow parent requests report assembly after perturbation.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Baseline metrics | Before metrics remain `data_freshness_p95_ms = 120.0` and `execution_planned_rate_per_min = 4.0`. |
| During latency | Data latency adds injection value to freshness and sets planned rate to `0.0`. |
| During event loss | Event loss keeps freshness and multiplies planned rate by `0.99`. |
| During disk pressure | Disk pressure adds `200.0` freshness and multiplies planned rate by `0.7`. |
| During clock skew | Clock skew adds `500.0` freshness and multiplies planned rate by `0.8`. |
| After metrics | After metrics add `5.0` freshness and subtract `0.1` planned rate. |
| Pass criteria | Each chaos type keeps the same pass/fail expression. |
| Alerts/actions | Alert and degradation action vectors keep the same values. |
| Report fields | Report assembly keeps ID, type, executed_at conversion, injection, metrics, recovery duration, passed flag, and notes unchanged. |

## Allowed BE-001OO-02 Movement

BE-001OO-02 may:

- create `src/backend/ops_governance/chaos/handlers/experiment_creation/report_projection.rs`;
- add a private `mod report_projection;` declaration in `src/backend/ops_governance/chaos/handlers/experiment_creation.rs`;
- move only pure metrics projection, pass criteria, alert/action assembly, and report construction into that private child;
- keep parent-owned bridge functions for baseline metrics and report assembly;
- add local projection tests if useful.

## Forbidden BE-001OO-02 Movement

BE-001OO-02 must not move or rewrite:

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

## Parent-Child Rule

Allowed call paths:

- experiment_creation parent create flow -> experiment_creation parent report projection bridge;
- experiment_creation parent report projection bridge -> private `report_projection::*`;
- report_projection child -> pure schema value construction only.

Forbidden call path:

Any report_projection child importing perturbation execution, report persistence, read routes, route facade, AppState, or ops-governance siblings.

## Proof

BE-001OO-02 must prove equivalence with:

- `cargo test -p quantpilot chaos`
- `cargo check -p quantpilot`

## Next Step

BE-001OO-02 backend.ops_governance.chaos.experiment_creation.report_projection extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
