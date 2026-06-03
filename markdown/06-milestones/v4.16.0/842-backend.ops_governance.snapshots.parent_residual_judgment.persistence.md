# v4.16.0 backend.ops_governance.snapshots parent residual judgment selects persistence

> Batch: BE-001NQ-01
> Node: `backend.ops_governance.snapshots`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots` returns to parent residual judgment after `restore_flow` closed as a final child.

The next child is fixed as:

`backend.ops_governance.snapshots.persistence`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.snapshots.persistence` | Snapshot atomic write, restore audit write, disk load path/read/parse, storage quota call, and storage error mapping. | Select for next baseline. |
| `backend.ops_governance.snapshots.signature_contract` | Shared signature input construction used by create and restore flows. | Keep as conditional residual. |
| `backend.ops_governance.snapshots.route_facade` | Route registration for create/list/get/restore. | Keep as parent-owned route surface for now. |

Closed children:

- `backend.ops_governance.snapshots.snapshot_id_validation`
- `backend.ops_governance.snapshots.create_flow`
- `backend.ops_governance.snapshots.read_routes`
- `backend.ops_governance.snapshots.restore_flow`

## Selected Child Boundary

`backend.ops_governance.snapshots.persistence` currently contains:

- `persist_snapshot_restore_audit`;
- `persist_snapshot`;
- `load_snapshot_from_disk`;
- snapshot storage quota check;
- snapshot directory creation;
- snapshot file path construction;
- restore audit file path construction;
- atomic JSON writes through `runtime_persistence`;
- disk read and JSON parse error mapping;
- snapshot ID validation call before disk path construction.

## Hard Boundaries

BE-001NR-01/02 must not move:

- create flow child;
- read routes child;
- restore flow child;
- snapshot ID validation child implementation;
- shared signature helper implementation;
- route facade;
- AppState memory insert/read/cleanup behavior;
- storage lifecycle internals beyond the existing call site;
- runtime persistence internals beyond the existing call site;
- runbook, chaos, hotswap, sandbox, or alerts code;
- release transition logic.

No sibling shortcut is allowed. Closed children may call persistence only through the snapshots parent until the persistence baseline explicitly defines any re-export or parent-mediated helper surface.

## Next Step

BE-001NR-01 backend.ops_governance.snapshots.persistence baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
