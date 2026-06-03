# v4.16.0 backend.ops_governance.alerts.trigger_engine equivalence baseline and extraction plan

> Batch: BE-001MX-01
> Node: `backend.ops_governance.alerts.trigger_engine`
> Parent: `backend.ops_governance.alerts`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts.trigger_engine` is frozen as the alert check route orchestration child.

BE-001MX-01 does not move code. It defines the exact baseline and allowed movement for BE-001MX-02.

## Current Owner

Current implementation is still in `src/backend/ops_governance/alerts/handlers.rs`.

The child boundary is:

- `trigger_alert_check`;
- rule snapshot read from `state.alert_rules`;
- enabled-rule filtering;
- already-firing deduplication;
- predicate dispatch through the parent-owned `should_fire_alert`;
- new `AlertFiring` construction and memory insertion;
- post-lock persistence call for new firings;
- auto-recovery scan through parent-owned `is_condition_resolved`;
- resolved firing state mutation;
- post-lock persistence call for recovered firings;
- resolved firing memory cleanup;
- resolved firing file cleanup.

## Frozen Semantics

The next extraction must preserve:

| Surface | Frozen behavior |
| --- | --- |
| Route shape | Parent route registration still posts to `/api/v1/alerts/check`. |
| Rule snapshot | Rules are cloned from `state.alert_rules` before iteration. |
| Enabled filter | Disabled rules are skipped. |
| Deduplication | Existing `Firing` records for the same rule skip new firing creation. |
| Predicate owner | Predicate implementation remains parent-owned and is called by the child. |
| Firing shape | Firing id, severity, state, timestamps, detail, and scoped key remain unchanged. |
| Lock boundary | Persistence for new and recovered firings remains outside write locks. |
| Auto-recovery | Rules with resolve conditions, plus `event_orphan_detected`, can auto-resolve. |
| Cleanup | Resolved firings are removed from memory and their disk files are deleted. |

## Allowed BE-001MX-02 Movement

BE-001MX-02 may:

- create a private child module for trigger engine under the alerts handler owner boundary;
- move `trigger_alert_check` into that child;
- update parent route registration to call the private child function;
- call existing parent-owned predicate helpers;
- call existing parent-owned persistence helper.

## Forbidden BE-001MX-02 Movement

BE-001MX-02 must not move or rewrite:

- alert list or alert rule list routes;
- rule catalog;
- acknowledge flow;
- predicate helper implementations;
- persistence helper implementation;
- startup initialization bridge;
- AppState fields or lock ordering beyond preserving existing lock boundaries;
- frontend API schema types;
- snapshots, runbook, chaos, hotswap, or sandbox modules;
- release transition logic.

## Proof

No direct trigger route test was found in the current alerts handler test filter. BE-001MX-02 must therefore keep the movement mechanical and prove equivalence with compile, existing alerts tests, and governance gates.

## Next Step

BE-001MX-02 backend.ops_governance.alerts.trigger_engine extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
