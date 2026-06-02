# v4.16.0 backend.ops_governance.sandbox.verification_run.proposal_gate equivalence baseline and extraction plan

> Batch: BE-001LW-01
> Node: `backend.ops_governance.sandbox.verification_run.proposal_gate`
> Parent: `backend.ops_governance.sandbox.verification_run`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.verification_run.proposal_gate` is frozen as the sandbox verification proposal eligibility boundary.

Current owner file:

- `src/backend/ops_governance/sandbox/verification_run.rs`

Current embedded block:

- `load_or_fetch_ai_proposal(state, &request.proposal_id).await?`
- `RuntimeAiProposalStatus::StaticCheckPassed` eligibility gate
- `SANDBOX_VERIFICATION_DENIED` bad-request response when the proposal is not eligible

BE-001LW-02 may move only this block into a dedicated child module under `verification_run`.

## White-Box Boundary

The child must receive:

- `state: &AppState`
- `request: &RequestSandboxVerificationRequest`

The child must return:

- `Result<RuntimeAiProposalRecord, (StatusCode, String)>`

The parent runner must continue passing the returned proposal to `compute_comparison_metrics(state, &ai_proposal).await?`.

## Gate Baseline

The extracted child must preserve this sequence exactly:

1. Load the proposal through `load_or_fetch_ai_proposal(state, &request.proposal_id).await?`.
2. Check `ai_proposal.status != RuntimeAiProposalStatus::StaticCheckPassed`.
3. On mismatch, return `json_bad_request("SANDBOX_VERIFICATION_DENIED", "沙箱验证要求 AI 提案已通过静态检查")`.
4. On success, return `Ok(ai_proposal)`.

The child must preserve both the machine code and the human-readable error text.

## Parent-Child Boundary

`proposal_gate` may call the verification_run parent-controlled `load_or_fetch_ai_proposal` bridge.

It must not import or call:

- `report_commit`;
- `report_api`;
- sandbox disk report loader;
- runtime mutation trigger;
- root compatibility bridge.

## Allowed BE-001LW-02 Movement

BE-001LW-02 may:

- create `src/backend/ops_governance/sandbox/verification_run/proposal_gate.rs`;
- add `mod proposal_gate;` inside `src/backend/ops_governance/sandbox/verification_run.rs`;
- replace the embedded gate block with `proposal_gate::load_eligible_proposal(state, request).await?;`;
- keep `proposal_gate` private to the `verification_run` parent.

BE-001LW-02 must not:

- move replay window generation;
- move comparison metric computation;
- move metric diff/verdict/warnings helper ownership;
- move `SandboxVerificationReport` assembly;
- move report_commit closed leaf internals;
- expose `proposal_gate` through the sandbox parent facade;
- change proposal status eligibility;
- change error code or error message;
- propose release transition.

## Split Decision Gate

After BE-001LW-02, BE-001LW-03 must run single-leaf closeout.

Expected decision: `stop_split: true`, because the child will own one compact eligibility gate. Continue splitting only if extraction reveals another concrete owner with independent failure semantics.

## Next Step

BE-001LW-02 backend.ops_governance.sandbox.verification_run.proposal_gate extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
