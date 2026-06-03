# v4.16.0 backend.ops_governance.snapshots parent residual judgment selects read_routes

> Batch: BE-001NM-01
> Node: `backend.ops_governance.snapshots`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots` returns to parent residual judgment after `create_flow` closed as a final child.

The next child is fixed as:

`backend.ops_governance.snapshots.read_routes`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.snapshots.read_routes` | List snapshots, get snapshot by ID, memory-first get, disk fallback through parent helper. | Select for next baseline. |
| `backend.ops_governance.snapshots.restore_flow` | Signature verification, restore audit, stale run/backtest cleanup, restore response. | Keep in parent residual queue. |
| `backend.ops_governance.snapshots.persistence` | Snapshot atomic write, restore audit write, and disk load behavior. | Keep in parent residual queue. |
| `backend.ops_governance.snapshots.signature_contract` | Shared signature input construction, if create/restore extraction leaves it as a reusable contract. | Keep as conditional residual. |
| `backend.ops_governance.snapshots.route_facade` | Route registration for create/list/get/restore. | Keep as parent-owned route surface for now. |

Closed children:

- `backend.ops_governance.snapshots.snapshot_id_validation`
- `backend.ops_governance.snapshots.create_flow`

## Selected Child Boundary

`backend.ops_governance.snapshots.read_routes` currently contains:

- `list_snapshots`;
- `get_snapshot`;
- `state.snapshots` read-lock projection;
- descending `created_at_ms` sort;
- pagination;
- memory-first snapshot lookup;
- parent-owned disk fallback call.

## Hard Boundaries

BE-001NN-01/02 must not move:

- create flow child;
- restore handler;
- snapshot ID validation child;
- disk load file read and JSON parse implementation;
- snapshot persistence or restore audit persistence implementation;
- shared signature helper;
- route facade unless a later static closeout selects it;
- AppState cleanup behavior;
- storage lifecycle internals;
- runbook, chaos, hotswap, sandbox, or alerts code;
- release transition logic.

No sibling shortcut is allowed. If read routes need disk fallback, the call must go through the snapshots parent implementation owner until persistence/disk-load baseline is frozen.

## Next Step

BE-001NN-01 backend.ops_governance.snapshots.read_routes baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
