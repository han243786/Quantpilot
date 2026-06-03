# v4.16.0 backend.ops_governance.chaos.report_persistence equivalence baseline and extraction plan

> Batch: BE-001OI-01
> Node: `backend.ops_governance.chaos.report_persistence`
> Parent: `backend.ops_governance.chaos`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.report_persistence` is frozen as the chaos report persistence and disk loading owner.

BE-001OI-01 does not move code. It defines the exact baseline and allowed movement for BE-001OI-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/chaos/handlers.rs`

Current selected boundary:

- `persist_chaos_report`;
- `load_chaos_report_from_disk`;
- `validate_experiment_id`.

The parent bridge must remain:

- `persist_chaos_report`;
- `load_chaos_report_from_disk`.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Quota check | Persistence still calls `ensure_storage_quota` for transient chaos storage. |
| Directory creation | Persistence still creates the chaos store directory before writing. |
| Atomic write | Persistence still writes reports with `runtime_persistence::atomic_write_json`. |
| ID validation | Empty, overlong, path-like, NUL-containing, and non-ASCII IDs are rejected. |
| Disk read | Disk loading still reads `<experiment_id>.json` from the chaos store. |
| Missing disk report | Missing reports still return `json_bad_request("not_found", ...)`. |
| Decode failure | JSON decode failures still map to `internal_error`. |

## Allowed BE-001OI-02 Movement

BE-001OI-02 may:

- create `src/backend/ops_governance/chaos/handlers/report_persistence.rs`;
- move only `persist_chaos_report`, `load_chaos_report_from_disk`, and `validate_experiment_id` into that private child module;
- add a private `mod report_persistence;` declaration in `src/backend/ops_governance/chaos/handlers.rs`;
- keep parent bridge functions with the same names and signatures, delegating to the child;
- add local ID validation tests if useful.

## Forbidden BE-001OI-02 Movement

BE-001OI-02 must not move or rewrite:

- create experiment handler;
- list/detail route handlers;
- route facade;
- chaos perturbation execution;
- metric projection or report assembly;
- closed hotswap, sandbox, alerts, snapshots, and runbook internals;
- AppState owner or lock order;
- chaos schema type definitions;
- frontend caller;
- release transition logic.

## Parent-Child Rule

Allowed call paths:

- chaos create/detail handlers -> chaos parent persistence bridge;
- chaos parent persistence bridge -> private `handlers::report_persistence::*`;
- report_persistence child -> storage lifecycle and runtime persistence helpers.

Forbidden call path:

Any create/detail handler importing or calling `handlers::report_persistence` directly.

## Proof

BE-001OI-02 must prove equivalence with:

- `cargo test -p quantpilot chaos`
- `cargo check -p quantpilot`

## Next Step

BE-001OI-02 backend.ops_governance.chaos.report_persistence extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
