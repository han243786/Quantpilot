# v4.16.0 backend.ops_governance.alerts.acknowledge_flow actual extraction complete

> Batch: BE-001MV-02
> Node: `backend.ops_governance.alerts.acknowledge_flow`
> Parent: `backend.ops_governance.alerts`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001MV-02 extracted the alert acknowledgment write path into a private child module under the alerts handler owner boundary.

Concrete movement:

- Added `src/backend/ops_governance/alerts/handlers/acknowledge_flow.rs`.
- Moved `AcknowledgeAlertRequest` into that child.
- Moved `acknowledge_alert` into that child.
- Updated parent route registration to call `acknowledge_flow::acknowledge_alert`.
- Kept `persist_alert_firing` in the alerts handler parent.
- Kept disk persistence after the alert firing write lock is dropped.

## Equivalence

The extraction preserves the frozen BE-001MV-01 baseline:

| Contract | Result |
| --- | --- |
| Route shape | Parent still routes POST `/api/v1/alerts/:firing_id/acknowledge` to the acknowledge handler. |
| Request DTO | Unknown JSON fields remain denied; the DTO is visible to the parent because it appears in the route handler signature. |
| Scope | Lookup still uses the authenticated user id plus firing id. |
| Missing firing | Missing records still return `ERR_ALERT_NOT_FOUND` through the existing not_found mapper. |
| First acknowledge | Non-acknowledged firings still become `Acknowledged`, with timestamp and actor. |
| Repeat acknowledge | Already acknowledged firings still become `Resolved`, with resolved timestamp. |
| Lock boundary | Persistence remains outside the write lock. |
| Parent mediation | The child calls the parent-owned persistence helper; no persistence sibling was introduced. |

## Untouched Areas

BE-001MV-02 did not move:

- alert list or alert rule list routes;
- rule catalog;
- trigger engine;
- predicate checks;
- persistence helper implementation;
- startup initialization bridge;
- AppState fields or lock ordering beyond preserving the existing lock drop;
- frontend API schema types;
- snapshots, runbook, chaos, hotswap, or sandbox modules;
- release transition logic.

## Residual

`backend.ops_governance.alerts.acknowledge_flow` needs single-leaf closeout before it can be marked complete.

Expected closeout decision:

`stop_split: true`

Rationale to verify in closeout:

- the child owns one write-path transition contract;
- deeper split would separate request DTO, lookup, transition, and persistence call into micro-leaves with higher communication cost;
- persistence implementation remains queued as its own future residual.

## Next Step

BE-001MV-03 backend.ops_governance.alerts.acknowledge_flow single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
