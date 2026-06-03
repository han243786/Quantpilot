# v4.16.0 backend.ops_governance.alerts parent residual judgment selects trigger_engine

> Batch: BE-001MW-01
> Node: `backend.ops_governance.alerts`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Decision

Select the next child:

`backend.ops_governance.alerts.trigger_engine`

BE-001MW-01 is a governance-only selection batch. It returns from the closed `acknowledge_flow` leaf to the alerts parent residual queue and selects the trigger/check route engine.

## Residual Review

Closed alerts children:

- `backend.ops_governance.alerts.rule_catalog`
- `backend.ops_governance.alerts.acknowledge_flow`

Remaining alerts residuals:

| Residual | Status | Decision |
| --- | --- | --- |
| trigger engine | Owns trigger route loop, deduplication, firing creation, auto-recovery, resolved cleanup, and parent-mediated predicate/persistence calls. | Select next. |
| predicate checks | Owns metric-specific alert predicates and AppState reads. | Keep queued. |
| persistence | Owns alert firing disk write and atomic write behavior. | Keep queued. |
| startup init | Owns rule initialization bridge behavior. | Keep under alerts parent until child shape is clear. |

## Selection Rationale

`backend.ops_governance.alerts.trigger_engine` is selected because:

- it is the remaining route-facing write/read orchestration owner;
- it has a different failure mode from acknowledge flow: deduplication, firing creation, auto-recovery, and cleanup;
- it can be extracted while keeping predicate checks and persistence helper implementation parent-mediated;
- extracting it leaves the parent handler closer to a true coordinator.

## Parent-Child Contract

BE-001MX-01 must freeze trigger engine as a parent-controlled private child of the alerts handler owner boundary.

The child may own:

- `trigger_alert_check`;
- enabled-rule iteration;
- already-firing deduplication;
- firing id construction;
- new firing insertion;
- auto-recovery key collection and resolved state mutation;
- memory cleanup of resolved firings;
- disk cleanup of resolved firing files;
- call sites for parent-owned predicate and persistence helpers.

The child must not own:

- route registration outside the function reference;
- alert list or alert rule list routes;
- rule catalog;
- acknowledge flow;
- predicate helper implementations;
- persistence helper implementation;
- startup initialization bridge;
- AppState owner or lock ordering beyond preserving existing lock boundaries;
- frontend API schema types.

## Forbidden Movement

BE-001MW-01 and the next baseline must not move:

- snapshots, runbook, chaos, hotswap, or sandbox code;
- closed ops governance children;
- runtime, capability, storage security, or strategy config code;
- frontend callers or schemas;
- release transition logic.

No sibling shortcut is allowed. Predicate checks and persistence must remain mediated by the alerts parent/owner boundary until their own baselines are frozen.

## Next Step

BE-001MX-01 backend.ops_governance.alerts.trigger_engine baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
