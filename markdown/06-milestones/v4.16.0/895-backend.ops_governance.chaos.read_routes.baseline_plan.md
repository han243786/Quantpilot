# v4.16.0 backend.ops_governance.chaos.read_routes equivalence baseline and extraction plan

> Batch: BE-001OT-01
> Node: `backend.ops_governance.chaos.read_routes`
> Parent: `backend.ops_governance.chaos`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.read_routes` is frozen as the chaos list/detail read handler owner.

BE-001OT-01 does not move code. It defines the exact baseline and allowed movement for BE-001OT-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/chaos/handlers.rs`

Current selected boundary:

- `list_experiments`;
- scoped prefix filtering;
- newest-first `executed_at` sort;
- `get_experiment`;
- scoped in-memory detail lookup;
- disk fallback through the chaos parent bridge.

The parent bridge must remain:

- route facade calls parent-owned read handler bridges;
- detail read calls parent-owned `load_chaos_report_from_disk` bridge for disk fallback.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| List scope | List still uses `auth::scoped_key(user_id, "")` as prefix. |
| List projection | List still clones matching reports from `chaos_experiments`. |
| List order | List still sorts newest-first by `executed_at`. |
| Detail scope | Detail lookup still uses `auth::scoped_key(user_id, experiment_id)`. |
| Memory hit | Detail still returns the in-memory cloned report when present. |
| Disk fallback | Detail still falls back to `load_chaos_report_from_disk` when memory misses. |

## Allowed BE-001OT-02 Movement

BE-001OT-02 may:

- create `src/backend/ops_governance/chaos/handlers/read_routes.rs`;
- add a private `mod read_routes;` declaration in `src/backend/ops_governance/chaos/handlers.rs`;
- move only list/detail read handlers and local read projection helpers into that private child;
- keep parent-owned `list_experiments` and `get_experiment` bridges for route registration;
- keep disk fallback parent-mediated through `super::load_chaos_report_from_disk`;
- add local read projection tests if useful.

## Forbidden BE-001OT-02 Movement

BE-001OT-02 must not move or rewrite:

- closed `experiment_creation` internals;
- closed `report_persistence` internals;
- route facade;
- closed hotswap, sandbox, alerts, snapshots, and runbook internals;
- AppState owner or lock order beyond existing read locks;
- chaos schema type definitions;
- frontend caller;
- release transition logic.

## Parent-Child Rule

Allowed call paths:

- route facade -> chaos parent read bridges;
- chaos parent read bridges -> private `read_routes::*`;
- read_routes detail -> chaos parent disk-load bridge;
- chaos parent disk-load bridge -> private `report_persistence::*`.

Forbidden call path:

Any read_routes child importing or calling `report_persistence` directly.

## Proof

BE-001OT-02 must prove equivalence with:

- `cargo test -p quantpilot chaos`
- `cargo check -p quantpilot`

## Next Step

BE-001OT-02 backend.ops_governance.chaos.read_routes extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
