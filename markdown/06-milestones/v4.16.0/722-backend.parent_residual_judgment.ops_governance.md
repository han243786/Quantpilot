# v4.16.0 backend parent residual judgment selects ops_governance

> Batch: BE-001LJ-01
> Node: `backend`
> Parent: `root`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend` residual judgment selects `backend.ops_governance` as the next top-level child.

Closed top-level children:

- `backend.interface_boundary`
- `backend.runtime`
- `backend.graph_compile`
- `backend.capability`
- `backend.strategy_config`
- `backend.storage_security`

Remaining top-level residuals:

- `backend.ops_governance`
- `backend.app_state_wiring`
- `backend.test_support`

Selected next child:

- `backend.ops_governance`

`backend.ops_governance` owns operational governance route facades for hotswap, sandbox verification, alerts, snapshots, runbook, and chaos. It should be frozen before any code movement because those branches touch runtime governance, auditability, operational controls, and test/smoke safety surfaces.

## Selection Rationale

| Candidate | Decision | Reason |
| --- | --- | --- |
| `backend.ops_governance` | SELECTED | It is the next remaining top-level backend residual after storage security closes. |
| `backend.app_state_wiring` | DEFERRED | Shared state wiring should wait until ops governance route boundaries are frozen. |
| `backend.test_support` | DEFERRED | Test support should remain after production-facing backend residuals unless a blocker requires it. |

## Boundary

**Selected child owns**:
- `src/backend/ops_governance.rs`
- `src/backend/ops_governance/alerts.rs`
- `src/backend/ops_governance/chaos.rs`
- `src/backend/ops_governance/hotswap.rs`
- `src/backend/ops_governance/runbook.rs`
- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/snapshots.rs`

**Forbidden carryover**:
- Do not move `AppState` ownership or lock order.
- Do not move test support fixtures.
- Do not move runtime/capability/storage security internals.
- Do not introduce release-transition behavior.

**Next step**:
BE-001LK-01 backend.ops_governance baseline_plan

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
