# v4.16.0 backend.ops_governance.runbook equivalence baseline and extraction plan

> Batch: BE-001NX-01
> Node: `backend.ops_governance.runbook`
> Parent: `backend.ops_governance`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.runbook` is frozen as the runbook route and scenario catalog owner.

BE-001NX-01 does not move code. It defines the exact baseline and allowed movement for BE-001NX-02.

## Current Owner

Current ops child facade:

- `src/backend/ops_governance/runbook.rs`

Current handler owner:

- `src/runbook.rs`

Current selected boundary:

- `register_runbook_routes`;
- `build_default_runbook`;
- `list_scenarios`;
- `get_scenario`;
- embedded runbook catalog tests.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Route facade | `GET /api/v1/runbook` still lists all default scenarios. |
| Detail route | `GET /api/v1/runbook/:scenario_id` still returns one matching scenario. |
| Not found | Missing scenario still returns `json_bad_request("not_found", ...)`. |
| Catalog size | Default runbook still contains six scenarios. |
| Catalog integrity | Each scenario still has diagnostic steps, recovery steps, and verification text. |
| Scenario identity | Scenario IDs remain unique. |
| Root compatibility | `src/runbook.rs` remains a compatibility bridge into `backend.ops_governance.runbook`. |

## Allowed BE-001NX-02 Movement

BE-001NX-02 may:

- create `src/backend/ops_governance/runbook/handlers.rs`;
- move runbook route registration, scenario catalog builder, list/detail handlers, and embedded tests into that child handler owner;
- update `src/backend/ops_governance/runbook.rs` to delegate to local handlers instead of `crate::runbook`;
- shrink `src/runbook.rs` to a root compatibility bridge that calls `backend.ops_governance.runbook::register_routes`.

## Forbidden BE-001NX-02 Movement

BE-001NX-02 must not move or rewrite:

- chaos route or handler owner;
- closed hotswap, sandbox, alerts, or snapshots internals;
- AppState owner or lock order;
- alert severity or runbook schema type definitions;
- runtime/capability/storage security internals;
- route paths or HTTP methods;
- frontend caller;
- release transition logic.

## Parent-Child Rule

The new handler owner must stay private under `backend.ops_governance.runbook`.

Allowed call paths:

- `backend.ops_governance.runbook::register_routes` -> private runbook handlers;
- `src/runbook.rs` compatibility bridge -> `backend.ops_governance.runbook::register_routes`.

Forbidden call path:

Any ops sibling or root route caller importing private runbook handlers directly.

## Proof

BE-001NX-02 must prove equivalence with:

- `cargo test -p quantpilot runbook`
- `cargo check -p quantpilot`

## Next Step

BE-001NX-02 backend.ops_governance.runbook extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
