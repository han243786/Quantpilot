# v4.16.0 backend.ops_governance.chaos.read_routes actual extraction complete

> Batch: BE-001OT-02
> Node: `backend.ops_governance.chaos.read_routes`
> Parent: `backend.ops_governance.chaos`
> Stage: `extract_closeout`
> Movement: Chaos list/detail read handlers moved into a private child module.

---

## Summary

`backend.ops_governance.chaos.read_routes` now owns list/detail read handling, scoped filtering, newest-first sorting, in-memory lookup, and parent-mediated disk fallback.

The chaos handler parent keeps route-facing `list_experiments` and `get_experiment` bridges.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/chaos/handlers.rs` | `src/backend/ops_governance/chaos/handlers/read_routes.rs` | List/detail read handlers and local read projection helpers moved. |
| `src/backend/ops_governance/chaos/handlers.rs` | `src/backend/ops_governance/chaos/handlers.rs` | Parent declares the private child and keeps route-facing read bridges. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| List scope | List still uses `auth::scoped_key(user_id, "")` as prefix. |
| List projection | List still clones matching reports from `chaos_experiments`. |
| List order | List still sorts newest-first by `executed_at`. |
| Detail scope | Detail lookup still uses `auth::scoped_key(user_id, experiment_id)`. |
| Memory hit | Detail still returns the in-memory cloned report when present. |
| Disk fallback | Detail still falls back to the chaos parent disk-load bridge when memory misses. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- route facade -> chaos parent read bridges;
- chaos parent read bridges -> private `read_routes::*`;
- read_routes detail -> chaos parent disk-load bridge;
- chaos parent disk-load bridge -> private `report_persistence::*`.

The following remain outside this child:

- closed experiment_creation internals;
- closed report_persistence internals;
- route facade;
- closed ops siblings, AppState owner, schema type definitions, frontend caller, and release transition logic.

## Proof

- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`

## Next Step

BE-001OT-03 backend.ops_governance.chaos.read_routes single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
