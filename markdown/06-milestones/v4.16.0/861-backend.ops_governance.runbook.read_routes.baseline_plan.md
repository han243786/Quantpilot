# v4.16.0 backend.ops_governance.runbook.read_routes equivalence baseline and extraction plan

> Batch: BE-001OB-01
> Node: `backend.ops_governance.runbook.read_routes`
> Parent: `backend.ops_governance.runbook`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.runbook.read_routes` is frozen as the runbook list/detail read handler owner.

BE-001OB-01 does not move code. It defines the exact baseline and allowed movement for BE-001OB-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/runbook/handlers.rs`

Current selected boundary:

- `list_scenarios`;
- `get_scenario`;
- parent-mediated `build_default_runbook` consumption;
- missing-scenario `not_found` error construction.

The parent bridge must remain:

- `build_default_runbook`.

Route registration must continue to be owned by the runbook handler parent for this batch.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| List route handler | Returns the full default runbook catalog. |
| Detail route handler | Returns the matching scenario when `scenario_id` exists. |
| Missing detail route | Returns `json_bad_request("not_found", ...)` for an unknown scenario ID. |
| Catalog access | Read handlers consume the default catalog through the parent bridge. |
| Route facade | `register_runbook_routes` still registers `/api/v1/runbook` and `/api/v1/runbook/:scenario_id`. |

## Allowed BE-001OB-02 Movement

BE-001OB-02 may:

- create `src/backend/ops_governance/runbook/handlers/read_routes.rs`;
- move only `list_scenarios` and `get_scenario` into that private child module;
- add a private `mod read_routes;` declaration in `src/backend/ops_governance/runbook/handlers.rs`;
- update route facade registration to call `read_routes::list_scenarios` and `read_routes::get_scenario`;
- keep `build_default_runbook` in the parent as the only bridge to the closed scenario catalog child;
- add local read-route equivalence tests if useful.

## Forbidden BE-001OB-02 Movement

BE-001OB-02 must not move or rewrite:

- closed scenario catalog internals;
- runbook route facade into its own child;
- root compatibility bridge;
- chaos route or handler owner;
- closed hotswap, sandbox, alerts, or snapshots internals;
- AppState owner or lock order;
- schema type definitions, frontend caller, or release transition logic.

## Parent-Child Rule

The child must stay private under the current runbook handler implementation owner.

Allowed call paths:

- runbook route facade -> private `handlers::read_routes::*`;
- runbook read handlers -> parent `build_default_runbook` bridge;
- parent `build_default_runbook` bridge -> closed private `handlers::scenario_catalog::build_default_runbook`.

Forbidden call path:

Any read handler importing or calling `handlers::scenario_catalog` directly.

## Proof

BE-001OB-02 must prove equivalence with:

- `cargo test -p quantpilot runbook`
- `cargo check -p quantpilot`

## Next Step

BE-001OB-02 backend.ops_governance.runbook.read_routes extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
