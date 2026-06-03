# v4.16.0 backend.ops_governance.runbook.scenario_catalog actual extraction complete

> Batch: BE-001NZ-02
> Node: `backend.ops_governance.runbook.scenario_catalog`
> Parent: `backend.ops_governance.runbook`
> Stage: `extract_closeout`
> Movement: Default runbook scenario catalog moved into a private child module.

---

## Summary

`backend.ops_governance.runbook.scenario_catalog` now owns default runbook construction and catalog integrity tests.

The runbook handler parent still owns the bridge helper name used by read handlers, so list/detail routes remain parent-mediated.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/runbook/handlers.rs` | `src/backend/ops_governance/runbook/handlers/scenario_catalog.rs` | `build_default_runbook` and catalog integrity tests moved. |
| `src/backend/ops_governance/runbook/handlers.rs` | `src/backend/ops_governance/runbook/handlers.rs` | Parent declares the private child module and keeps same-name bridge helper delegating to the child. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| Catalog size | Default runbook still contains six scenarios. |
| Scenario IDs | Scenario IDs remain unchanged and unique. |
| Scenario contents | Diagnostic steps, recovery steps, severity, verification text, and API call hints remain unchanged. |
| List route data source | `list_scenarios` still returns the default catalog through the parent bridge. |
| Detail route data source | `get_scenario` still searches the default catalog through the parent bridge. |
| Tests | Catalog integrity tests moved with the catalog owner. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- runbook handler parent bridge -> private `handlers::scenario_catalog::build_default_runbook`;
- runbook read handlers -> runbook handler parent bridge.

The following remain outside this child:

- list/detail route handlers;
- runbook route facade;
- root compatibility bridge;
- chaos route and handler owner;
- closed hotswap, sandbox, alerts, and snapshots internals;
- AppState owner or lock order;
- schema type definitions, frontend caller, and release transition logic.

## Proof

- `cargo check -p quantpilot`

## Next Step

BE-001NZ-03 backend.ops_governance.runbook.scenario_catalog single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
