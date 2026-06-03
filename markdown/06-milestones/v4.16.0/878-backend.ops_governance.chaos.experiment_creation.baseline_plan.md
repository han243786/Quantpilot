# v4.16.0 backend.ops_governance.chaos.experiment_creation equivalence baseline and extraction plan

> Batch: BE-001OK-01
> Node: `backend.ops_governance.chaos.experiment_creation`
> Parent: `backend.ops_governance.chaos`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.experiment_creation` is frozen as the chaos experiment create-flow owner.

BE-001OK-01 does not move code. It defines the exact baseline and allowed movement for BE-001OK-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/chaos/handlers.rs`

Current selected boundary:

- `create_experiment` route handler implementation;
- experiment ID generation;
- `chaos_mode` enable/disable lifecycle;
- evidence metric sampling;
- max-duration environment clamp;
- perturbation execution for all four chaos experiment types;
- steady-state metric projection;
- pass/fail criteria;
- alert and degradation action assembly;
- report assembly;
- persistence through the chaos parent bridge;
- in-memory experiment insertion.

The parent bridge must remain:

- route-facing `create_experiment` binding used by route registration;
- `persist_chaos_report`.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Experiment ID | IDs still use `chaos-<current_time_ms>`. |
| Chaos mode | `chaos_mode` is set true before injection and false after injection. |
| Evidence sampling | The same evidence metric counters are sampled before and after injection. |
| Duration clamp | `QUANTPILOT_CHAOS_MAX_DURATION_MS` still overrides the default 10 second clamp when parseable. |
| Disk pressure | The same temp pressure directory and 10 MiB write loop are used, then removed. |
| Latency/event loss/clock skew | These variants still sleep for the clamped injection duration. |
| Metrics | Before/during/after steady-state metrics keep the same projections. |
| Pass criteria | Each experiment type keeps the same pass/fail expression. |
| Alerts/actions | Alert and degradation action vectors keep the same values. |
| Persistence | Create flow still persists through the chaos parent persistence bridge. |
| Memory insert | Reports are still inserted under `auth::scoped_key(user_id, experiment_id)`. |

## Allowed BE-001OK-02 Movement

BE-001OK-02 may:

- create `src/backend/ops_governance/chaos/handlers/experiment_creation.rs`;
- move only the create-flow implementation body into that private child module;
- add a private `mod experiment_creation;` declaration in `src/backend/ops_governance/chaos/handlers.rs`;
- keep a parent route-handler bridge named `create_experiment` with the same extractor signature;
- keep parent-mediated persistence by calling `super::persist_chaos_report` from the child.

## Forbidden BE-001OK-02 Movement

BE-001OK-02 must not move or rewrite:

- closed `report_persistence` internals;
- list/detail read handlers;
- route facade;
- chaos schema type definitions;
- closed hotswap, sandbox, alerts, snapshots, and runbook internals;
- AppState owner or lock order;
- frontend caller;
- release transition logic.

## Parent-Child Rule

Allowed call paths:

- chaos route facade -> chaos parent create bridge;
- chaos parent create bridge -> private `handlers::experiment_creation::*`;
- experiment_creation child -> chaos parent persistence bridge;
- chaos parent persistence bridge -> private `handlers::report_persistence::*`.

Forbidden call path:

Any experiment_creation child importing or calling `handlers::report_persistence` directly.

## Proof

BE-001OK-02 must prove equivalence with:

- `cargo test -p quantpilot chaos`
- `cargo check -p quantpilot`

## Next Step

BE-001OK-02 backend.ops_governance.chaos.experiment_creation extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
