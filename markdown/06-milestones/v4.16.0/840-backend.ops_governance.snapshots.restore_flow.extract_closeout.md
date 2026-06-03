# v4.16.0 backend.ops_governance.snapshots.restore_flow actual extraction complete

> Batch: BE-001NP-02
> Node: `backend.ops_governance.snapshots.restore_flow`
> Parent: `backend.ops_governance.snapshots`
> Stage: `extract_closeout`
> Movement: Snapshot restore handler moved into a private child module.

---

## Summary

`backend.ops_governance.snapshots.restore_flow` now owns the snapshot restore handler.

The snapshots handler parent still owns disk load, restore audit persistence, shared signature input construction, and route facade mediation.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/snapshots/handlers.rs` | `src/backend/ops_governance/snapshots/handlers/restore_flow.rs` | `restore_snapshot` moved. |
| `src/backend/ops_governance/snapshots/handlers.rs` | `src/backend/ops_governance/snapshots/handlers.rs` | Parent declares the private child module and routes POST `/api/v1/snapshots/:snapshot_id/restore` to the child. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| Snapshot lookup | Restore still checks memory first and falls back through parent-owned disk load. |
| Signature verification | Restore still calls parent-owned signature input helper and canonical SHA-256 digest. |
| Signature mismatch | Mismatch still returns BAD_REQUEST conflict with the existing message shape. |
| Audit persistence | Restore still calls parent-owned audit persistence before response cleanup. |
| Response shape | Restore JSON still includes the same fields and warning. |
| Runtime cleanup | Restore still retains only runs/backtests newer than `now_ms`. |
| Logging | Restore lifecycle logs remain in the handler flow. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- `handlers.rs` route registration -> private `handlers::restore_flow::restore_snapshot`;
- `restore_flow` child -> snapshots handler parent helpers (`load_snapshot_from_disk`, `build_signature_input`, `persist_snapshot_restore_audit`).

The following remain outside this child:

- create flow child;
- read routes child;
- snapshot ID validation child;
- disk load file path construction, file read, JSON parse, and error mapping;
- snapshot persistence and restore audit persistence implementation;
- shared signature helper implementation;
- route facade beyond handler reference;
- storage lifecycle internals, sibling ops modules, and release transition logic.

## Proof

- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`

## Next Step

BE-001NP-03 backend.ops_governance.snapshots.restore_flow single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
