# v4.16.0 backend.ops_governance.snapshots parent residual judgment closes route_facade

> Batch: BE-001NU-01
> Node: `backend.ops_governance.snapshots.route_facade`
> Parent: `backend.ops_governance.snapshots`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots.route_facade` is closed as a parent-owned static facade residual.

Decision:

`static_closeout: true`

No child extraction is selected for route facade.

## Reason

The remaining route facade is the snapshots parent boundary itself:

- declares snapshot route paths;
- wires list and get read handlers;
- wires restore handler;
- wires create handler;
- preserves route order and HTTP methods.

Extracting it into a child would force that child to reference create, read, and restore siblings directly, or require generic handler plumbing that only wraps parent mediation. Both options would reduce clarity under the hard parent-child communication rule.

## Residual Judgment

| Candidate | Decision | Evidence |
| --- | --- | --- |
| `backend.ops_governance.snapshots.route_facade` | Close in parent. | It is parent facade wiring, not a separate behavior owner. |
| `backend.ops_governance.snapshots` | Select parent closeout next. | Snapshot ID validation, create flow, read routes, restore flow, persistence, signature contract, and route facade are all closed. |

## Closed Boundary

Closed residual:

`backend.ops_governance.snapshots.route_facade`

The route facade remains in `src/backend/ops_governance/snapshots/handlers.rs`.

## Hard Boundaries

Future snapshots batches must not extract route facade unless a new explicit proposal proves a route owner that avoids sibling direct calls and improves local proof.

Route paths, methods, handler mapping, and route order remain parent-owned.

## Next Step

BE-001NV-01 backend.ops_governance.snapshots parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
