# v4.16.0 backend.ops_governance.chaos.experiment_creation.perturbation_execution actual extraction complete

> Batch: BE-001OM-02
> Node: `backend.ops_governance.chaos.experiment_creation.perturbation_execution`
> Parent: `backend.ops_governance.chaos.experiment_creation`
> Stage: `extract_closeout`
> Movement: Chaos perturbation side effects moved into a private child module.

---

## Summary

`backend.ops_governance.chaos.experiment_creation.perturbation_execution` now owns max-duration resolution and all chaos perturbation side effects.

The experiment_creation parent keeps a local `execute_perturbation` bridge and continues to own the larger create-flow orchestration.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/chaos/handlers/experiment_creation.rs` | `src/backend/ops_governance/chaos/handlers/experiment_creation/perturbation_execution.rs` | Max-duration resolution and perturbation execution moved. |
| `src/backend/ops_governance/chaos/handlers/experiment_creation.rs` | `src/backend/ops_governance/chaos/handlers/experiment_creation.rs` | Parent declares the private child and keeps the local execution bridge. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| Default duration | Default clamp remains 10 seconds. |
| Environment override | `QUANTPILOT_CHAOS_MAX_DURATION_MS` is used only when present and parseable as `u64`. |
| Clamp | Each perturbation uses `request.injection.duration_ms.min(max_duration_ms)`. |
| Disk pressure path | Disk pressure still writes under `<chaos_store_dir>/temp_pressure`. |
| Disk pressure write | Disk pressure still writes ten 1 MiB files named `pressure_<n>.bin`. |
| Disk cleanup | Disk pressure still removes the temp directory after the sleep. |
| Latency/event loss/clock skew | These variants still perform only the clamped sleep. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- experiment_creation parent create flow -> experiment_creation parent perturbation bridge;
- experiment_creation parent perturbation bridge -> private `perturbation_execution::*`;
- perturbation_execution child -> runtime fs/sleep primitives.

The following remain outside this child:

- route bridge;
- chaos mode lifecycle;
- evidence metric sampling;
- metric projection, pass criteria, alert/action assembly, and report assembly;
- persistence and memory commit;
- closed report_persistence, read routes, route facade, closed ops siblings, AppState owner, frontend caller, and release transition logic.

## Proof

- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`

## Next Step

BE-001OM-03 backend.ops_governance.chaos.experiment_creation.perturbation_execution single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
