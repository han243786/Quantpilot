# v4.16.0 backend.ops_governance.chaos.report_persistence actual extraction complete

> Batch: BE-001OI-02
> Node: `backend.ops_governance.chaos.report_persistence`
> Parent: `backend.ops_governance.chaos`
> Stage: `extract_closeout`
> Movement: Chaos report persistence and ID validation moved into a private child module.

---

## Summary

`backend.ops_governance.chaos.report_persistence` now owns chaos report disk persistence, disk loading, and experiment ID validation.

The chaos handler parent still owns the same-named persistence bridge functions used by create/detail handlers.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/chaos/handlers.rs` | `src/backend/ops_governance/chaos/handlers/report_persistence.rs` | `persist_chaos_report`, `load_chaos_report_from_disk`, and `validate_experiment_id` moved. |
| `src/backend/ops_governance/chaos/handlers.rs` | `src/backend/ops_governance/chaos/handlers.rs` | Parent declares the private child and keeps same-name bridge functions. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| Quota check | Persistence still calls `ensure_storage_quota` for transient chaos storage. |
| Directory creation | Persistence still creates the chaos store directory before writing. |
| Atomic write | Persistence still writes reports with `runtime_persistence::atomic_write_json`. |
| ID validation | Empty, overlong, path-like, NUL-containing, and non-ASCII IDs are rejected. |
| Disk read | Disk loading still reads `<experiment_id>.json` from the chaos store. |
| Missing disk report | Missing reports still return `json_bad_request("not_found", ...)`. |
| Decode failure | JSON decode failures still map to `internal_error`. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- chaos create/detail handlers -> chaos parent persistence bridge;
- chaos parent persistence bridge -> private `handlers::report_persistence::*`;
- report_persistence child -> storage lifecycle and runtime persistence helpers.

The following remain outside this child:

- create experiment handler;
- list/detail route handlers;
- route facade;
- chaos perturbation execution;
- metric projection and report assembly;
- closed ops siblings, AppState owner, schema type definitions, frontend caller, and release transition logic.

## Proof

- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`

## Next Step

BE-001OI-03 backend.ops_governance.chaos.report_persistence single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
