# v4.16.0 backend.ops_governance.runbook parent residual judgment selects read_routes

> Batch: BE-001OA-01
> Node: `backend.ops_governance.runbook`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.runbook` returns to parent residual judgment after `scenario_catalog` closed as a stable static catalog child.

The next child is fixed as:

`backend.ops_governance.runbook.read_routes`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.runbook.read_routes` | `list_scenarios`, `get_scenario`, default catalog bridge consumption, and not_found response behavior. | Select for next baseline. |
| `backend.ops_governance.runbook.route_facade` | Route registration for list/detail runbook endpoints. | Keep in parent residual queue until read routes are separated. |

## Selected Child Boundary

`backend.ops_governance.runbook.read_routes` currently contains:

- `list_scenarios`;
- `get_scenario`;
- parent-mediated consumption of `build_default_runbook`;
- not_found error code and message construction for missing scenarios.

The selected child must continue to consume the scenario catalog through a parent-owned bridge. It must not import the closed `scenario_catalog` child directly.

## Hard Boundaries

BE-001OB-01/02 must not move:

- closed scenario catalog internals;
- route facade registration;
- root compatibility bridge;
- chaos route or handler owner;
- closed hotswap, sandbox, alerts, or snapshots internals;
- AppState owner or lock order;
- schema type definitions, frontend caller, or release transition logic.

No sibling shortcut is allowed.

## Next Step

BE-001OB-01 backend.ops_governance.runbook.read_routes baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
