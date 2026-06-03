# v4.16.0 backend.ops_governance.alerts parent residual judgment selects startup_initialization

> Batch: BE-001NB-01
> Node: `backend.ops_governance.alerts`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts` returns to parent residual judgment after `persistence` closed with `stop_split: true`.

Next selected child:

`backend.ops_governance.alerts.startup_initialization`

## Current Residual Map

Closed children:

- `backend.ops_governance.alerts.rule_catalog`;
- `backend.ops_governance.alerts.acknowledge_flow`;
- `backend.ops_governance.alerts.trigger_engine`;
- `backend.ops_governance.alerts.predicate_checks`;
- `backend.ops_governance.alerts.persistence`.

Remaining parent residuals:

- startup rule initialization bridge;
- list/read route helpers;
- route registration facade;
- alert recovery predicate bridge.

## Selection Gate

| Rule | Result | Evidence |
| --- | --- | --- |
| Concrete owner exists? | Yes. | `init_alert_rules` owns startup-time alert rule seeding. |
| Independent IO or state failure mode? | Partial. | It owns a write lock and idempotent empty-store initialization, not disk IO. |
| Parent-child communication improves? | Yes. | It removes startup seeding from the route/read parent while preserving parent mediation to rule catalog defaults. |
| Local proof can remain focused? | Yes. | Movement can be mechanical and verified with compile, alerts handler tests, and governance gates. |
| Line count only? | No. | The split is driven by lifecycle phase and AppState write-lock ownership. |

## Selected Boundary

`backend.ops_governance.alerts.startup_initialization` owns only:

- alert rules write-lock acquisition for startup seeding;
- empty-rule check;
- assignment of default alert rules when the store is empty.

The parent bridge must remain:

- `init_alert_rules`

The child must not call `rule_catalog` directly. The alerts parent must mediate default rule access so sibling modules do not form a direct cross-link.

## Forbidden Movement

BE-001NB-02 must not move:

- rule catalog implementation;
- acknowledge route logic;
- trigger route logic;
- predicate dispatch;
- persistence implementation;
- list/read route handlers;
- route registration;
- DTO schema owner;
- AppState fields or lock ordering beyond preserving the current alert rules write lock;
- release transition logic.

## Next Step

BE-001NB-02 backend.ops_governance.alerts.startup_initialization baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
