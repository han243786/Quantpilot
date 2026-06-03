# v4.16.0 backend.ops_governance.snapshots.read_routes equivalence baseline and extraction plan

> Batch: BE-001NN-01
> Node: `backend.ops_governance.snapshots.read_routes`
> Parent: `backend.ops_governance.snapshots`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots.read_routes` is frozen as the snapshot read projection child.

BE-001NN-01 does not move code. It defines the exact baseline and allowed movement for BE-001NN-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/snapshots/handlers.rs`

Current selected handlers:

- `list_snapshots`;
- `get_snapshot`.

Current parent-owned helper used by the selected handlers:

- `load_snapshot_from_disk`.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| List route | Still clones in-memory snapshots, sorts descending by `created_at_ms`, and paginates. |
| Get route | Still checks `state.snapshots` memory first. |
| Disk fallback | Missing memory entry still falls back through the parent-owned `load_snapshot_from_disk`. |
| Error mapping | Disk fallback error mapping remains parent-owned and unchanged. |
| Route surface | GET `/api/v1/snapshots` and GET `/api/v1/snapshots/:snapshot_id` remain unchanged. |

## Allowed BE-001NN-02 Movement

BE-001NN-02 may:

- create `src/backend/ops_governance/snapshots/handlers/read_routes.rs`;
- move `list_snapshots` into that private child module;
- move `get_snapshot` into that private child module;
- add a private `mod read_routes;` declaration in `src/backend/ops_governance/snapshots/handlers.rs`;
- update route registration to call `read_routes::list_snapshots` and `read_routes::get_snapshot`.

## Forbidden BE-001NN-02 Movement

BE-001NN-02 must not move or rewrite:

- create flow child;
- restore handler;
- snapshot ID validation child;
- disk load file path construction, file read, JSON parse, or error mapping;
- snapshot persistence or restore audit persistence implementation;
- shared signature helper;
- route facade beyond swapping handler function references;
- AppState cleanup behavior;
- storage lifecycle internals;
- runbook, chaos, hotswap, sandbox, or alerts code;
- release transition logic.

## Parent-Child Rule

The child must stay private under the current snapshots handler implementation owner.

Allowed call paths:

- `handlers.rs` route registration -> private `handlers::read_routes` handlers;
- `read_routes` child -> snapshots handler parent `load_snapshot_from_disk`.

Forbidden call path:

Any sibling ops module or release-transition shortcut directly calling read routes or disk-load internals.

## Proof

BE-001NN-02 must prove equivalence with:

- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `cargo check -p quantpilot`

## Next Step

BE-001NN-02 backend.ops_governance.snapshots.read_routes extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
