# v4.16.0 backend.ops_governance.runbook.read_routes actual extraction complete

> Batch: BE-001OB-02
> Node: `backend.ops_governance.runbook.read_routes`
> Parent: `backend.ops_governance.runbook`
> Stage: `extract_closeout`
> Movement: Runbook list/detail read handlers moved into a private child module.

---

## Summary

`backend.ops_governance.runbook.read_routes` now owns runbook list/detail handler behavior.

The runbook handler parent still owns the route facade and the `build_default_runbook` bridge, so read handlers remain parent-mediated and do not import the closed scenario catalog child directly.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/runbook/handlers.rs` | `src/backend/ops_governance/runbook/handlers/read_routes.rs` | `list_scenarios` and `get_scenario` moved. |
| `src/backend/ops_governance/runbook/handlers.rs` | `src/backend/ops_governance/runbook/handlers.rs` | Parent declares the private child module, keeps route facade registration, and keeps the catalog bridge. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| List route handler | Returns the full default runbook catalog. |
| Detail route handler | Returns the matching scenario when `scenario_id` exists. |
| Missing detail route | Returns `json_bad_request("not_found", ...)` for an unknown scenario ID. |
| Catalog access | Read handlers consume the default catalog through the parent bridge. |
| Route facade | `register_runbook_routes` still registers `/api/v1/runbook` and `/api/v1/runbook/:scenario_id`. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- runbook route facade -> private `handlers::read_routes::*`;
- runbook read handlers -> parent `build_default_runbook` bridge;
- parent `build_default_runbook` bridge -> closed private `handlers::scenario_catalog::build_default_runbook`.

The following remain outside this child:

- closed scenario catalog internals;
- route facade registration ownership;
- root compatibility bridge;
- chaos route and handler owner;
- closed hotswap, sandbox, alerts, and snapshots internals;
- AppState owner or lock order;
- schema type definitions, frontend caller, and release transition logic.

## Proof

- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`

## Next Step

BE-001OB-03 backend.ops_governance.runbook.read_routes single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
