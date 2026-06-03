# v4.16.0 backend.ops_governance.alerts parent residual judgment selects acknowledge_flow

> Batch: BE-001MU-01
> Node: `backend.ops_governance.alerts`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Decision

Select the next child:

`backend.ops_governance.alerts.acknowledge_flow`

BE-001MU-01 is a governance-only selection batch. It returns from the closed `rule_catalog` leaf to the alerts parent residual queue and selects the next concrete owner.

## Residual Review

Closed child:

- `backend.ops_governance.alerts.rule_catalog`

Remaining alerts residuals:

| Residual | Status | Decision |
| --- | --- | --- |
| acknowledge flow | Owns acknowledge request DTO, firing lookup, state transition, not_found mapping, and lock-free persistence call. | Select next. |
| trigger engine | Owns trigger loop, deduplication, predicate dispatch, auto-recovery, and cleanup. | Keep queued. |
| predicate checks | Owns metric-specific alert predicates and AppState reads. | Keep queued. |
| persistence | Owns alert firing disk write and atomic write behavior. | Keep queued. |
| startup init | Owns rule initialization bridge behavior. | Keep under alerts parent until child shape is clear. |

## Selection Rationale

`backend.ops_governance.alerts.acknowledge_flow` is selected because:

- it is a concrete write-path owner with an independent state transition contract;
- it has a distinct not_found error mapping from trigger and list routes;
- it already keeps disk persistence outside the alert firing write lock;
- extracting it reduces the route handler file without touching trigger logic, predicate checks, or persistence internals.

## Parent-Child Contract

BE-001MV-01 must freeze acknowledge flow as a parent-controlled private child of the alerts handler owner boundary.

The child may own:

- `AcknowledgeAlertRequest`;
- `acknowledge_alert`;
- scoped firing lookup by user and firing id;
- transition from `Firing` to `Acknowledged`;
- transition from `Acknowledged` to `Resolved`;
- `ERR_ALERT_NOT_FOUND` mapping for missing firings;
- the call site that asks the parent-owned persistence helper to persist the updated firing after the lock is dropped.

The child must not own:

- route registration outside the function reference;
- alert list or alert rule list routes;
- trigger engine;
- predicate checks;
- persistence helper implementation;
- rule catalog;
- startup compatibility bridge;
- AppState owner or lock ordering;
- frontend API schema types.

## Forbidden Movement

BE-001MU-01 and the next baseline must not move:

- snapshots, runbook, chaos, hotswap, or sandbox code;
- closed ops governance children;
- runtime, capability, storage security, or strategy config code;
- frontend callers or schemas;
- release transition logic.

No sibling shortcut is allowed. Persistence must remain mediated by the alerts parent/owner boundary until the persistence leaf gets its own baseline.

## Next Step

BE-001MV-01 backend.ops_governance.alerts.acknowledge_flow baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
