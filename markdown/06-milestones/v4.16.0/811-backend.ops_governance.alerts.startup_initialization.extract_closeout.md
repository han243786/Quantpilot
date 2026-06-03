# v4.16.0 backend.ops_governance.alerts.startup_initialization extraction closeout

> Batch: BE-001NB-03
> Node: `backend.ops_governance.alerts.startup_initialization`
> Parent: `backend.ops_governance.alerts`
> Stage: `extract_closeout`
> Movement: Alert startup rule seeding moved into a private child module.

---

## Summary

`backend.ops_governance.alerts.startup_initialization` is extracted into `src/backend/ops_governance/alerts/handlers/startup_initialization.rs`.

The alerts handler owner keeps the parent bridge:

- `init_alert_rules`

The child owns:

- alert rules write-lock acquisition;
- empty-store check;
- assignment of default alert rules when the store is empty.

## Boundary Result

| Surface | Result |
| --- | --- |
| Parent startup bridge | `init_alert_rules(state)` remains parent-owned and callable by the alerts root. |
| Default rules mediation | Parent passes `rule_catalog::default_alert_rules` into the child. |
| Sibling coupling | Startup child does not import or call the rule_catalog sibling directly. |
| Startup child | Owns only the write-lock and empty-store seeding transaction. |
| Routes and schema | Not moved. |
| Release transition | No release-transition shortcut or sibling direct connection was introduced. |

## Equivalence Proof

The extraction is mechanical:

- alert rules write lock remains `state.alert_rules.write().await`;
- default rules are still assigned only when the rules collection is empty;
- default rules still come from `rule_catalog::default_alert_rules`;
- parent bridge still exposes `init_alert_rules(state)`;
- route registration, list/read routes, trigger flow, acknowledge flow, predicate checks, and persistence are unchanged.

## Next Step

BE-001NB-04 backend.ops_governance.alerts.startup_initialization single_leaf_closeout

## Gates

- `cargo fmt`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `git diff --check`
- `cargo fmt --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
