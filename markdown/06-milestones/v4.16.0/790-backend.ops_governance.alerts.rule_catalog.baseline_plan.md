# v4.16.0 backend.ops_governance.alerts.rule_catalog equivalence baseline and extraction plan

> Batch: BE-001MT-01
> Node: `backend.ops_governance.alerts.rule_catalog`
> Parent: `backend.ops_governance.alerts`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts.rule_catalog` is frozen as the static default alert rule catalog child.

BE-001MT-01 does not move code. It defines the exact equivalence baseline for the next extraction batch.

## Current Owner

Current implementation is still in `src/backend/ops_governance/alerts/handlers.rs`.

The child boundary is:

- `default_alert_rules`;
- the ten default rule records returned by that function;
- the three direct catalog invariant tests:
  - `default_alert_rules_has_ten_rules`;
  - `p1_rules_include_data_freshness_and_storage`;
  - `all_rules_have_severity_and_action`.

## Frozen Semantics

The next extraction must preserve:

| Surface | Frozen behavior |
| --- | --- |
| Count | `default_alert_rules()` returns exactly 10 rules. |
| Rule identity | Existing `rule_name` values and uniqueness expectations are unchanged. |
| P1 membership | `data_freshness_critical` and `storage_watermark_critical` remain P1 rules. |
| Rule fields | Each rule keeps its current description, trigger condition, severity, action, enabled flag, and resolve condition. |
| Action invariant | Every default rule has a non-empty action. |
| Initialization contract | Startup init only installs the defaults when `state.alert_rules` is empty. |
| Route contract | `/api/v1/alerts/rules` still reads from `state.alert_rules`; it does not call the default catalog directly. |

## Allowed BE-001MT-02 Movement

BE-001MT-02 may:

- create a private child module for the rule catalog under the alerts handler owner boundary;
- move `default_alert_rules` into that child;
- move the three direct catalog tests with it;
- update the alerts handler owner to call the child through its parent-controlled private module.

## Forbidden BE-001MT-02 Movement

BE-001MT-02 must not move or rewrite:

- route registration;
- `list_alerts` or `list_alert_rules`;
- acknowledge flow;
- trigger engine;
- predicate checks;
- alert firing persistence;
- startup compatibility bridge;
- AppState fields or lock ordering;
- frontend API schema types;
- snapshots, runbook, chaos, hotswap, or sandbox modules;
- release transition logic.

## Split Gate

This is a valid child because:

- concrete owner exists: static rule catalog;
- local proof exists: three direct unit tests;
- parent-child communication improves: init can call a narrow catalog child instead of keeping the static list inside the broad handler file;
- no release transition or sibling shortcut is introduced.

Further splitting inside `rule_catalog` is not pre-approved. After BE-001MT-02, the child must go through single-leaf closeout before any deeper split is considered.

## Next Step

BE-001MT-02 backend.ops_governance.alerts.rule_catalog extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
