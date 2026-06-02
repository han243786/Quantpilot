# v4.16.0 backend.ops_governance.sandbox.verification_run.report_assembly actual extraction complete

> Batch: BE-001MA-02
> Node: `backend.ops_governance.sandbox.verification_run.report_assembly`
> Parent: `backend.ops_governance.sandbox.verification_run`
> Stage: `extract_closeout`
> Movement: Code movement completed.

---

## Summary

`backend.ops_governance.sandbox.verification_run.report_assembly` has been extracted into a private child module under the verification runner.

New owner file:

- `src/backend/ops_governance/sandbox/verification_run/report_assembly.rs`

Updated parent file:

- `src/backend/ops_governance/sandbox/verification_run.rs`

The parent runner now calls `report_assembly::build_report(...)` after metrics, diffs, verdict, warnings, and replay fidelity are computed.

## Preserved Behavior

BE-001MA-02 preserves:

- `request.proposal_id.clone()` as the only assembly clone;
- `sandbox_run_id` field mapping;
- `replay_window` field mapping;
- `baseline_metrics` and `candidate_metrics` field mapping;
- `diffs`, `verdict`, and `warnings` field mapping;
- `replay_fidelity: fidelity` field mapping;
- `generated_at_ms: now_ms` field mapping;
- parent runner commit and return semantics after report assembly.

## Parent-Child Boundary

`report_assembly` is private to `verification_run`.

It is not exposed by:

- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/sandbox/report_api.rs`
- `src/sandbox_verification.rs`

No sibling child imports were introduced.

## Non-Movement

BE-001MA-02 did not move:

- proposal_gate closed leaf internals;
- replay_window closed leaf internals;
- comparison metric computation;
- metric diff, verdict, or warning computation;
- report_commit closed leaf internals;
- route handler behavior;
- disk report loader behavior;
- runtime mutation trigger behavior;
- AppState owner or storage lifecycle owner;
- release transition policy.

## Next Step

BE-001MA-03 backend.ops_governance.sandbox.verification_run.report_assembly single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
