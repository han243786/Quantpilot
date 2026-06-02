# v4.16.0 backend.ops_governance.sandbox.verification_run parent residual judgment selects proposal_gate

> Batch: BE-001LV-01
> Node: `backend.ops_governance.sandbox.verification_run`
> Parent: `backend.ops_governance.sandbox`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.verification_run` returns to parent residual judgment after `report_commit` closed with `stop_split: true`.

The next child is fixed as:

`backend.ops_governance.sandbox.verification_run.proposal_gate`

Selection reasons:

- It owns the first independent failure boundary inside the runner.
- It combines proposal load/fetch and `StaticCheckPassed` eligibility enforcement.
- It returns the proposal object needed by comparison metrics while keeping denial semantics isolated.
- It is more concrete than pure report assembly and has stronger failure semantics than replay window shaping.

BE-001LW-01 must establish the proposal_gate equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.verification_run.proposal_gate` | Proposal load/fetch and `StaticCheckPassed` denial. | Select for next baseline. |
| `backend.ops_governance.sandbox.verification_run.replay_window` | `now_ms`, sandbox run id, env replay-days parsing, `ReplayWindow` generation. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.verification_run.report_assembly` | `SandboxVerificationReport` construction from computed values. | Keep in parent residual queue. |

## Selected Child Boundary

`proposal_gate` currently contains:

- `load_or_fetch_ai_proposal(state, &request.proposal_id).await?`
- `RuntimeAiProposalStatus::StaticCheckPassed` eligibility check
- `json_bad_request("SANDBOX_VERIFICATION_DENIED", "沙箱验证要求 AI 提案已通过静态检查")`
- successful return of the loaded proposal to the parent runner

The child should receive `state` and `request` or `proposal_id`, and return the loaded proposal after the eligibility gate passes.

## Hard Boundaries

BE-001LW-01/02 must not move:

- replay window generation;
- comparison metrics;
- metric diff/verdict/warnings helper ownership;
- report assembly;
- report_commit closed leaf internals;
- report_api closed leaf internals;
- disk loader ownership;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner or storage lifecycle owner;
- release transition policy.

No sibling shortcut is allowed. Proposal gate must live under `verification_run` and be called only by its parent runner.

## Next Step

BE-001LW-01 backend.ops_governance.sandbox.verification_run.proposal_gate baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
