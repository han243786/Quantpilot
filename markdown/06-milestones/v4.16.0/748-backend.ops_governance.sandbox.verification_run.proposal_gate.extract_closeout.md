# v4.16.0 backend.ops_governance.sandbox.verification_run.proposal_gate actual extraction complete

> Batch: BE-001LW-02
> Node: `backend.ops_governance.sandbox.verification_run.proposal_gate`
> Parent: `backend.ops_governance.sandbox.verification_run`
> Stage: `extract_closeout`
> Movement: Code movement completed.

---

## Summary

`backend.ops_governance.sandbox.verification_run.proposal_gate` has been extracted into a private child module under the verification runner.

New owner file:

- `src/backend/ops_governance/sandbox/verification_run/proposal_gate.rs`

Updated parent file:

- `src/backend/ops_governance/sandbox/verification_run.rs`

The parent runner now calls `proposal_gate::load_eligible_proposal(state, request).await?` before replay window generation and comparison metric computation.

## Preserved Behavior

BE-001LW-02 preserves:

- proposal loading through `load_or_fetch_ai_proposal(state, &request.proposal_id).await?`;
- `RuntimeAiProposalStatus::StaticCheckPassed` eligibility requirement;
- `SANDBOX_VERIFICATION_DENIED` error code;
- `沙箱验证要求 AI 提案已通过静态检查` error message;
- return of `RuntimeAiProposalRecord` to the parent runner;
- parent runner use of `compute_comparison_metrics(state, &ai_proposal).await?`.

## Parent-Child Boundary

`proposal_gate` is private to `verification_run`.

It uses the verification_run parent-controlled `load_or_fetch_ai_proposal` bridge and is not exposed by:

- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/sandbox/report_api.rs`
- `src/sandbox_verification.rs`

No sibling child imports were introduced.

## Non-Movement

BE-001LW-02 did not move:

- replay window generation;
- comparison metric computation;
- metric diff, verdict, or warning computation;
- `SandboxVerificationReport` assembly;
- report_commit closed leaf internals;
- route handler behavior;
- disk report loader behavior;
- runtime mutation trigger behavior;
- AppState owner or storage lifecycle owner;
- release transition policy.

## Next Step

BE-001LW-03 backend.ops_governance.sandbox.verification_run.proposal_gate single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
