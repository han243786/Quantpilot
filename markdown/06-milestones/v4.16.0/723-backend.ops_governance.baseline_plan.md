# v4.16.0 backend.ops_governance equivalence baseline and extraction plan

> Batch: BE-001LK-01
> Node: `backend.ops_governance`
> Parent: `backend`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance` equivalence baseline and extraction plan are frozen.

Frozen current boundary:

- `src/backend/ops_governance.rs` owns the parent ops governance facade and six route bridge functions.
- `src/backend/ops_governance/hotswap.rs` owns hotswap route facade wiring for `/api/hotswap`, `/api/hotswap/list`, and `/api/hotswap/:hotswap_id`.
- `src/backend/ops_governance/sandbox.rs` owns sandbox verification route facade wiring.
- `src/backend/ops_governance/alerts.rs` owns alert route facade wiring.
- `src/backend/ops_governance/snapshots.rs` owns snapshot route facade wiring.
- `src/backend/ops_governance/runbook.rs` owns runbook route facade wiring.
- `src/backend/ops_governance/chaos.rs` owns chaos route facade wiring.

Root handler migration remains deferred for:

- `src/hotswap_api.rs`
- `src/sandbox_verification.rs`
- `src/alert_engine.rs`
- `src/snapshot_service.rs`
- `src/runbook.rs`
- `src/chaos_experiment.rs`

Allowed BE-001LK-02 movement:

1. Confirm the already-existing `backend.ops_governance` parent facade as an extraction closeout.
2. Do not move root handler implementations yet.
3. Keep route paths, HTTP methods, AppState ownership, lock order, audit behavior, and response shapes unchanged.
4. After closeout, choose ops children only through explicit parent residual judgment and baseline gates.

Forbidden BE-001LK-02 movement:

- Do not move hotswap, sandbox, alert, snapshot, runbook, or chaos handler logic.
- Do not move `AppState` owner fields, lock order, persistence stores, approval/sandbox/report/snapshot/chaos state, or runtime governance semantics.
- Do not move runtime, capability, storage security, app state wiring, or test support internals.
- Do not introduce sibling horizontal links between ops children.
- Do not introduce release-transition behavior.

## Boundary

**Real files**:
- `src/backend/ops_governance.rs`
- `src/backend/ops_governance/hotswap.rs`
- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/alerts.rs`
- `src/backend/ops_governance/snapshots.rs`
- `src/backend/ops_governance/runbook.rs`
- `src/backend/ops_governance/chaos.rs`
- `src/hotswap_api.rs`
- `src/sandbox_verification.rs`
- `src/alert_engine.rs`
- `src/snapshot_service.rs`
- `src/runbook.rs`
- `src/chaos_experiment.rs`

**Markers**:
- `BE-001LK-01`
- `baseline_frozen`
- `ops_governance_facade`
- `hotswap_facade`
- `sandbox_facade`
- `alerts_facade`
- `snapshots_facade`
- `runbook_facade`
- `chaos_facade`
- `root_handler_migration_deferred`
- `release_transition_guard`

**Next step**:
BE-001LK-02 backend.ops_governance extract_closeout

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
