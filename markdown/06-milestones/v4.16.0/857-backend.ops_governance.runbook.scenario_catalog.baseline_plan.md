# v4.16.0 backend.ops_governance.runbook.scenario_catalog equivalence baseline and extraction plan

> Batch: BE-001NZ-01
> Node: `backend.ops_governance.runbook.scenario_catalog`
> Parent: `backend.ops_governance.runbook`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.runbook.scenario_catalog` is frozen as the default runbook scenario catalog owner.

BE-001NZ-01 does not move code. It defines the exact baseline and allowed movement for BE-001NZ-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/runbook/handlers.rs`

Current selected boundary:

- `build_default_runbook`;
- six default scenario definitions;
- `default_runbook_has_six_scenarios`;
- `each_scenario_has_diagnostic_and_recovery_steps`;
- `all_scenario_ids_are_unique`.

The parent bridge must remain:

- `build_default_runbook`.

Read handlers must continue to call the parent bridge, not the child directly.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Catalog size | Default runbook still contains six scenarios. |
| Scenario IDs | Scenario IDs remain unchanged and unique. |
| Scenario contents | Diagnostic steps, recovery steps, severity, verification text, and API call hints remain unchanged. |
| List route data source | `list_scenarios` still returns the default catalog through the parent bridge. |
| Detail route data source | `get_scenario` still searches the default catalog through the parent bridge. |
| Tests | Catalog integrity tests still prove size, nonempty diagnostic/recovery/verification fields, and unique IDs. |

## Allowed BE-001NZ-02 Movement

BE-001NZ-02 may:

- create `src/backend/ops_governance/runbook/handlers/scenario_catalog.rs`;
- move only `build_default_runbook` and its catalog integrity tests into that private child module;
- add a private `mod scenario_catalog;` declaration in `src/backend/ops_governance/runbook/handlers.rs`;
- keep a parent bridge named `build_default_runbook` with the same signature that delegates to the child;
- keep list/detail route handlers parent-mediated.

## Forbidden BE-001NZ-02 Movement

BE-001NZ-02 must not move or rewrite:

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

## Parent-Child Rule

The child must stay private under the current runbook handler implementation owner.

Allowed call paths:

- runbook handler parent bridge -> private `handlers::scenario_catalog::build_default_runbook`;
- runbook read handlers -> runbook handler parent bridge.

Forbidden call path:

Any runbook read handler importing or calling `handlers::scenario_catalog` directly.

## Proof

BE-001NZ-02 must prove equivalence with:

- `cargo test -p quantpilot runbook`
- `cargo check -p quantpilot`

## Next Step

BE-001NZ-02 backend.ops_governance.runbook.scenario_catalog extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
