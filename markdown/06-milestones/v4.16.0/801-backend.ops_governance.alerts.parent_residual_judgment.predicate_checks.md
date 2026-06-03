# v4.16.0 backend.ops_governance.alerts parent residual judgment selects predicate_checks

> Batch: BE-001MY-01
> Node: `backend.ops_governance.alerts`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Decision

Select the next child:

`backend.ops_governance.alerts.predicate_checks`

BE-001MY-01 is a governance-only selection batch. It returns from the closed `trigger_engine` leaf to the alerts parent residual queue and selects the predicate evaluation owner.

## Residual Review

Closed alerts children:

- `backend.ops_governance.alerts.rule_catalog`
- `backend.ops_governance.alerts.acknowledge_flow`
- `backend.ops_governance.alerts.trigger_engine`

Remaining alerts residuals:

| Residual | Status | Decision |
| --- | --- | --- |
| predicate checks | Owns alert rule dispatch plus metric-specific AppState reads. | Select next. |
| persistence | Owns alert firing disk write and atomic write behavior. | Keep queued. |
| startup init | Owns rule initialization bridge behavior. | Keep under alerts parent until child shape is clear. |

## Selection Rationale

`backend.ops_governance.alerts.predicate_checks` is selected because:

- it is now the largest remaining non-persistence behavior in the alerts parent;
- it owns a clear read-only evaluation boundary across evidence metrics, sandbox reports, hotswap records, backtests, approvals, and AI proposals;
- trigger engine already treats predicate evaluation as parent-mediated behavior, so this child can be inserted behind a parent bridge without adding sibling shortcuts;
- extracting it leaves persistence and startup init as the remaining alerts residuals.

## Parent-Child Contract

BE-001MZ-01 must freeze predicate checks as a parent-controlled private child of the alerts handler owner boundary.

The child may own:

- alert rule dispatch by `rule.rule_name`;
- metric-specific predicate helpers;
- AppState reads required by those helpers;
- environment parsing for sandbox timeout and storage watermark checks;
- storage lifecycle size probe used by the storage watermark predicate.

The parent must keep a bridge function that trigger engine can call. The bridge may delegate to the private predicate child, preserving parent-child mediation.

The child must not own:

- trigger route orchestration;
- new firing creation or recovery mutation;
- alert firing persistence;
- rule catalog;
- acknowledge flow;
- startup initialization bridge;
- AppState owner or lock ordering beyond preserving existing read locks;
- frontend API schema types.

## Forbidden Movement

BE-001MY-01 and the next baseline must not move:

- snapshots, runbook, chaos, hotswap, or sandbox code;
- closed ops governance children;
- runtime, capability, storage security, or strategy config code;
- frontend callers or schemas;
- release transition logic.

No sibling shortcut is allowed. Trigger engine must continue to call the alerts parent bridge, not the predicate child directly.

## Next Step

BE-001MZ-01 backend.ops_governance.alerts.predicate_checks baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
