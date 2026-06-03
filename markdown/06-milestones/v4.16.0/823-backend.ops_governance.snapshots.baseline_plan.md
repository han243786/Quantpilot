# v4.16.0 backend.ops_governance.snapshots equivalence baseline and extraction plan

> Batch: BE-001NH-01
> Node: `backend.ops_governance.snapshots`
> Parent: `backend.ops_governance`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots` is frozen as the snapshot route and handler owner candidate.

BE-001NH-01 does not move code. It defines the exact baseline and allowed movement for BE-001NH-02.

## Current Owner

Current facade:

- `src/backend/ops_governance/snapshots.rs`

Current handler owner:

- `src/snapshot_service.rs`

The current facade delegates route registration to `crate::snapshot_service::register_snapshot_routes(router)`.

## Frozen Route Surface

The next extraction must preserve:

| Method | Path | Handler |
| --- | --- | --- |
| GET | `/api/v1/snapshots` | `list_snapshots` |
| GET | `/api/v1/snapshots/:snapshot_id` | `get_snapshot` |
| POST | `/api/v1/snapshots/:snapshot_id/restore` | `restore_snapshot` |
| POST | `/api/v1/snapshots/create` | `create_snapshot` |

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Create request DTO | `CreateSnapshotRequest` keeps deny-unknown-fields and all current fields. |
| Missing create body | Missing request body still returns BAD_REQUEST with the current message shape. |
| Snapshot ID creation | Created snapshots still use `snap-{current_time_ms}`. |
| Event bounds | `EventSliceBounds` still comes from request event IDs, sequences, and event count. |
| Signature input | `build_signature_input` still covers capability hash, strategy version, parameter version, core IR digest, event slice bounds, and created timestamp. |
| Signature digest | Still uses `canonical_json_sha256_digest` and maps failures through `internal_error`. |
| Snapshot persistence | Created snapshots are still persisted before insertion into `state.snapshots`. |
| List route | Still clones in-memory snapshots, sorts descending by `created_at_ms`, and paginates. |
| Get route | Still checks memory first and falls back to disk load. |
| Restore route | Still checks memory first, falls back to disk load, verifies signature, writes restore audit, returns restore JSON, and clears stale runs/backtests. |
| Snapshot persistence helper | Still enforces transient storage quota for namespace `snapshots`, creates directory, and atomically writes JSON. |
| Disk load helper | Still validates snapshot ID, reads `{snapshot_id}.json`, maps missing file to `not_found`, and parses JSON. |
| Snapshot ID validation | Empty, overlength, path separator, NUL, non-ASCII, and non `[A-Za-z0-9_-]` IDs remain rejected. |
| Tests | Existing embedded tests for deterministic signature, event bounds, ID validation, and request serialization remain equivalent. |

## Allowed BE-001NH-02 Movement

BE-001NH-02 may:

- move the snapshot route and handler implementation from `src/snapshot_service.rs` into a private implementation module under `src/backend/ops_governance/snapshots/`;
- keep `src/snapshot_service.rs` as a compatibility bridge that delegates to `backend.ops_governance.snapshots`;
- update `src/backend/ops_governance/snapshots.rs` to own the moved handler implementation;
- move the embedded snapshot tests with the implementation owner.

## Forbidden BE-001NH-02 Movement

BE-001NH-02 must not move or rewrite:

- runbook route or handler owner;
- chaos route or handler owner;
- closed hotswap, sandbox, or alerts internals;
- runtime mutation activation snapshot side effects;
- AppState fields or lock ordering;
- frontend API schema owner;
- storage lifecycle internals;
- runtime persistence internals;
- release transition logic.

## Proof

BE-001NH-02 must keep the movement mechanical and prove equivalence with compile, snapshot-focused tests, and governance gates.

Candidate focused test:

- `cargo test -p quantpilot snapshot_service`

## Next Step

BE-001NH-02 backend.ops_governance.snapshots extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot snapshot_service`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
