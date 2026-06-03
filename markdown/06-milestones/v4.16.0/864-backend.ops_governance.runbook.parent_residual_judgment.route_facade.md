# v4.16.0 backend.ops_governance.runbook parent residual judgment selects route_facade

> Batch: BE-001OC-01
> Node: `backend.ops_governance.runbook`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.runbook` returns to parent residual judgment after `read_routes` closed as the runbook list/detail read behavior owner.

The next child is fixed as:

`backend.ops_governance.runbook.route_facade`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.runbook.route_facade` | Route registration for `/api/v1/runbook` and `/api/v1/runbook/:scenario_id`. | Select for next baseline. |

## Selected Child Boundary

`backend.ops_governance.runbook.route_facade` currently contains:

- `register_runbook_routes`;
- route path ownership for list/detail runbook endpoints;
- binding of route paths to read handler bridge functions.

The selected child must not directly call the closed `read_routes` child. If moved, route registration must bind to parent-owned bridge handlers.

## Hard Boundaries

BE-001OD-01/02 must not move:

- closed scenario catalog internals;
- closed read route internals;
- root compatibility bridge;
- chaos route or handler owner;
- closed hotswap, sandbox, alerts, or snapshots internals;
- AppState owner or lock order;
- schema type definitions, frontend caller, or release transition logic.

No sibling shortcut is allowed.

## Next Step

BE-001OD-01 backend.ops_governance.runbook.route_facade baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
