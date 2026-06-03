# v4.16.0 backend.ops_governance.snapshots parent residual judgment selects create_flow

> Batch: BE-001NK-01
> Node: `backend.ops_governance.snapshots`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots` returns to parent residual judgment after `snapshot_id_validation` closed as a final child.

The next child is fixed as:

`backend.ops_governance.snapshots.create_flow`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.snapshots.create_flow` | Create request DTO, event bounds assembly, snapshot ID creation, signature calculation, persistence call, and memory insert. | Select for next baseline. |
| `backend.ops_governance.snapshots.read_routes` | List/get projection, memory-first get, disk fallback. | Keep in parent residual queue. |
| `backend.ops_governance.snapshots.restore_flow` | Signature verification, restore audit, stale run/backtest cleanup, restore response. | Keep in parent residual queue. |
| `backend.ops_governance.snapshots.persistence` | Snapshot atomic write, restore audit write, and disk load behavior. | Keep in parent residual queue. |
| `backend.ops_governance.snapshots.signature_contract` | Shared signature input construction, if create/restore extraction leaves it as a reusable contract. | Keep as conditional residual. |
| `backend.ops_governance.snapshots.route_facade` | Route registration for create/list/get/restore. | Keep as parent-owned route surface for now. |

Closed children:

- `backend.ops_governance.snapshots.snapshot_id_validation`

## Selected Child Boundary

`backend.ops_governance.snapshots.create_flow` currently contains:

- `CreateSnapshotRequest`;
- `create_snapshot`;
- missing-body BAD_REQUEST mapping;
- `snap-{current_time_ms}` ID creation;
- `EventSliceBounds` construction from request fields;
- shared signature input call and canonical SHA-256 digest;
- `DeploymentSignatureSnapshot` assembly;
- parent-owned persistence call;
- `state.snapshots` write-lock insertion;
- direct request serialization test.

## Hard Boundaries

BE-001NL-01/02 must not move:

- list/get/restore handlers;
- snapshot ID validation child;
- disk load file read and JSON parse behavior;
- restore audit persistence;
- stale run/backtest cleanup;
- storage lifecycle internals;
- shared signature helper unless the baseline explicitly keeps it parent-owned;
- runbook, chaos, hotswap, sandbox, or alerts code;
- release transition logic.

No sibling shortcut is allowed. If create flow needs shared signature or persistence helpers, those calls must go through the snapshots parent implementation owner until their own baselines are frozen.

## Next Step

BE-001NL-01 backend.ops_governance.snapshots.create_flow baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
