# v4.16.0 backend.ops_governance.runbook.route_facade equivalence baseline and extraction plan

> Batch: BE-001OD-01
> Node: `backend.ops_governance.runbook.route_facade`
> Parent: `backend.ops_governance.runbook`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.runbook.route_facade` is frozen as the runbook route registration owner.

BE-001OD-01 does not move code. It defines the exact baseline and allowed movement for BE-001OD-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/runbook/handlers.rs`

Current selected boundary:

- `register_runbook_routes`;
- `/api/v1/runbook` route path;
- `/api/v1/runbook/:scenario_id` route path;
- binding route paths to runbook read handlers.

The parent must retain bridges for:

- list handler dispatch;
- detail handler dispatch;
- default catalog construction.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Route paths | `/api/v1/runbook` and `/api/v1/runbook/:scenario_id` remain unchanged. |
| List route binding | List route still resolves to runbook list handler behavior. |
| Detail route binding | Detail route still resolves to runbook detail handler behavior. |
| Catalog bridge | Read handlers still consume the catalog through the parent bridge. |
| External API | `backend::ops_governance::runbook::register_routes` remains the parent-facing entrypoint. |

## Allowed BE-001OD-02 Movement

BE-001OD-02 may:

- create `src/backend/ops_governance/runbook/handlers/route_facade.rs`;
- move only `register_runbook_routes` into that private child module;
- add a private `mod route_facade;` declaration in `src/backend/ops_governance/runbook/handlers.rs`;
- keep a parent bridge named `register_runbook_routes` delegating to the route facade child;
- add parent-owned async bridge handlers that delegate to the closed read_routes child;
- bind route facade paths to parent-owned bridge handlers.

## Forbidden BE-001OD-02 Movement

BE-001OD-02 must not move or rewrite:

- closed scenario catalog internals;
- closed read route internals;
- root compatibility bridge;
- chaos route or handler owner;
- closed hotswap, sandbox, alerts, or snapshots internals;
- AppState owner or lock order;
- schema type definitions, frontend caller, or release transition logic.

## Parent-Child Rule

The child must stay private under the current runbook handler implementation owner.

Allowed call paths:

- runbook parent bridge -> private `handlers::route_facade::register_runbook_routes`;
- route facade child -> parent-owned read handler bridge functions;
- parent-owned read handler bridge functions -> closed private `handlers::read_routes::*`;
- read handlers -> parent `build_default_runbook` bridge;
- parent `build_default_runbook` bridge -> closed private `handlers::scenario_catalog::build_default_runbook`.

Forbidden call path:

Any route facade child function importing or calling `handlers::read_routes` directly.

## Proof

BE-001OD-02 must prove equivalence with:

- `cargo test -p quantpilot runbook`
- `cargo check -p quantpilot`

## Next Step

BE-001OD-02 backend.ops_governance.runbook.route_facade extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
