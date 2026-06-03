# v4.16.0 backend.ops_governance.snapshots.create_flow equivalence baseline and extraction plan

> Batch: BE-001NL-01
> Node: `backend.ops_governance.snapshots.create_flow`
> Parent: `backend.ops_governance.snapshots`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots.create_flow` is frozen as the snapshot creation write-path child.

BE-001NL-01 does not move code. It defines the exact baseline and allowed movement for BE-001NL-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/snapshots/handlers.rs`

Current selected items:

- `CreateSnapshotRequest`;
- `create_snapshot`;
- direct `create_snapshot_request_serialization` test.

Current parent-owned helpers used by the selected handler:

- `build_signature_input`;
- `persist_snapshot`;
- `current_time_ms`;
- `canonical_json_sha256_digest`;
- `internal_error` and `io_error`.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Missing body | Missing request body still returns BAD_REQUEST with the current message shape. |
| DTO schema | `CreateSnapshotRequest` keeps deny-unknown-fields and all current fields. |
| Snapshot ID | Created snapshot IDs remain `snap-{current_time_ms}`. |
| Event bounds | `EventSliceBounds` still maps request event IDs, sequences, and event count exactly. |
| Signature | Create flow still calls the parent-owned shared signature input helper and canonical SHA-256 digest. |
| Persistence order | Snapshot persistence still happens before `state.snapshots` write-lock insertion. |
| Memory insert | Snapshot still inserts by generated snapshot ID and returns the cloned `DeploymentSignatureSnapshot`. |
| Direct test | Request serialization proof stays with the create flow child. |

## Allowed BE-001NL-02 Movement

BE-001NL-02 may:

- create `src/backend/ops_governance/snapshots/handlers/create_flow.rs`;
- move `CreateSnapshotRequest` into that private child module;
- move `create_snapshot` into that private child module;
- move the direct `create_snapshot_request_serialization` test with the child;
- add a private `mod create_flow;` declaration in `src/backend/ops_governance/snapshots/handlers.rs`;
- update route registration to call `create_flow::create_snapshot`.

## Forbidden BE-001NL-02 Movement

BE-001NL-02 must not move or rewrite:

- list/get/restore handlers;
- snapshot ID validation child;
- disk load file read and JSON parse behavior;
- snapshot persistence or restore audit persistence implementation;
- `build_signature_input`;
- signature deterministic test;
- event-bounds direct type test;
- AppState lock behavior outside create insertion;
- stale run/backtest cleanup;
- storage lifecycle internals;
- runbook, chaos, hotswap, sandbox, or alerts code;
- release transition logic.

## Parent-Child Rule

The child must stay private under the current snapshots handler implementation owner.

Allowed call paths:

- `handlers.rs` route registration -> private `handlers::create_flow::create_snapshot`;
- `create_flow` child -> snapshots handler parent helpers (`build_signature_input`, `persist_snapshot`) through the parent module.

Forbidden call path:

Any sibling ops module or release-transition shortcut directly calling the create flow child.

## Proof

BE-001NL-02 must prove equivalence with:

- `cargo test -p quantpilot create_snapshot_request_serialization`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `cargo check -p quantpilot`

## Next Step

BE-001NL-02 backend.ops_governance.snapshots.create_flow extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot create_snapshot_request_serialization`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
