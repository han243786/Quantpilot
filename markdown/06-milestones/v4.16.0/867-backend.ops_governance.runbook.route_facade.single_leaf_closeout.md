# v4.16.0 backend.ops_governance.runbook.route_facade single leaf closeout

> Batch: BE-001OD-03
> Node: `backend.ops_governance.runbook.route_facade`
> Parent: `backend.ops_governance.runbook`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.runbook.route_facade` is closed after BE-001OD-02.

Decision:

`stop_split: true`

## Split Decision

| Rule | Result |
| --- | --- |
| Independent failure boundary | The child owns one coherent route registration surface. |
| Route or public boundary density | Both runbook route paths belong to the same public route facade. |
| Local proof exists | Compile and runbook tests verify the handler bindings remain valid. |
| Parent-child communication cost | Parent-owned read handler bridges prevent direct sibling calls. |
| Persistence surface | No persistence or lock ownership exists in this child. |
| Line-count-only split | Rejected: deeper split would separate two route lines without a stronger boundary. |

## Closed Boundary

`backend.ops_governance.runbook.route_facade` owns:

- `/api/v1/runbook` route registration;
- `/api/v1/runbook/:scenario_id` route registration;
- binding route paths to parent-owned handler bridges.

Allowed call paths remain:

- runbook parent bridge -> private `handlers::route_facade::register_runbook_routes`;
- route facade child -> parent-owned read handler bridge functions;
- parent-owned read handler bridge functions -> closed private `handlers::read_routes::*`;
- read handlers -> parent `build_default_runbook` bridge;
- parent `build_default_runbook` bridge -> closed private `handlers::scenario_catalog::build_default_runbook`.

## Remaining Parent Residuals

Return to `backend.ops_governance.runbook` parent closeout.

Current runbook queue:

- none.

## Hard Boundaries

Next runbook parent closeout must not move:

- closed scenario catalog internals;
- closed read route internals;
- closed route facade internals;
- root compatibility bridge;
- chaos route or handler owner;
- closed hotswap, sandbox, alerts, or snapshots internals;
- AppState owner or lock order;
- schema type definitions, frontend caller, or release transition logic.

No sibling shortcut is allowed.

## Next Step

BE-001OE-01 backend.ops_governance.runbook parent_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
