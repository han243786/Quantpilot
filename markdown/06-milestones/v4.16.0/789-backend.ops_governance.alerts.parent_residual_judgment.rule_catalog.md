# v4.16.0 backend.ops_governance.alerts parent residual judgment selects rule_catalog

> Batch: BE-001MS-01
> Node: `backend.ops_governance.alerts`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Decision

Select the next child:

`backend.ops_governance.alerts.rule_catalog`

BE-001MS-01 is a governance-only selection batch. It freezes the next alerts child boundary and moves the recursive cursor to the rule catalog baseline batch.

## Residual Review

`backend.ops_governance.alerts` remains open after BE-001MR-03 because the extracted handler owner still contains several concrete responsibilities:

| Residual | Status | Decision |
| --- | --- | --- |
| rule catalog | Contains `default_alert_rules` and direct rule invariant tests. | Select first. |
| acknowledge flow | Owns acknowledge state transition, not_found mapping, and persistence call. | Keep queued. |
| trigger engine | Owns trigger iteration, deduplication, predicate dispatch, auto-recovery, and cleanup. | Keep queued. |
| predicate checks | Owns metric-specific alert predicates and AppState reads. | Keep queued. |
| persistence | Owns alert firing disk write and atomic write behavior. | Keep queued. |
| startup init | Owns rule initialization bridge behavior. | Keep under alerts parent until child shape is clear. |

## Selection Rationale

`backend.ops_governance.alerts.rule_catalog` is selected because:

- it is a pure catalog owner with no disk IO, route response mapping, or AppState mutation;
- it already has direct unit tests in the alerts handler test module;
- extracting it first reduces the largest handler file without touching route lifecycle or alert firing semantics;
- it gives the next batch a small, easily verified equivalence baseline.

## Parent-Child Contract

BE-001MT-01 must freeze the rule catalog as a parent-controlled child of `backend.ops_governance.alerts`.

The child may own:

- the static default alert rule list;
- alert rule ids, names, severity, threshold, duration, condition, enabled flag, and action list defaults;
- existing unit tests that prove the catalog count, id uniqueness, and required rule ids.

The child must not own:

- route registration;
- list or acknowledge route response mapping;
- trigger engine iteration or deduplication;
- predicate checks;
- alert firing persistence;
- startup compatibility bridge;
- AppState owner or lock ordering;
- DTO/schema owner in `src/frontend_api_types.rs`.

## Forbidden Movement

BE-001MS-01 and the next baseline must not move:

- snapshots, runbook, chaos, hotswap, or sandbox code;
- closed ops governance children;
- runtime, capability, storage security, or strategy config code;
- frontend callers or schemas;
- release transition logic.

No sibling shortcut is allowed. Any future rule catalog implementation must be mediated by the alerts parent/owner boundary, not by horizontal links between independent child leaves.

## Next Step

BE-001MT-01 backend.ops_governance.alerts.rule_catalog baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers::tests`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
