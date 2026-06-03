# v4.16.0 backend.ops_governance.alerts.recovery_bridge equivalence baseline and extraction plan

> Batch: BE-001NE-01
> Node: `backend.ops_governance.alerts.recovery_bridge`
> Parent: `backend.ops_governance.alerts`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts.recovery_bridge` is frozen as the alert auto-recovery predicate bridge child.

BE-001NE-01 does not move code. It defines the exact baseline and allowed movement for BE-001NE-02.

## Current Owner

Current implementation is still in `src/backend/ops_governance/alerts/handlers.rs`.

The child boundary is:

- `is_condition_resolved`

The parent bridge must remain:

- `is_condition_resolved`

`should_fire_alert` must remain parent-owned and continue to delegate to `predicate_checks`.

## Frozen Semantics

The next extraction must preserve:

| Surface | Frozen behavior |
| --- | --- |
| Recovery bridge input | Still accepts `&AppState`, `&auth::UserId`, and `&AlertRule`. |
| Recovery bridge output | Still returns `bool`. |
| Recovery semantics | Recovery still means the alert trigger condition is no longer true. |
| Predicate call | Still evaluates `should_fire_alert(state, user_id, rule).await`. |
| Negation | Still returns `!should_fire_alert(...).await`. |
| Trigger engine import | Trigger engine still imports and calls the parent `is_condition_resolved` bridge. |
| Predicate ownership | `should_fire_alert` and predicate dispatch remain outside the recovery child. |

## Allowed BE-001NE-02 Movement

BE-001NE-02 may:

- create a private child module for recovery bridge under the alerts handler owner boundary;
- move only the implementation body of `is_condition_resolved` into that child;
- keep a parent bridge named `is_condition_resolved`;
- let the child call the parent-owned `should_fire_alert` bridge instead of importing predicate_checks directly.

## Forbidden BE-001NE-02 Movement

BE-001NE-02 must not move or rewrite:

- `should_fire_alert`;
- predicate dispatch;
- trigger route logic;
- persistence implementation;
- route registration;
- read route handlers;
- rule catalog implementation;
- startup initialization;
- DTO schema owner;
- AppState fields or lock ordering;
- release transition logic.

## Proof

No direct recovery bridge unit test is currently isolated for alerts. BE-001NE-02 must therefore keep the movement mechanical and prove equivalence with compile, existing alerts tests, and governance gates.

## Next Step

BE-001NE-02 backend.ops_governance.alerts.recovery_bridge extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
