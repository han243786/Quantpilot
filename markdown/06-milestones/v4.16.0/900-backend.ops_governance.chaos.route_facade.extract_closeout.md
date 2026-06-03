# v4.16.0 backend.ops_governance.chaos.route_facade actual extraction complete

> Batch: BE-001OV-02
> Node: `backend.ops_governance.chaos.route_facade`
> Parent: `backend.ops_governance.chaos`
> Stage: `extract_closeout`
> Movement: Chaos route registration moved into a private child module.

---

## Summary

`backend.ops_governance.chaos.route_facade` now owns create/list/detail chaos endpoint registration.

The chaos handler parent keeps a local `register_chaos_routes` bridge and still owns create/list/detail handler bridges.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/chaos/handlers.rs` | `src/backend/ops_governance/chaos/handlers/route_facade.rs` | Route registration moved. |
| `src/backend/ops_governance/chaos/handlers.rs` | `src/backend/ops_governance/chaos/handlers.rs` | Parent declares the private child and keeps the route registration bridge. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| Create route | `POST /api/v1/chaos/experiments` still binds to the parent create bridge. |
| List route | `GET /api/v1/chaos/experiments` still binds to the parent list bridge. |
| Detail route | `GET /api/v1/chaos/experiments/:experiment_id` still binds to the parent detail bridge. |
| Route order | Route registration keeps the same endpoint sequence. |
| Handler ownership | Route facade binds only to parent handler bridges. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- chaos parent register bridge -> private `route_facade::*`;
- route_facade child -> chaos parent handler bridges.

The following remain outside this child:

- closed experiment_creation internals;
- closed read_routes internals;
- closed report_persistence internals;
- closed ops siblings, AppState owner, schema type definitions, frontend caller, and release transition logic.

## Proof

- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`

## Next Step

BE-001OV-03 backend.ops_governance.chaos.route_facade single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
