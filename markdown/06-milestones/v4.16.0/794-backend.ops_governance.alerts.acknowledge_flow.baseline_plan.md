# v4.16.0 backend.ops_governance.alerts.acknowledge_flow equivalence baseline and extraction plan

> Batch: BE-001MV-01
> Node: `backend.ops_governance.alerts.acknowledge_flow`
> Parent: `backend.ops_governance.alerts`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts.acknowledge_flow` is frozen as the alert acknowledgment write-path child.

BE-001MV-01 does not move code. It defines the exact baseline and allowed movement for BE-001MV-02.

## Current Owner

Current implementation is still in `src/backend/ops_governance/alerts/handlers.rs`.

The child boundary is:

- `AcknowledgeAlertRequest`;
- `acknowledge_alert`;
- scoped lookup using `auth::scoped_key`;
- missing firing response using `ERR_ALERT_NOT_FOUND`;
- transition from `Firing` to `Acknowledged`;
- transition from `Acknowledged` to `Resolved`;
- clone of the updated firing after mutation;
- persistence call after the `alert_firings` write lock is dropped.

## Frozen Semantics

The next extraction must preserve:

| Surface | Frozen behavior |
| --- | --- |
| Route shape | Parent route registration still posts to `/api/v1/alerts/:firing_id/acknowledge`. |
| Request DTO | Unknown JSON fields remain denied; `actor_id` remains required. |
| Scope | Lookup uses the authenticated user id plus firing id. |
| Missing firing | Missing records return `ERR_ALERT_NOT_FOUND` through the existing not_found mapper. |
| First acknowledge | Non-acknowledged firings become `Acknowledged`, set `acknowledged_at_ms`, and store `acknowledged_by`. |
| Repeat acknowledge | Already acknowledged firings become `Resolved` and set `resolved_at_ms`. |
| Lock boundary | Disk persistence remains outside the `alert_firings` write lock. |
| Persistence owner | The persistence helper implementation remains in the alerts parent owner. |

## Allowed BE-001MV-02 Movement

BE-001MV-02 may:

- create a private child module for acknowledge flow under the alerts handler owner boundary;
- move `AcknowledgeAlertRequest` into that child;
- move `acknowledge_alert` into that child;
- update parent route registration to call the private child function;
- call the existing parent-owned persistence helper after the lock is dropped.

## Forbidden BE-001MV-02 Movement

BE-001MV-02 must not move or rewrite:

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

## Proof

No direct acknowledge route test was found in the current alerts handler test filter. BE-001MV-02 must therefore keep the movement mechanical and prove equivalence with compile, existing alerts tests, and governance gates.

## Next Step

BE-001MV-02 backend.ops_governance.alerts.acknowledge_flow extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
