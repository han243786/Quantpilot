# v4.16.0 backend.ops_governance.snapshots parent residual judgment selects restore_flow

> Batch: BE-001NO-01
> Node: `backend.ops_governance.snapshots`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.snapshots` returns to parent residual judgment after `read_routes` closed as a final child.

The next child is fixed as:

`backend.ops_governance.snapshots.restore_flow`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.snapshots.restore_flow` | Restore handler, memory-first snapshot lookup, signature verification call, audit call, response assembly, and stale run/backtest cleanup. | Select for next baseline. |
| `backend.ops_governance.snapshots.persistence` | Snapshot atomic write, restore audit write, and disk load behavior. | Keep in parent residual queue. |
| `backend.ops_governance.snapshots.signature_contract` | Shared signature input construction used by create and restore flows. | Keep as conditional residual. |
| `backend.ops_governance.snapshots.route_facade` | Route registration for create/list/get/restore. | Keep as parent-owned route surface for now. |

Closed children:

- `backend.ops_governance.snapshots.snapshot_id_validation`
- `backend.ops_governance.snapshots.create_flow`
- `backend.ops_governance.snapshots.read_routes`

## Selected Child Boundary

`backend.ops_governance.snapshots.restore_flow` currently contains:

- `restore_snapshot`;
- memory-first lookup from `state.snapshots`;
- parent-owned disk fallback call;
- shared signature input call and canonical SHA-256 digest;
- conflict error on signature mismatch;
- parent-owned restore audit persistence call;
- restore response JSON assembly;
- stale `state.runs` and `state.backtests` cleanup;
- restore lifecycle log messages.

## Hard Boundaries

BE-001NP-01/02 must not move:

- create flow child;
- read routes child;
- snapshot ID validation child;
- disk load file path construction, file read, JSON parse, or error mapping;
- snapshot persistence or restore audit persistence implementation;
- shared signature helper implementation;
- route facade beyond swapping handler function references;
- storage lifecycle internals;
- runbook, chaos, hotswap, sandbox, or alerts code;
- release transition logic.

No sibling shortcut is allowed. If restore flow needs disk fallback, signature helper, or audit persistence, those calls must go through the snapshots parent implementation owner until their own baselines are frozen.

## Next Step

BE-001NP-01 backend.ops_governance.snapshots.restore_flow baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::snapshots`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
