# v4.16.0 backend.ops_governance.snapshots.signature_contract actual extraction complete

> Batch: BE-001NT-02
> Node: `backend.ops_governance.snapshots.signature_contract`
> Parent: `backend.ops_governance.snapshots`
> Stage: `extract_closeout`
> Movement: Snapshot signature input construction moved into a private child module.

---

## Summary

`backend.ops_governance.snapshots.signature_contract` now owns shared snapshot signature input construction.

The snapshots handler parent still owns the bridge helper name used by create and restore children, so closed children remain parent-mediated.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/snapshots/handlers.rs` | `src/backend/ops_governance/snapshots/handlers/signature_contract.rs` | Implementation of `build_signature_input` moved. |
| `src/backend/ops_governance/snapshots/handlers.rs` | `src/backend/ops_governance/snapshots/handlers.rs` | Parent declares the private child module and keeps same-name bridge helper delegating to the child. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| Return type | Still returns `serde_json::Value`. |
| Field shape | Capability hash, strategy version, parameter version, core IR digest, event bounds, and created timestamp remain identical. |
| Caller contract | Create and restore children still call the snapshots parent bridge. |
| Digest owner | Canonical digest implementation remains outside this child. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- snapshots handler parent bridge -> private `handlers::signature_contract::build_signature_input`;
- create/restore children -> snapshots handler parent bridge.

The following remain outside this child:

- create flow child;
- read routes child;
- restore flow child;
- persistence child;
- snapshot ID validation child;
- route facade;
- canonical digest implementation;
- AppState memory insert/read/cleanup behavior;
- sibling ops modules and release transition logic.

## Proof

- `cargo check -p quantpilot`

## Next Step

BE-001NT-03 backend.ops_governance.snapshots.signature_contract single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
