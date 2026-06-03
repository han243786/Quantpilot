# v4.16.0 backend.ops_governance.snapshots.persistence equivalence baseline and extraction plan

> Batch: BE-001NR-01
> Node: `backend.ops_governance.snapshots.persistence`
> Parent: `backend.ops_governance.snapshots`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots.persistence` is frozen as the snapshot disk persistence and disk load child.

BE-001NR-01 does not move code. It defines the exact baseline and allowed movement for BE-001NR-02.

## Current Owner

Current implementation is still in `src/backend/ops_governance/snapshots/handlers.rs`.

The child boundary is:

- `persist_snapshot_restore_audit`;
- `persist_snapshot`;
- `load_snapshot_from_disk`.

The parent bridge must remain:

- `persist_snapshot_restore_audit`;
- `persist_snapshot`;
- `load_snapshot_from_disk`.

Closed children must continue to call the parent bridge, not the child directly.

## Frozen Semantics

The next extraction must preserve:

| Surface | Frozen behavior |
| --- | --- |
| Snapshot write return type | `persist_snapshot` still returns `std::io::Result<()>`. |
| Restore audit write return type | `persist_snapshot_restore_audit` still returns `std::io::Result<()>`. |
| Disk load return type | `load_snapshot_from_disk` still returns `Result<DeploymentSignatureSnapshot, (StatusCode, String)>`. |
| Storage root | Snapshot write still calls `ensure_storage_quota(std::path::Path::new("storage"), "snapshots", StorageLifecycle::Transient)`. |
| Directory creation | Snapshot and audit writes still call async `fs::create_dir_all(...).await?` before writing. |
| Snapshot path | Snapshot write still writes to `store_dir.join(format!("{}.json", snapshot.snapshot_id))`. |
| Restore audit path | Restore audit still writes to `snapshot-restore-{snapshot_id}-{restored_at_ms}.json`. |
| Write primitive | Both writes still use `runtime_persistence::atomic_write_json`. |
| Disk guard | Disk load still calls `snapshot_id_validation::validate_snapshot_id` before path construction. |
| Disk path | Disk load still reads `store_dir.join(format!("{}.json", snapshot_id))`. |
| Error mapping | Invalid ID, missing file, and JSON parse errors keep their existing response mapping. |
| Caller contract | Create/read/restore children still pass the same arguments through the snapshots parent bridge. |

## Allowed BE-001NR-02 Movement

BE-001NR-02 may:

- create `src/backend/ops_governance/snapshots/handlers/persistence.rs`;
- move only the implementation bodies of `persist_snapshot_restore_audit`, `persist_snapshot`, and `load_snapshot_from_disk` into that private child module;
- add a private `mod persistence;` declaration in `src/backend/ops_governance/snapshots/handlers.rs`;
- keep parent bridge functions with the same names and signatures that delegate to the child;
- keep all existing create/read/restore child call sites parent-mediated.

## Forbidden BE-001NR-02 Movement

BE-001NR-02 must not move or rewrite:

- create flow child;
- read routes child;
- restore flow child;
- snapshot ID validation child implementation;
- shared signature helper implementation;
- signature deterministic test;
- event-bounds direct type test;
- route facade;
- AppState memory insert/read/cleanup behavior;
- storage lifecycle implementation internals;
- runtime persistence implementation internals;
- runbook, chaos, hotswap, sandbox, or alerts code;
- release transition logic.

## Parent-Child Rule

The child must stay private under the current snapshots handler implementation owner.

Allowed call paths:

- snapshots handler parent bridge -> private `handlers::persistence` functions;
- create/read/restore children -> snapshots handler parent bridge.

Forbidden call path:

Any create/read/restore child importing or calling `handlers::persistence` directly.

## Proof

No direct persistence unit test is currently isolated for snapshots. BE-001NR-02 must therefore keep the movement mechanical and prove equivalence with:

- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `cargo check -p quantpilot`

## Next Step

BE-001NR-02 backend.ops_governance.snapshots.persistence extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
