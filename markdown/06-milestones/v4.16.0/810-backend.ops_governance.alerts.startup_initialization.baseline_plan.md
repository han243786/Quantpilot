# v4.16.0 backend.ops_governance.alerts.startup_initialization equivalence baseline and extraction plan

> Batch: BE-001NB-02
> Node: `backend.ops_governance.alerts.startup_initialization`
> Parent: `backend.ops_governance.alerts`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts.startup_initialization` is frozen as the alert rule startup seeding child.

BE-001NB-02 does not move code. It defines the exact baseline and allowed movement for BE-001NB-03.

## Current Owner

Current implementation is still in `src/backend/ops_governance/alerts/handlers.rs`.

The child boundary is:

- `init_alert_rules`

The parent bridge must remain:

- `init_alert_rules`

The child must not directly call the `rule_catalog` sibling. The alerts parent must mediate default rule access.

## Frozen Semantics

The next extraction must preserve:

| Surface | Frozen behavior |
| --- | --- |
| Public owner bridge | `alerts::handlers::init_alert_rules(state)` remains callable by the alerts root. |
| Locking | Acquires `state.alert_rules.write().await`. |
| Idempotence | Seeds rules only when the current rules collection is empty. |
| Default source | Uses `rule_catalog::default_alert_rules()` as the default rule source. |
| Assignment | Replaces the empty rules collection with the default alert rules. |
| Sibling access | Startup child does not directly import or call rule_catalog; parent mediates the default-rule provider. |
| Route behavior | Alert routes and handlers are unchanged. |

## Allowed BE-001NB-03 Movement

BE-001NB-03 may:

- create a private child module for startup initialization under the alerts handler owner boundary;
- move only the alert rules write-lock and empty-store seeding implementation into that child;
- keep a parent bridge named `init_alert_rules`;
- pass the default rule provider from the parent bridge into the child to avoid direct sibling coupling.

## Forbidden BE-001NB-03 Movement

BE-001NB-03 must not move or rewrite:

- rule catalog implementation;
- acknowledge route logic;
- trigger route logic;
- predicate dispatch;
- persistence implementation;
- list/read route handlers;
- route registration;
- DTO schema owner;
- AppState fields beyond preserving the current alert rules write lock;
- release transition logic.

## Proof

No direct startup initialization unit test is currently isolated for alerts. BE-001NB-03 must therefore keep the movement mechanical and prove equivalence with compile, existing alerts tests, and governance gates.

## Next Step

BE-001NB-03 backend.ops_governance.alerts.startup_initialization extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
