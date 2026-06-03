# v4.16.0 backend.ops_governance.chaos.read_routes single leaf closeout

> Batch: BE-001OT-03
> Node: `backend.ops_governance.chaos.read_routes`
> Parent: `backend.ops_governance.chaos`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.read_routes` is closed after BE-001OT-02.

Decision:

`stop_split: true`

## Split Decision

| Rule | Result |
| --- | --- |
| Independent failure boundary | The child owns one coherent list/detail read boundary. |
| Route or public boundary density | Route registration remains outside and parent-owned. |
| Local proof exists | Local tests verify scoped list filtering, newest-first sorting, and scoped detail lookup. |
| Parent-child communication cost | Further splitting list and detail would add route bridges without isolating a stronger owner. |
| Persistence surface | Disk fallback remains parent-mediated through the chaos persistence bridge. |
| Line-count-only split | Rejected: deeper split would be based on two handlers, not separate failure ownership. |

## Closed Boundary

`backend.ops_governance.chaos.read_routes` owns:

- list read handler;
- scoped prefix filtering;
- newest-first sorting by `executed_at`;
- detail read handler;
- scoped in-memory detail lookup;
- parent-mediated disk fallback.

Allowed call paths remain:

- route facade -> chaos parent read bridges;
- chaos parent read bridges -> private `read_routes::*`;
- read_routes detail -> chaos parent disk-load bridge;
- chaos parent disk-load bridge -> private `report_persistence::*`.

## Remaining Parent Residuals

Return to `backend.ops_governance.chaos` parent residual judgment.

Current chaos queue:

- `backend.ops_governance.chaos.route_facade`.

## Hard Boundaries

Next chaos residual batches must not move:

- closed `experiment_creation` internals;
- closed `read_routes` internals;
- closed `report_persistence` internals;
- closed hotswap, sandbox, alerts, snapshots, and runbook internals;
- AppState owner or lock order;
- chaos schema type definitions;
- frontend caller;
- release transition logic.

No sibling shortcut is allowed.

## Next Step

BE-001OU-01 backend.ops_governance.chaos parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
