# v4.16.0 backend.ops_governance.snapshots parent residual judgment selects signature_contract

> Batch: BE-001NS-01
> Node: `backend.ops_governance.snapshots`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots` returns to parent residual judgment after `persistence` closed as a final child.

The next child is fixed as:

`backend.ops_governance.snapshots.signature_contract`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.snapshots.signature_contract` | Shared signature input construction used by create and restore flows. | Select for next baseline. |
| `backend.ops_governance.snapshots.route_facade` | Route registration for create/list/get/restore. | Keep as parent-owned route surface for now. |

Closed children:

- `backend.ops_governance.snapshots.snapshot_id_validation`
- `backend.ops_governance.snapshots.create_flow`
- `backend.ops_governance.snapshots.read_routes`
- `backend.ops_governance.snapshots.restore_flow`
- `backend.ops_governance.snapshots.persistence`

## Selected Child Boundary

`backend.ops_governance.snapshots.signature_contract` currently contains:

- `build_signature_input`;
- capability hash field projection;
- strategy version field projection;
- parameter version field projection;
- core IR digest field projection;
- event slice bounds field projection;
- created timestamp field projection.

## Hard Boundaries

BE-001NT-01/02 must not move:

- create flow child;
- read routes child;
- restore flow child;
- persistence child;
- snapshot ID validation child;
- route facade;
- canonical digest implementation;
- AppState memory insert/read/cleanup behavior;
- storage lifecycle or runtime persistence internals;
- runbook, chaos, hotswap, sandbox, or alerts code;
- release transition logic.

No sibling shortcut is allowed. Create and restore children must continue to call the snapshots parent bridge for signature input until the signature contract baseline explicitly defines the parent-mediated helper surface.

## Next Step

BE-001NT-01 backend.ops_governance.snapshots.signature_contract baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
