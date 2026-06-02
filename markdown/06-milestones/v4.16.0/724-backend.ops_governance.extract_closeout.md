# v4.16.0 backend.ops_governance facade extraction closeout

> Batch: BE-001LK-02
> Node: `backend.ops_governance`
> Parent: `backend`
> Stage: `extract_closeout`
> Movement: No code movement.

---

## Summary

BE-001LK-02 confirms the already-existing `backend.ops_governance` facade extraction.

Confirmed parent facade:

- `src/backend/ops_governance.rs` owns `MODULE_ID = "backend.ops_governance"`.
- `src/backend/ops_governance.rs` declares hotswap, sandbox, alerts, snapshots, runbook, and chaos child facades.
- `src/backend/ops_governance.rs` exposes parent bridge functions that delegate to child facades.

Confirmed child facades:

- `src/backend/ops_governance/hotswap.rs` registers hotswap routes through `crate::hotswap_api`.
- `src/backend/ops_governance/sandbox.rs` delegates sandbox verification routes to `crate::sandbox_verification`.
- `src/backend/ops_governance/alerts.rs` delegates alert routes to `crate::alert_engine`.
- `src/backend/ops_governance/snapshots.rs` delegates snapshot routes to `crate::snapshot_service`.
- `src/backend/ops_governance/runbook.rs` delegates runbook routes to `crate::runbook`.
- `src/backend/ops_governance/chaos.rs` delegates chaos routes to `crate::chaos_experiment`.

Deferred residuals:

- root handler migration remains deferred for hotswap, sandbox, alerts, snapshots, runbook, and chaos.
- `AppState` owner fields and lock order remain unchanged.
- runtime, capability, storage security, app state wiring, and test support internals remain unchanged.
- release-transition behavior remains forbidden.

## Boundary

**Real files**:
- `src/backend/ops_governance.rs`
- `src/backend/ops_governance/hotswap.rs`
- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/alerts.rs`
- `src/backend/ops_governance/snapshots.rs`
- `src/backend/ops_governance/runbook.rs`
- `src/backend/ops_governance/chaos.rs`

**Markers**:
- `BE-001LK-02`
- `actual_extraction_confirmed`
- `ops_governance_facade`
- `root_handler_migration_deferred`
- `app_state_owner_unchanged`
- `release_transition_guard`

**Next step**:
BE-001LK-03 backend.ops_governance single_leaf_closeout

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
