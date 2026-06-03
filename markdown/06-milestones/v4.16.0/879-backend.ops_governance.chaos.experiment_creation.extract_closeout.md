# v4.16.0 backend.ops_governance.chaos.experiment_creation actual extraction complete

> Batch: BE-001OK-02
> Node: `backend.ops_governance.chaos.experiment_creation`
> Parent: `backend.ops_governance.chaos`
> Stage: `extract_closeout`
> Movement: Chaos create-flow implementation moved into a private child module.

---

## Summary

`backend.ops_governance.chaos.experiment_creation` now owns the chaos experiment create-flow implementation.

The chaos handler parent still owns the route-facing `create_experiment` bridge used by route registration.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/chaos/handlers.rs` | `src/backend/ops_governance/chaos/handlers/experiment_creation.rs` | Create-flow implementation moved. |
| `src/backend/ops_governance/chaos/handlers.rs` | `src/backend/ops_governance/chaos/handlers.rs` | Parent declares the private child and keeps the route-facing create bridge. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| Experiment ID | IDs still use `chaos-<current_time_ms>`. |
| Chaos mode | `chaos_mode` is still set true before injection and false after injection. |
| Evidence sampling | The same evidence metric counters are sampled before and after injection. |
| Duration clamp | `QUANTPILOT_CHAOS_MAX_DURATION_MS` still overrides the default 10 second clamp when parseable. |
| Disk pressure | Disk pressure still creates temp files, sleeps for the clamped duration, and removes the temp directory. |
| Other variants | Latency, event loss, and clock skew still sleep for the clamped duration. |
| Metrics and criteria | Metric projections, pass criteria, alerts, and degradation actions are unchanged. |
| Persistence | The create child calls the chaos parent persistence bridge; it does not call `report_persistence` directly. |
| Memory insert | Reports are still inserted under `auth::scoped_key(user_id, experiment_id)`. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- chaos route facade -> chaos parent create bridge;
- chaos parent create bridge -> private `handlers::experiment_creation::*`;
- experiment_creation child -> chaos parent persistence bridge;
- chaos parent persistence bridge -> private `handlers::report_persistence::*`.

The following remain outside this child:

- closed `report_persistence` internals;
- list/detail read handlers;
- route facade;
- closed ops siblings, AppState owner, schema type definitions, frontend caller, and release transition logic.

## Proof

- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`

## Next Step

BE-001OK-03 backend.ops_governance.chaos.experiment_creation single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
