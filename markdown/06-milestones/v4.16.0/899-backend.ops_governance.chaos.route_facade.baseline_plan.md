# v4.16.0 backend.ops_governance.chaos.route_facade equivalence baseline and extraction plan

> Batch: BE-001OV-01
> Node: `backend.ops_governance.chaos.route_facade`
> Parent: `backend.ops_governance.chaos`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.route_facade` is frozen as the chaos endpoint registration owner.

BE-001OV-01 does not move code. It defines the exact baseline and allowed movement for BE-001OV-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/chaos/handlers.rs`

Current selected boundary:

- `register_chaos_routes`;
- `POST /api/v1/chaos/experiments`;
- `GET /api/v1/chaos/experiments`;
- `GET /api/v1/chaos/experiments/:experiment_id`;
- binding to parent-owned `create_experiment`, `list_experiments`, and `get_experiment` bridges.

The parent bridge must remain:

- `register_chaos_routes` on the chaos handler parent delegates to the private route facade;
- handler functions remain parent-owned bridges.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Create route | `POST /api/v1/chaos/experiments` still binds to `create_experiment`. |
| List route | `GET /api/v1/chaos/experiments` still binds to `list_experiments`. |
| Detail route | `GET /api/v1/chaos/experiments/:experiment_id` still binds to `get_experiment`. |
| Route order | Route registration keeps the same endpoint sequence. |
| Handler ownership | Route facade binds to parent handler bridges only. |

## Allowed BE-001OV-02 Movement

BE-001OV-02 may:

- create `src/backend/ops_governance/chaos/handlers/route_facade.rs`;
- add a private `mod route_facade;` declaration in `src/backend/ops_governance/chaos/handlers.rs`;
- move only route registration into that private child;
- keep a parent-owned `register_chaos_routes` bridge;
- bind route handlers only through parent-owned `create_experiment`, `list_experiments`, and `get_experiment`.

## Forbidden BE-001OV-02 Movement

BE-001OV-02 must not move or rewrite:

- closed `experiment_creation` internals;
- closed `read_routes` internals;
- closed `report_persistence` internals;
- closed hotswap, sandbox, alerts, snapshots, and runbook internals;
- AppState owner or lock order;
- chaos schema type definitions;
- frontend caller;
- release transition logic.

## Parent-Child Rule

Allowed call paths:

- chaos parent register bridge -> private `route_facade::*`;
- route_facade child -> chaos parent handler bridges.

Forbidden call path:

Any route_facade child importing or calling `experiment_creation`, `read_routes`, or `report_persistence` directly.

## Proof

BE-001OV-02 must prove equivalence with:

- `cargo test -p quantpilot chaos`
- `cargo check -p quantpilot`

## Next Step

BE-001OV-02 backend.ops_governance.chaos.route_facade extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
