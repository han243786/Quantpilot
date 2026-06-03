# v4.16.0 backend.ops_governance.alerts.persistence equivalence baseline and extraction plan

> Batch: BE-001NA-02
> Node: `backend.ops_governance.alerts.persistence`
> Parent: `backend.ops_governance.alerts`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts.persistence` is frozen as the alert firing disk persistence child.

BE-001NA-02 does not move code. It defines the exact baseline and allowed movement for BE-001NA-03.

## Current Owner

Current implementation is still in `src/backend/ops_governance/alerts/handlers.rs`.

The child boundary is:

- `persist_alert_firing`

The parent bridge must remain:

- `persist_alert_firing`

Both `acknowledge_flow` and `trigger_engine` must continue to call the parent bridge, not the child directly.

## Frozen Semantics

The next extraction must preserve:

| Surface | Frozen behavior |
| --- | --- |
| Return type | Still returns `std::io::Result<()>`. |
| Storage root | Uses `std::path::Path::new("storage")`. |
| Storage namespace | Uses `"alerts"`. |
| Lifecycle class | Uses `StorageLifecycle::Transient`. |
| Quota order | Calls `ensure_storage_quota` before creating the firing directory. |
| Directory creation | Calls async `fs::create_dir_all(store_dir).await?`. |
| File path | Writes to `store_dir.join(format!("{}.json", firing.firing_id))`. |
| Write primitive | Uses `runtime_persistence::atomic_write_json(&file_path, firing).await`. |
| Caller contract | Acknowledge flow and trigger engine still pass the same `store_dir` and `AlertFiring` values through the parent bridge. |

## Allowed BE-001NA-03 Movement

BE-001NA-03 may:

- create a private child module for alert persistence under the alerts handler owner boundary;
- move only the implementation body of `persist_alert_firing` into that child;
- keep a parent bridge named `persist_alert_firing` that delegates to the child;
- keep all existing call sites parent-mediated.

## Forbidden BE-001NA-03 Movement

BE-001NA-03 must not move or rewrite:

- acknowledge route logic;
- trigger route logic;
- predicate dispatch;
- rule catalog;
- startup initialization;
- list/read route handlers;
- route registration;
- DTO schema owner;
- AppState fields or lock ordering;
- storage lifecycle implementation internals;
- runtime persistence implementation internals;
- release transition logic.

## Proof

No direct persistence unit test is currently isolated for alerts. BE-001NA-03 must therefore keep the movement mechanical and prove equivalence with compile, existing alerts tests, and governance gates.

## Next Step

BE-001NA-03 backend.ops_governance.alerts.persistence extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
