# v4.16.0 backend.ops_governance.sandbox parent residual judgment selects proposal_loader

> Batch: BE-001ML-01
> Node: `backend.ops_governance.sandbox`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox` returns to parent residual judgment after `comparison_metrics` closed as a completed parent.

The next child is fixed as:

`backend.ops_governance.sandbox.proposal_loader`

Selection reasons:

- It owns the memory-first AI proposal lookup used by sandbox verification.
- It owns the disk fallback through `load_runtime_ai_proposal_record`.
- It is independent from sandbox report disk loading.
- It can be extracted while keeping verification_run connected only through the sandbox parent boundary.

BE-001MM-01 must establish the proposal_loader equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.proposal_loader` | `state.ai_proposals` memory-first read and `ai_proposal_store_dir` disk fallback. | Select for next baseline. |
| `backend.ops_governance.sandbox.report_disk_loader` | Sandbox report proposal id validation, disk JSON read, and JSON parse error mapping. | Keep in parent residual queue. |

Closed children:

- `backend.ops_governance.sandbox.report_api`
- `backend.ops_governance.sandbox.verification_run`
- `backend.ops_governance.sandbox.metrics_evaluation`
- `backend.ops_governance.sandbox.comparison_metrics`

## Selected Child Boundary

`proposal_loader` currently contains:

- `load_or_fetch_ai_proposal`;
- `state.ai_proposals.read().await.get(proposal_id).cloned()`;
- fallback to `load_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), proposal_id).await`.

The child should return `Result<RuntimeAiProposalRecord, (StatusCode, String)>`.

## Hard Boundaries

BE-001MM-01/02 must not move:

- sandbox report disk loader;
- report_api closed leaf internals;
- verification_run closed parent internals;
- metrics_evaluation closed leaf internals;
- comparison_metrics closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner;
- release transition policy.

No sibling shortcut is allowed. The selected child must live under `sandbox` and be called only by its parent.

## Next Step

BE-001MM-01 backend.ops_governance.sandbox.proposal_loader baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
