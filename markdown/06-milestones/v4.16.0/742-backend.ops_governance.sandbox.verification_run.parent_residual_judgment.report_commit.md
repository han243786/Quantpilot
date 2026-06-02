# v4.16.0 backend.ops_governance.sandbox.verification_run parent residual judgment selects report_commit

> Batch: BE-001LT-01
> Node: `backend.ops_governance.sandbox.verification_run`
> Parent: `backend.ops_governance.sandbox`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.verification_run` returns to parent residual judgment after BE-001LS-03 confirmed `stop_split: false`.

The next child is fixed as:

`backend.ops_governance.sandbox.verification_run.report_commit`

Selection reasons:

- It owns the only durable side-effect cluster still embedded inside the runner.
- It groups sandbox report storage quota, `persist_json`, memory cache insert, and evidence metric increment.
- It has independent IO and state failure modes distinct from proposal gate, replay window, and metric/verdict helper calls.

BE-001LU-01 must establish the report_commit equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.verification_run.report_commit` | Quota check, report persistence, cache insert, evidence metric increment. | Select for next baseline. |
| `backend.ops_governance.sandbox.verification_run.report_assembly` | `SandboxVerificationReport` construction from computed values. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.verification_run.replay_window` | `QUANTPILOT_SANDBOX_REPLAY_WINDOW_DAYS` parsing and `ReplayWindow` generation. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.verification_run.proposal_gate` | `load_or_fetch_ai_proposal` and `StaticCheckPassed` gate. | Keep in parent residual queue. |

## Selected Child Boundary

`report_commit` currently contains:

- `ensure_storage_quota(Path::new("storage"), "sandbox-reports", StorageLifecycle::Transient)`
- `persist_json(&state.sandbox_report_store_dir, &report.proposal_id, &report)`
- `state.sandbox_reports.write().await.insert(request.proposal_id.clone(), report.clone())`
- `state.evidence_metrics.report_generation_count.fetch_add(1, Ordering::Relaxed)`

The child should receive the already assembled `SandboxVerificationReport` and the request proposal id. It must not own report construction or any metric helper.

## Hard Boundaries

BE-001LU-01/02 must not move:

- report_api closed leaf internals;
- runner proposal gate;
- replay window generation;
- report assembly;
- metric diff/verdict/warnings helper ownership;
- comparison metrics/proposal lookup ownership;
- disk loader ownership;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner or lock order;
- storage lifecycle owner;
- DTO schema owner;
- release transition policy.

No sibling shortcut is allowed. Report commit must live under `verification_run` and be called only by its parent runner.

## Next Step

BE-001LU-01 backend.ops_governance.sandbox.verification_run.report_commit baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
