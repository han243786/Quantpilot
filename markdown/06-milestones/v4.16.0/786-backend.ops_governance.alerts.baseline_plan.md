# v4.16.0 backend.ops_governance.alerts equivalence baseline and extraction plan

> Batch: BE-001MR-01
> Node: `backend.ops_governance.alerts`
> Parent: `backend.ops_governance`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.alerts` is frozen as a route facade plus root alert handler owner boundary.

Current facade:

- `src/backend/ops_governance/alerts.rs`

Current handler owner:

- `src/alert_engine.rs`

BE-001MR-02 may perform the minimum physical extraction needed to move alert handler ownership under `backend.ops_governance.alerts`, but it must preserve route behavior, alert initialization behavior, and sibling ops domains.

## Route Chain

| Layer | File | Boundary |
| --- | --- | --- |
| app router | `src/app_router.rs` | Calls `interface_boundary::register_alert_routes(router)`. |
| interface boundary | `src/backend/interface_boundary.rs` | Bridges alert routes into ops governance. |
| ops governance bridge | `src/backend/interface_boundary/ops_governance_bridge.rs` | Calls `crate::backend::ops_governance::register_alert_routes(router)`. |
| ops governance parent | `src/backend/ops_governance.rs` | Calls `alerts::register_routes(router)`. |
| alerts child facade | `src/backend/ops_governance/alerts.rs` | Delegates to `crate::alert_engine::register_alert_routes(router)`. |
| root handler owner | `src/alert_engine.rs` | Implements alert rules, firings, checks, persistence, and initialization. |

## Route Baseline

The alerts surface currently registers:

- `GET /api/v1/alerts`
- `GET /api/v1/alerts/rules`
- `POST /api/v1/alerts/:firing_id/acknowledge`
- `POST /api/v1/alerts/check`

## Handler Baseline

| Handler | Input | Output | State and behavior |
| --- | --- | --- | --- |
| `list_alerts` | `auth::UserId`, `State<AppState>` | `AlertListResponse` | Scopes `state.alert_firings` by user prefix and returns all current `state.alert_rules`. |
| `list_alert_rules` | `State<AppState>` | `Vec<AlertRule>` | Returns the current rules clone. |
| `acknowledge_alert` | `auth::UserId`, `State<AppState>`, `Path<String>`, `Json<AcknowledgeAlertRequest>` | `AlertFiring` or not_found problem JSON | Scopes by user and firing id, changes acknowledged firing to resolved on repeated acknowledgment, persists outside the write lock. |
| `trigger_alert_check` | `auth::UserId`, `State<AppState>` | `Vec<AlertFiring>` | Evaluates enabled rules, deduplicates current firing rules, persists new firings outside the write lock, auto-resolves recovered firings, and cleans resolved disk records. |

## Helper Baseline

The current root owner also contains:

- `default_alert_rules`
- `should_fire_alert`
- `check_data_freshness`
- `check_event_orphan`
- `check_risk_reject_rate`
- `check_replay_divergence`
- `check_sandbox_timeout`
- `check_storage_watermark`
- `check_approval_expiry`
- `check_ai_reject_rate`
- `check_hotswap_rollback`
- `check_capability_hash_mismatch`
- `init_alert_rules`
- `persist_alert_firing`
- `is_condition_resolved`

`init_alert_rules` is not route-facing, but it is externally called by the backend startup path. BE-001MR-02 must preserve that startup call through an equivalent root compatibility bridge or an explicitly documented parent-controlled replacement.

## Data Baseline

State owners remain unchanged:

- `AppState.alert_rules`
- `AppState.alert_firings`
- `AppState.alert_store_dir`
- `AppState.evidence_metrics`
- `AppState.sandbox_reports`
- `AppState.hotswap_records`
- `AppState.backtests`
- `AppState.approval_records`
- `AppState.ai_proposals`

DTO/schema owners remain in `src/frontend_api_types.rs`.

## Known Proof

`src/alert_engine.rs` contains direct tests for default alert rules:

- `default_alert_rules_has_ten_rules`
- `p1_rules_include_data_freshness_and_storage`
- `all_rules_have_severity_and_action`

No route smoke test was found in the current search result. BE-001MR-02 must therefore keep the movement mechanical and prove equivalence with compile, existing tests, and governance gates.

## Allowed BE-001MR-02 Movement

BE-001MR-02 may:

- move alert route handlers and helper implementation under `backend.ops_governance.alerts`;
- update the alerts route facade to call local child handlers;
- keep a root `alert_engine` compatibility bridge for startup initialization if needed;
- update module declarations only as required by the move.

BE-001MR-02 must not:

- move snapshots, runbook, chaos, hotswap, sandbox, runtime, storage security, capability, app state wiring, or test support internals;
- change route paths, HTTP methods, response status codes, problem JSON fields, alert rule names, default rule count, scoped key behavior, AppState owner, storage quota behavior, or lock order;
- introduce sibling shortcuts across ops child modules;
- propose release transition.

## Next Step

BE-001MR-02 backend.ops_governance.alerts extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test default_alert_rules_has_ten_rules p1_rules_include_data_freshness_and_storage all_rules_have_severity_and_action`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
