# v4.16.0 backend.ops_governance.runbook parent residual judgment selects scenario_catalog

> Batch: BE-001NY-01
> Node: `backend.ops_governance.runbook`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.runbook` returns to parent residual judgment after the first runbook handler extraction stayed open for internal split.

The next child is fixed as:

`backend.ops_governance.runbook.scenario_catalog`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.runbook.scenario_catalog` | Default runbook scenario catalog builder and catalog integrity tests. | Select for next baseline. |
| `backend.ops_governance.runbook.read_routes` | List/detail handlers that consume the default catalog. | Keep in parent residual queue. |
| `backend.ops_governance.runbook.route_facade` | Route registration for list/detail runbook routes. | Keep as parent-owned route surface for now. |

## Selected Child Boundary

`backend.ops_governance.runbook.scenario_catalog` currently contains:

- `build_default_runbook`;
- six default scenario definitions;
- catalog size test;
- diagnostic/recovery/verification integrity test;
- unique scenario ID test.

## Hard Boundaries

BE-001NZ-01/02 must not move:

- list/detail route handlers;
- runbook route facade;
- root compatibility bridge;
- chaos route or handler owner;
- closed hotswap, sandbox, alerts, or snapshots internals;
- AppState owner or lock order;
- alert severity or runbook schema type definitions;
- runtime/capability/storage security internals;
- frontend caller;
- release transition logic.

No sibling shortcut is allowed. Read handlers must consume the catalog through the runbook parent bridge until their own baseline changes ownership.

## Next Step

BE-001NZ-01 backend.ops_governance.runbook.scenario_catalog baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
