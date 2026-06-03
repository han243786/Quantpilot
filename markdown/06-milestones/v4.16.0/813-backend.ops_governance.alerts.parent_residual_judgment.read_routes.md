# v4.16.0 backend.ops_governance.alerts parent residual judgment selects read_routes

> Batch: BE-001NC-01
> Node: `backend.ops_governance.alerts`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts` returns to parent residual judgment after `startup_initialization` closed with `stop_split: true`.

Next selected child:

`backend.ops_governance.alerts.read_routes`

## Current Residual Map

Closed children:

- `backend.ops_governance.alerts.rule_catalog`;
- `backend.ops_governance.alerts.acknowledge_flow`;
- `backend.ops_governance.alerts.trigger_engine`;
- `backend.ops_governance.alerts.predicate_checks`;
- `backend.ops_governance.alerts.persistence`;
- `backend.ops_governance.alerts.startup_initialization`.

Remaining parent residuals:

- list/read route helpers;
- route registration facade;
- alert recovery predicate bridge.

## Selection Gate

| Rule | Result | Evidence |
| --- | --- | --- |
| Concrete owner exists? | Yes. | `list_alerts` and `list_alert_rules` own read-only alert projections. |
| Independent IO or state failure mode? | Partial. | The child is read-only and has no disk IO, but it owns user-scoped filtering and response assembly. |
| Parent-child communication improves? | Yes. | Route facade can call a focused read child while write, trigger, predicate, persistence, startup, and catalog leaves remain separate. |
| Local proof can remain focused? | Yes. | Movement can be mechanical and verified with compile, alerts handler tests, and governance gates. |
| Line count only? | No. | The split is driven by route read projection ownership and user-scoped filtering. |

## Selected Boundary

`backend.ops_governance.alerts.read_routes` owns only:

- `list_alerts`;
- `list_alert_rules`;
- user-scoped alert firing projection;
- alert rules read projection;
- `AlertListResponse` assembly.

The alerts parent may route directly to the child handlers because parent-to-child calls preserve the hard communication rule.

## Forbidden Movement

BE-001NC-02 must not move:

- route registration facade;
- rule catalog implementation;
- acknowledge route logic;
- trigger route logic;
- predicate dispatch;
- persistence implementation;
- startup initialization;
- alert recovery predicate bridge;
- DTO schema owner;
- AppState fields or lock ordering beyond preserving the current read locks;
- release transition logic.

## Next Step

BE-001NC-02 backend.ops_governance.alerts.read_routes baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
