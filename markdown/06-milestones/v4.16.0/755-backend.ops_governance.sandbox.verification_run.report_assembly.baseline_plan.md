# v4.16.0 backend.ops_governance.sandbox.verification_run.report_assembly equivalence baseline and extraction plan

> Batch: BE-001MA-01
> Node: `backend.ops_governance.sandbox.verification_run.report_assembly`
> Parent: `backend.ops_governance.sandbox.verification_run`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.verification_run.report_assembly` is frozen as the sandbox verification report DTO assembly boundary.

Current owner file:

- `src/backend/ops_governance/sandbox/verification_run.rs`

Current embedded block:

- `SandboxVerificationReport { ... }`

BE-001MA-02 may move only this struct construction into a dedicated child module under `verification_run`.

## White-Box Boundary

The child must receive:

- `request: &RequestSandboxVerificationRequest`
- `now_ms: u64`
- `sandbox_run_id: String`
- `replay_window: ReplayWindow`
- `baseline_metrics: SandboxMetrics`
- `candidate_metrics: SandboxMetrics`
- `diffs: SandboxMetricsDiff`
- `verdict: SandboxVerdict`
- `warnings: Vec<String>`
- `fidelity: String`

The child must return:

- `SandboxVerificationReport`

## Assembly Baseline

The extracted child must preserve this field mapping exactly:

| Report field | Source |
| --- | --- |
| `proposal_id` | `request.proposal_id.clone()` |
| `sandbox_run_id` | `sandbox_run_id` |
| `replay_window` | `replay_window` |
| `baseline_metrics` | `baseline_metrics` |
| `candidate_metrics` | `candidate_metrics` |
| `diffs` | `diffs` |
| `verdict` | `verdict` |
| `warnings` | `warnings` |
| `replay_fidelity` | `fidelity` |
| `generated_at_ms` | `now_ms` |

The child must not add extra clones beyond `request.proposal_id.clone()`.

## Parent-Child Boundary

`report_assembly` may use DTO types through `crate::*`.

It must not import or call:

- `proposal_gate`;
- `replay_window`;
- `report_commit`;
- `report_api`;
- comparison metrics helpers;
- metric diff/verdict/warnings helpers;
- runtime mutation trigger;
- root compatibility bridge.

## Allowed BE-001MA-02 Movement

BE-001MA-02 may:

- create `src/backend/ops_governance/sandbox/verification_run/report_assembly.rs`;
- add `mod report_assembly;` inside `src/backend/ops_governance/sandbox/verification_run.rs`;
- replace the embedded `SandboxVerificationReport` literal with `report_assembly::build_report(...)`;
- keep `report_assembly` private to the `verification_run` parent.

BE-001MA-02 must not:

- move proposal_gate closed leaf internals;
- move replay_window closed leaf internals;
- move comparison metric computation;
- move metric diff/verdict/warnings helper ownership;
- move report_commit closed leaf internals;
- expose `report_assembly` through the sandbox parent facade;
- change DTO schema or field mapping;
- propose release transition.

## Split Decision Gate

After BE-001MA-02, BE-001MA-03 must run single-leaf closeout.

Expected decision: `stop_split: true`, because the child will own one DTO assembly literal. Continue splitting only if extraction reveals a concrete owner with independent schema or mapping behavior.

## Next Step

BE-001MA-02 backend.ops_governance.sandbox.verification_run.report_assembly extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
