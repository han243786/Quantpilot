# v4.16.0 backend.ops_governance.alerts.recovery_bridge extraction closeout

> Batch: BE-001NE-02
> Node: `backend.ops_governance.alerts.recovery_bridge`
> Parent: `backend.ops_governance.alerts`
> Stage: `extract_closeout`
> Movement: Alert recovery condition bridge moved into a private child module.

---

## Summary

`backend.ops_governance.alerts.recovery_bridge` is extracted into `src/backend/ops_governance/alerts/handlers/recovery_bridge.rs`.

The alerts handler owner keeps the parent bridge:

- `is_condition_resolved`

The child owns:

- recovery condition negation;
- parent-mediated call to `should_fire_alert`.

## Boundary Result

| Surface | Result |
| --- | --- |
| Parent recovery bridge | `is_condition_resolved` remains parent-owned and callable by trigger_engine. |
| Predicate bridge | `should_fire_alert` remains parent-owned and still delegates to predicate_checks. |
| Recovery child | The child calls parent-owned `should_fire_alert` and negates it. |
| Predicate child | Not imported or called directly by recovery_bridge. |
| Trigger engine | Still imports and calls parent-owned `is_condition_resolved`. |
| Release transition | No release-transition shortcut or sibling direct connection was introduced. |

## Equivalence Proof

The extraction is mechanical:

- recovery input remains `&AppState`, `&auth::UserId`, and `&AlertRule`;
- recovery output remains `bool`;
- recovery still means the alert trigger condition is no longer true;
- implementation still returns `!should_fire_alert(state, user_id, rule).await`;
- trigger_engine call sites are unchanged.

## Next Step

BE-001NE-03 backend.ops_governance.alerts.recovery_bridge single_leaf_closeout

## Gates

- `cargo fmt`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `git diff --check`
- `cargo fmt --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
