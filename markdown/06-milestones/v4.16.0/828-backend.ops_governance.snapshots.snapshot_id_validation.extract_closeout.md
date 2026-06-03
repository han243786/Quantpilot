# v4.16.0 backend.ops_governance.snapshots.snapshot_id_validation actual extraction complete

> Batch: BE-001NJ-02
> Node: `backend.ops_governance.snapshots.snapshot_id_validation`
> Parent: `backend.ops_governance.snapshots`
> Stage: `extract_closeout`
> Movement: Snapshot ID validation moved into a private child module.

---

## Summary

`backend.ops_governance.snapshots.snapshot_id_validation` now owns the snapshot ID validation guard and its direct tests.

The snapshots handler parent still owns disk load path construction, file read, JSON parse behavior, and error mapping.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/snapshots/handlers.rs` | `src/backend/ops_governance/snapshots/handlers/snapshot_id_validation.rs` | `validate_snapshot_id` and its two direct validation tests moved. |
| `src/backend/ops_governance/snapshots/handlers.rs` | `src/backend/ops_governance/snapshots/handlers.rs` | Parent declares the private child module and calls `snapshot_id_validation::validate_snapshot_id` before disk path construction. |

## Preserved Behavior

| Case | Preserved behavior |
| --- | --- |
| Empty ID | Still rejected. |
| Overlength ID | IDs longer than 128 characters are still rejected. |
| Path traversal or separators | `..`, `/`, `\`, and NUL remain rejected. |
| Character set | Only ASCII alphanumeric characters, `_`, and `-` remain accepted. |
| Disk load order | Validation still runs before `{snapshot_id}.json` path construction and file read. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call path:

`handlers.rs` -> private `handlers::snapshot_id_validation`.

The following remain outside this child:

- create/list/get/restore route handlers;
- disk load file path construction, file read, and JSON parse behavior;
- snapshot persistence and restore audit persistence;
- signature input construction;
- AppState lock behavior and stale run/backtest cleanup;
- storage lifecycle internals;
- runbook, chaos, hotswap, sandbox, alerts, and release transition logic.

## Proof

- `cargo test -p quantpilot validate_snapshot_id`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `cargo check -p quantpilot`

## Next Step

BE-001NJ-03 backend.ops_governance.snapshots.snapshot_id_validation single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot validate_snapshot_id`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
