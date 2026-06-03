# v4.16.0 backend.ops_governance.snapshots.restore_flow equivalence baseline and extraction plan

> Batch: BE-001NP-01
> Node: `backend.ops_governance.snapshots.restore_flow`
> Parent: `backend.ops_governance.snapshots`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots.restore_flow` is frozen as the snapshot restore orchestration child.

BE-001NP-01 does not move code. It defines the exact baseline and allowed movement for BE-001NP-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/snapshots/handlers.rs`

Current selected handler:

- `restore_snapshot`.

Current parent-owned helpers used by the selected handler:

- `load_snapshot_from_disk`;
- `build_signature_input`;
- `persist_snapshot_restore_audit`;
- `current_time_ms`;
- `canonical_json_sha256_digest`;
- `internal_error` and `io_error`.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Snapshot lookup | Restore still checks `state.snapshots` memory first and falls back through parent-owned disk load. |
| Signature verification | Restore still calls parent-owned signature input helper and canonical SHA-256 digest. |
| Signature mismatch | Mismatch still returns BAD_REQUEST conflict with the existing message shape. |
| Audit persistence | Restore still calls parent-owned restore audit persistence before response cleanup. |
| Response shape | Restore JSON still includes snapshot ID, deployment revision, strategy version, parameter version, restored timestamp, actor, reason, status, and warning. |
| Runtime cleanup | Restore still retains only runs/backtests newer than `now_ms`. |
| Logging | Restore lifecycle log messages remain in the handler flow. |

## Allowed BE-001NP-02 Movement

BE-001NP-02 may:

- create `src/backend/ops_governance/snapshots/handlers/restore_flow.rs`;
- move `restore_snapshot` into that private child module;
- add a private `mod restore_flow;` declaration in `src/backend/ops_governance/snapshots/handlers.rs`;
- update route registration to call `restore_flow::restore_snapshot`.

## Forbidden BE-001NP-02 Movement

BE-001NP-02 must not move or rewrite:

- create flow child;
- read routes child;
- snapshot ID validation child;
- disk load file path construction, file read, JSON parse, or error mapping;
- snapshot persistence or restore audit persistence implementation;
- shared signature helper implementation;
- signature deterministic test;
- event-bounds direct type test;
- route facade beyond swapping handler function references;
- storage lifecycle internals;
- runbook, chaos, hotswap, sandbox, or alerts code;
- release transition logic.

## Parent-Child Rule

The child must stay private under the current snapshots handler implementation owner.

Allowed call paths:

- `handlers.rs` route registration -> private `handlers::restore_flow::restore_snapshot`;
- `restore_flow` child -> snapshots handler parent helpers (`load_snapshot_from_disk`, `build_signature_input`, `persist_snapshot_restore_audit`).

Forbidden call path:

Any sibling ops module or release-transition shortcut directly calling restore flow or its helper dependencies.

## Proof

BE-001NP-02 must prove equivalence with:

- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `cargo check -p quantpilot`

## Next Step

BE-001NP-02 backend.ops_governance.snapshots.restore_flow extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
