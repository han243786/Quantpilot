# v4.16.0 backend.ops_governance.alerts.read_routes equivalence baseline and extraction plan

> Batch: BE-001NC-02
> Node: `backend.ops_governance.alerts.read_routes`
> Parent: `backend.ops_governance.alerts`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts.read_routes` is frozen as the alert read-only route handler child.

BE-001NC-02 does not move code. It defines the exact baseline and allowed movement for BE-001NC-03.

## Current Owner

Current implementation is still in `src/backend/ops_governance/alerts/handlers.rs`.

The child boundary is:

- `list_alerts`;
- `list_alert_rules`.

The alerts parent route registration may call the child handlers directly.

## Frozen Semantics

The next extraction must preserve:

| Surface | Frozen behavior |
| --- | --- |
| Alert list route handler input | Still accepts `auth::UserId` and `State<AppState>`. |
| Alert list response | Still returns `Result<Json<AlertListResponse>, (StatusCode, String)>`. |
| User scope | Still uses `auth::scoped_key(&user_id, "")`. |
| Firing read lock | Still reads `state.alert_firings.read().await`. |
| Firing filter | Still keeps only keys starting with the scoped prefix. |
| Firing projection | Still clones alert firing values into a `Vec<AlertFiring>`. |
| Rules projection in list route | Still clones `state.alert_rules.read().await`. |
| Alert rules route handler input | Still accepts `State<AppState>`. |
| Alert rules route response | Still returns `Result<Json<Vec<AlertRule>>, (StatusCode, String)>`. |
| Route facade | Route registration remains parent-owned. |

## Allowed BE-001NC-03 Movement

BE-001NC-03 may:

- create a private child module for alert read routes under the alerts handler owner boundary;
- move only `list_alerts` and `list_alert_rules` into that child;
- route from the alerts parent directly to the child handler functions.

## Forbidden BE-001NC-03 Movement

BE-001NC-03 must not move or rewrite:

- route registration facade;
- rule catalog implementation;
- acknowledge route logic;
- trigger route logic;
- predicate dispatch;
- persistence implementation;
- startup initialization;
- alert recovery predicate bridge;
- DTO schema owner;
- AppState fields or lock ordering beyond preserving the current read locks;
- release transition logic.

## Proof

No direct read-route unit test is currently isolated for alerts. BE-001NC-03 must therefore keep the movement mechanical and prove equivalence with compile, existing alerts tests, and governance gates.

## Next Step

BE-001NC-03 backend.ops_governance.alerts.read_routes extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
