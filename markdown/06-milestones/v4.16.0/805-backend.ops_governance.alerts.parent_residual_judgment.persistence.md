# v4.16.0 backend.ops_governance.alerts parent residual judgment selects persistence

> Batch: BE-001NA-01
> Node: `backend.ops_governance.alerts`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts` returns to parent residual judgment after `predicate_checks` closed with `stop_split: true`.

Next selected child:

`backend.ops_governance.alerts.persistence`

## Current Residual Map

Closed children:

- `backend.ops_governance.alerts.rule_catalog`;
- `backend.ops_governance.alerts.acknowledge_flow`;
- `backend.ops_governance.alerts.trigger_engine`;
- `backend.ops_governance.alerts.predicate_checks`.

Remaining parent residuals:

- alert firing persistence helper;
- startup rule initialization bridge;
- list/read route helpers;
- route registration facade.

## Selection Gate

| Rule | Result | Evidence |
| --- | --- | --- |
| Concrete owner exists? | Yes. | `persist_alert_firing` owns alert firing disk write behavior. |
| Independent IO or state failure mode? | Yes. | Storage quota enforcement, directory creation, file path selection, and atomic JSON writing fail differently from trigger or acknowledge orchestration. |
| Parent-child communication improves? | Yes. | Both `acknowledge_flow` and `trigger_engine` currently call the parent persistence helper; extracting the helper creates one IO child while preserving parent mediation. |
| Local proof can remain focused? | Yes. | Movement can be mechanical and verified with compile, alerts handler tests, and governance gates. |
| Line count only? | No. | The split is driven by durable storage IO ownership and shared write-flow reuse. |

## Selected Boundary

`backend.ops_governance.alerts.persistence` owns only:

- storage quota call for alert firings;
- alert store directory creation;
- firing JSON file path construction;
- atomic JSON write call.

The parent bridge must remain:

- `persist_alert_firing`

Both write flows must continue through the alerts handler parent bridge.

## Forbidden Movement

BE-001NA-02 must not move:

- acknowledge route logic;
- trigger route logic;
- predicate dispatch;
- rule catalog;
- startup initialization;
- list/read route handlers;
- DTO schema owner;
- AppState fields or lock ordering;
- storage lifecycle implementation internals;
- release transition logic.

## Next Step

BE-001NA-02 backend.ops_governance.alerts.persistence baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
