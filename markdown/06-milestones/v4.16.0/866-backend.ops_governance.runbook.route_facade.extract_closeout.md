# v4.16.0 backend.ops_governance.runbook.route_facade actual extraction complete

> Batch: BE-001OD-02
> Node: `backend.ops_governance.runbook.route_facade`
> Parent: `backend.ops_governance.runbook`
> Stage: `extract_closeout`
> Movement: Runbook route registration moved into a private child module.

---

## Summary

`backend.ops_governance.runbook.route_facade` now owns runbook route path registration.

The runbook handler parent still owns the public `register_runbook_routes` bridge and parent-owned read handler bridges, so the route facade child does not call the closed `read_routes` child directly.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/runbook/handlers.rs` | `src/backend/ops_governance/runbook/handlers/route_facade.rs` | Route registration for `/api/v1/runbook` and `/api/v1/runbook/:scenario_id` moved. |
| `src/backend/ops_governance/runbook/handlers.rs` | `src/backend/ops_governance/runbook/handlers.rs` | Parent declares the private child module, keeps the public route registration bridge, and adds read handler bridge functions. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| Route paths | `/api/v1/runbook` and `/api/v1/runbook/:scenario_id` remain unchanged. |
| List route binding | List route still resolves to runbook list handler behavior through the parent bridge. |
| Detail route binding | Detail route still resolves to runbook detail handler behavior through the parent bridge. |
| Catalog bridge | Read handlers still consume the catalog through the parent bridge. |
| External API | `backend::ops_governance::runbook::register_routes` remains the parent-facing entrypoint. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- runbook parent bridge -> private `handlers::route_facade::register_runbook_routes`;
- route facade child -> parent-owned read handler bridge functions;
- parent-owned read handler bridge functions -> closed private `handlers::read_routes::*`;
- read handlers -> parent `build_default_runbook` bridge;
- parent `build_default_runbook` bridge -> closed private `handlers::scenario_catalog::build_default_runbook`.

The following remain outside this child:

- closed scenario catalog internals;
- closed read route internals;
- root compatibility bridge;
- chaos route and handler owner;
- closed hotswap, sandbox, alerts, and snapshots internals;
- AppState owner or lock order;
- schema type definitions, frontend caller, and release transition logic.

## Proof

- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`

## Next Step

BE-001OD-03 backend.ops_governance.runbook.route_facade single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
