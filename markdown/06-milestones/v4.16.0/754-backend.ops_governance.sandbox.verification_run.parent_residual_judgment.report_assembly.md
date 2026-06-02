# v4.16.0 backend.ops_governance.sandbox.verification_run parent residual judgment selects report_assembly

> Batch: BE-001LZ-01
> Node: `backend.ops_governance.sandbox.verification_run`
> Parent: `backend.ops_governance.sandbox`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.verification_run` returns to parent residual judgment after `replay_window` closed with `stop_split: true`.

The next child is fixed as:

`backend.ops_governance.sandbox.verification_run.report_assembly`

Selection reasons:

- It owns the final `SandboxVerificationReport` DTO construction.
- It is independent from proposal eligibility, replay window shaping, and durable report commit.
- It groups all generated values into the report schema without owning metric computation.
- It is the last concrete child candidate before the parent runner can be closed.

BE-001MA-01 must establish the report_assembly equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.verification_run.report_assembly` | `SandboxVerificationReport` construction from request, replay window, metrics, diffs, verdict, warnings, fidelity, and timestamp. | Select for next baseline. |

Closed children:

- `backend.ops_governance.sandbox.verification_run.report_commit`
- `backend.ops_governance.sandbox.verification_run.proposal_gate`
- `backend.ops_governance.sandbox.verification_run.replay_window`

## Selected Child Boundary

`report_assembly` currently contains:

- `proposal_id: request.proposal_id.clone()`
- `sandbox_run_id`
- `replay_window`
- `baseline_metrics`
- `candidate_metrics`
- `diffs`
- `verdict`
- `warnings`
- `replay_fidelity: fidelity`
- `generated_at_ms: now_ms`

The child should receive already computed values and return `SandboxVerificationReport`.

## Hard Boundaries

BE-001MA-01/02 must not move:

- proposal_gate closed leaf internals;
- replay_window closed leaf internals;
- comparison metrics;
- metric diff/verdict/warnings helper ownership;
- report_commit closed leaf internals;
- report_api closed leaf internals;
- disk loader ownership;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner or storage lifecycle owner;
- release transition policy.

No sibling shortcut is allowed. Report assembly must live under `verification_run` and be called only by its parent runner.

## Next Step

BE-001MA-01 backend.ops_governance.sandbox.verification_run.report_assembly baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
