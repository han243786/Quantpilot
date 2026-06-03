# v4.16.0 backend.ops_governance.alerts.predicate_checks extraction closeout

> Batch: BE-001MZ-02
> Node: `backend.ops_governance.alerts.predicate_checks`
> Parent: `backend.ops_governance.alerts`
> Stage: `extract_closeout`
> Movement: Rule dispatch and metric-specific predicate checks moved into a private child module.

---

## Summary

`backend.ops_governance.alerts.predicate_checks` is extracted into `src/backend/ops_governance/alerts/handlers/predicate_checks.rs`.

The alerts handler owner keeps the parent bridge:

- `should_fire_alert`

The child owns:

- alert rule dispatch by `rule.rule_name`;
- `check_data_freshness`;
- `check_event_orphan`;
- `check_risk_reject_rate`;
- `check_replay_divergence`;
- `check_sandbox_timeout`;
- `check_hotswap_rollback`;
- `check_capability_hash_mismatch`;
- `check_storage_watermark`;
- `check_approval_expiry`;
- `check_ai_reject_rate`.

## Boundary Result

| Surface | Result |
| --- | --- |
| Parent route owner | `src/backend/ops_governance/alerts/handlers.rs` still owns route registration, list routes, startup init, persistence helper, and recovery mediation. |
| Parent bridge | `should_fire_alert` remains parent-owned and delegates to the child. |
| Trigger engine | `trigger_engine` still calls the parent bridge; it does not directly call the predicate child. |
| Predicate child | The child owns rule-name dispatch and metric-specific AppState read checks. |
| Persistence | Alert firing persistence remains parent-owned and unchanged. |
| Startup | Alert rule initialization remains parent-owned and unchanged. |
| Release transition | No release-transition shortcut or sibling direct connection was introduced. |

## Equivalence Proof

The extraction is mechanical:

- all rule names map to the same predicate functions as BE-001MZ-01;
- unknown rule names still return `false`;
- AppState read locks and atomic metric reads stay in the same predicate bodies;
- environment variable defaults are unchanged;
- storage watermark cleanup and size comparison are unchanged;
- `is_condition_resolved` still negates the parent bridge.

## Next Step

BE-001MZ-03 backend.ops_governance.alerts.predicate_checks single_leaf_closeout

## Gates

- `cargo fmt`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `git diff --check`
- `cargo fmt --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
