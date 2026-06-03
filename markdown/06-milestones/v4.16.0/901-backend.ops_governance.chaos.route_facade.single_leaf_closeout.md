# v4.16.0 backend.ops_governance.chaos.route_facade single leaf closeout

> Batch: BE-001OV-03
> Node: `backend.ops_governance.chaos.route_facade`
> Parent: `backend.ops_governance.chaos`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.route_facade` is closed after BE-001OV-02.

Decision:

`stop_split: true`

## Split Decision

| Rule | Result |
| --- | --- |
| Independent failure boundary | The child owns one coherent chaos route registration surface. |
| Route or public boundary density | This is the only public route-registration boundary for chaos. |
| Local proof exists | Route wiring is covered by `cargo check` and route handler type compatibility. |
| Parent-child communication cost | Further splitting three endpoints would fragment a single registration contract. |
| Persistence surface | No persistence surface lives inside this child. |
| Line-count-only split | Rejected: the route facade is already minimal. |

## Closed Boundary

`backend.ops_governance.chaos.route_facade` owns:

- `POST /api/v1/chaos/experiments`;
- `GET /api/v1/chaos/experiments`;
- `GET /api/v1/chaos/experiments/:experiment_id`;
- binding to chaos parent create/list/detail handler bridges.

Allowed call paths remain:

- chaos parent register bridge -> private `route_facade::*`;
- route_facade child -> chaos parent handler bridges.

## Remaining Parent Residuals

All internal chaos children are now closed:

- `backend.ops_governance.chaos.report_persistence`;
- `backend.ops_governance.chaos.experiment_creation`;
- `backend.ops_governance.chaos.read_routes`;
- `backend.ops_governance.chaos.route_facade`.

Return to `backend.ops_governance.chaos` parent closeout.

## Hard Boundaries

Next parent closeout must not move:

- closed `experiment_creation`, `read_routes`, `route_facade`, or `report_persistence` internals;
- closed hotswap, sandbox, alerts, snapshots, and runbook internals;
- AppState owner or lock order;
- chaos schema type definitions;
- frontend caller;
- release transition logic.

No sibling shortcut is allowed.

## Next Step

BE-001OW-01 backend.ops_governance.chaos parent_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
