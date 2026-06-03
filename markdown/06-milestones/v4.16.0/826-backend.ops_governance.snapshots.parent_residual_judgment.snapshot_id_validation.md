# v4.16.0 backend.ops_governance.snapshots parent residual judgment selects snapshot_id_validation

> Batch: BE-001NI-01
> Node: `backend.ops_governance.snapshots`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots` returns to parent residual judgment after BE-001NH-03 confirmed `stop_split: false`.

The next child is fixed as:

`backend.ops_governance.snapshots.snapshot_id_validation`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.snapshots.snapshot_id_validation` | Validates snapshot IDs before disk path construction and disk read. | Select for next baseline. |
| `backend.ops_governance.snapshots.create_flow` | Create request DTO, event bounds assembly, signature creation, persistence call, and memory insert. | Keep in parent residual queue. |
| `backend.ops_governance.snapshots.read_routes` | List/get projection, memory-first get, disk fallback. | Keep in parent residual queue. |
| `backend.ops_governance.snapshots.restore_flow` | Signature verification, restore audit, stale run/backtest cleanup, restore response. | Keep in parent residual queue. |
| `backend.ops_governance.snapshots.persistence` | Snapshot atomic write, restore audit write, and disk load behavior. | Keep in parent residual queue. |
| `backend.ops_governance.snapshots.signature_contract` | Shared signature input construction, if later extractions leave it as a reusable contract. | Keep as conditional residual. |

## Selected Child Boundary

`backend.ops_governance.snapshots.snapshot_id_validation` currently contains:

- `validate_snapshot_id(id: &str) -> Result<(), String>`;
- direct accept/reject tests for valid IDs, empty IDs, path traversal, path separators, NUL, and invalid characters;
- the guard used by disk load before constructing `{snapshot_id}.json`.

## Hard Boundaries

BE-001NJ-01/02 must not move:

- create request DTO;
- create/list/get/restore route handlers;
- snapshot signature construction;
- snapshot persistence or restore audit persistence;
- disk load file read and JSON parse behavior;
- AppState lock behavior;
- runtime run/backtest cleanup behavior;
- storage lifecycle internals;
- runbook, chaos, hotswap, sandbox, or alerts code;
- release transition logic.

No sibling shortcut is allowed. The child may be called only through the snapshots parent implementation owner.

## Next Step

BE-001NJ-01 backend.ops_governance.snapshots.snapshot_id_validation baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
