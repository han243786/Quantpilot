# v4.16.0 backend.ops_governance.snapshots.snapshot_id_validation equivalence baseline and extraction plan

> Batch: BE-001NJ-01
> Node: `backend.ops_governance.snapshots.snapshot_id_validation`
> Parent: `backend.ops_governance.snapshots`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots.snapshot_id_validation` is frozen as the first child inside the extracted snapshots owner.

BE-001NJ-01 does not move code. It defines the exact baseline and allowed movement for BE-001NJ-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/snapshots/handlers.rs`

Current selected function:

- `validate_snapshot_id(id: &str) -> Result<(), String>`

Current callers:

- `load_snapshot_from_disk(store_dir, snapshot_id)` calls `validate_snapshot_id(snapshot_id)` before constructing `{snapshot_id}.json`.

## Frozen Behavior

| Case | Frozen behavior |
| --- | --- |
| Empty ID | Reject with the existing empty-ID error message. |
| Overlength ID | Reject IDs longer than 128 characters. |
| Path traversal or separators | Reject IDs containing `..`, `/`, `\`, or NUL. |
| Character set | Accept only ASCII alphanumeric characters, `_`, and `-`. |
| Valid examples | Continue accepting `snap-123`, `abc_def`, and `my-snapshot-001`. |
| Disk load order | Disk load must validate before path construction and file read. |

## Allowed BE-001NJ-02 Movement

BE-001NJ-02 may:

- create `src/backend/ops_governance/snapshots/handlers/snapshot_id_validation.rs`;
- move `validate_snapshot_id` into that private child module;
- move the two direct validation tests with the child;
- add a private `mod snapshot_id_validation;` declaration in `src/backend/ops_governance/snapshots/handlers.rs`;
- call the child from the handlers parent implementation owner.

## Forbidden BE-001NJ-02 Movement

BE-001NJ-02 must not move or rewrite:

- snapshot create/list/get/restore route handlers;
- `load_snapshot_from_disk` file path construction, file read, or JSON parse behavior;
- snapshot persistence or restore audit persistence;
- signature input construction;
- request/response DTO schema;
- AppState lock behavior or stale run/backtest cleanup;
- storage lifecycle internals;
- runbook, chaos, hotswap, sandbox, or alerts code;
- release transition logic.

## Parent-Child Rule

The child must stay private under the current snapshots handler implementation owner.

Allowed call path:

`handlers.rs` -> private `handlers::snapshot_id_validation`.

Forbidden call path:

Any sibling ops module or release-transition shortcut directly calling the validation child.

## Proof

BE-001NJ-02 must prove equivalence with:

- `cargo test -p quantpilot validate_snapshot_id`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `cargo check -p quantpilot`

## Next Step

BE-001NJ-02 backend.ops_governance.snapshots.snapshot_id_validation extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot validate_snapshot_id`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
