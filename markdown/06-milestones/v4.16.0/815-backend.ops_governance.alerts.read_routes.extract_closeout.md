# v4.16.0 backend.ops_governance.alerts.read_routes extraction closeout

> Batch: BE-001NC-03
> Node: `backend.ops_governance.alerts.read_routes`
> Parent: `backend.ops_governance.alerts`
> Stage: `extract_closeout`
> Movement: Alert read-only route handlers moved into a private child module.

---

## Summary

`backend.ops_governance.alerts.read_routes` is extracted into `src/backend/ops_governance/alerts/handlers/read_routes.rs`.

The alerts handler owner keeps the parent route facade:

- `register_alert_routes`

The child owns:

- `list_alerts`;
- `list_alert_rules`;
- user-scoped alert firing projection;
- alert rules read projection;
- `AlertListResponse` assembly.

## Boundary Result

| Surface | Result |
| --- | --- |
| Parent route facade | `register_alert_routes` remains parent-owned. |
| Alert list route | Parent route facade points to `read_routes::list_alerts`. |
| Alert rules route | Parent route facade points to `read_routes::list_alert_rules`. |
| Read child | The child owns only read-only handler projections. |
| Write/trigger flows | Not moved. |
| Routes and schema | No route path or DTO schema changed. |
| Release transition | No release-transition shortcut or sibling direct connection was introduced. |

## Equivalence Proof

The extraction is mechanical:

- `list_alerts` still accepts `auth::UserId` and `State<AppState>`;
- user prefix still comes from `auth::scoped_key(&user_id, "")`;
- alert firings still use the same read lock, prefix filter, and cloned values;
- alert rules are still cloned from `state.alert_rules.read().await`;
- `list_alert_rules` still returns `Json<Vec<AlertRule>>`;
- route paths remain `/api/v1/alerts` and `/api/v1/alerts/rules`.

## Next Step

BE-001NC-04 backend.ops_governance.alerts.read_routes single_leaf_closeout

## Gates

- `cargo fmt`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `git diff --check`
- `cargo fmt --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
