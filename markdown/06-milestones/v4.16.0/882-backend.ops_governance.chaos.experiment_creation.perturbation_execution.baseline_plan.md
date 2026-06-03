# v4.16.0 backend.ops_governance.chaos.experiment_creation.perturbation_execution equivalence baseline and extraction plan

> Batch: BE-001OM-01
> Node: `backend.ops_governance.chaos.experiment_creation.perturbation_execution`
> Parent: `backend.ops_governance.chaos.experiment_creation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.experiment_creation.perturbation_execution` is frozen as the side-effect perturbation execution owner for chaos experiment creation.

BE-001OM-01 does not move code. It defines the exact baseline and allowed movement for BE-001OM-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/chaos/handlers/experiment_creation.rs`

Current selected boundary:

- `DEFAULT_CHAOS_MAX_DURATION_MS`;
- `QUANTPILOT_CHAOS_MAX_DURATION_MS` parsing;
- clamped duration calculation;
- disk pressure temp directory lifecycle;
- 10 MiB pressure file write loop;
- latency, event loss, and clock skew sleeps.

The parent bridge must remain:

- create-flow parent calls a local perturbation execution bridge;
- perturbation execution child remains private to the create-flow parent.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Default duration | Default clamp remains 10 seconds. |
| Environment override | `QUANTPILOT_CHAOS_MAX_DURATION_MS` is used only when present and parseable as `u64`. |
| Clamp | Each perturbation sleeps for `request.injection.duration_ms.min(max_duration_ms)`. |
| Disk pressure path | Disk pressure still writes under `<chaos_store_dir>/temp_pressure`. |
| Disk pressure write | Disk pressure still writes ten 1 MiB files named `pressure_<n>.bin`. |
| Disk cleanup | Disk pressure still removes the temp directory after the sleep. |
| Latency/event loss/clock skew | These variants still perform only the clamped sleep. |

## Allowed BE-001OM-02 Movement

BE-001OM-02 may:

- create `src/backend/ops_governance/chaos/handlers/experiment_creation/perturbation_execution.rs`;
- add a private `mod perturbation_execution;` declaration in `src/backend/ops_governance/chaos/handlers/experiment_creation.rs`;
- move only max-duration resolution and perturbation side-effect execution into that private child;
- keep a parent-owned `execute_perturbation` bridge in `experiment_creation.rs`;
- add local unit tests for max-duration resolution if the implementation exposes a private helper.

## Forbidden BE-001OM-02 Movement

BE-001OM-02 must not move or rewrite:

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

## Parent-Child Rule

Allowed call paths:

- experiment_creation parent create flow -> experiment_creation parent perturbation bridge;
- experiment_creation parent perturbation bridge -> private `perturbation_execution::*`;
- perturbation_execution child -> runtime fs/sleep primitives.

Forbidden call path:

Any perturbation_execution child importing read routes, route facade, report persistence, or ops-governance siblings.

## Proof

BE-001OM-02 must prove equivalence with:

- `cargo test -p quantpilot chaos`
- `cargo check -p quantpilot`

## Next Step

BE-001OM-02 backend.ops_governance.chaos.experiment_creation.perturbation_execution extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
