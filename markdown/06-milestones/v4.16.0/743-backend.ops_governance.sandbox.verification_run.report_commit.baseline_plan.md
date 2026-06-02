# v4.16.0 backend.ops_governance.sandbox.verification_run.report_commit equivalence baseline and extraction plan

> Batch: BE-001LU-01
> Node: `backend.ops_governance.sandbox.verification_run.report_commit`
> Parent: `backend.ops_governance.sandbox.verification_run`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.verification_run.report_commit` is frozen as the durable sandbox report commit boundary.

Current owner file:

- `src/backend/ops_governance/sandbox/verification_run.rs`

Current embedded block:

- `ensure_storage_quota`
- `persist_json`
- `state.sandbox_reports` cache insert
- `report_generation_count` evidence metric increment

BE-001LU-02 may move only this block into a dedicated child module under `verification_run`.

## White-Box Boundary

The child must receive:

- `state: &AppState`
- `request: &RequestSandboxVerificationRequest`
- `report: &SandboxVerificationReport`

The child must not own:

- proposal lookup or status gate;
- replay window generation;
- comparison metric computation;
- metric diff, verdict, or warning computation;
- `SandboxVerificationReport` assembly;
- route handler behavior;
- disk report loader behavior;
- runtime mutation trigger behavior.

## Commit Baseline

The extracted child must preserve this sequence exactly:

1. Check storage quota with:
   - root path: `Path::new("storage")`
   - layer: `"sandbox-reports"`
   - lifecycle: `StorageLifecycle::Transient`
2. On quota failure, return `Err(io_error(e))`.
3. Persist the report with `persist_json(&state.sandbox_report_store_dir, &report.proposal_id, report)`.
4. Map persistence failure with `.map_err(io_error)?`.
5. Acquire `state.sandbox_reports.write().await`.
6. Insert under `request.proposal_id.clone()`.
7. Store `report.clone()` as the cached value.
8. Increment `state.evidence_metrics.report_generation_count` with `Ordering::Relaxed`.
9. Return `Ok(())` to the parent runner.

The parent runner must continue returning `Ok(report)` after the child commit succeeds.

## Allowed BE-001LU-02 Movement

BE-001LU-02 may:

- create `src/backend/ops_governance/sandbox/verification_run/report_commit.rs`;
- add `mod report_commit;` inside `src/backend/ops_governance/sandbox/verification_run.rs`;
- replace the embedded commit block with `report_commit::commit_report(state, request, &report).await?;`;
- keep `report_commit` private to the `verification_run` parent.

BE-001LU-02 must not:

- expose `report_commit` through the sandbox parent facade;
- import `report_api` or any sibling child;
- change request/report DTO schema;
- change cache key from `request.proposal_id.clone()`;
- change persisted report id from `report.proposal_id`;
- change lock order;
- change evidence metric semantics;
- change storage lifecycle policy;
- propose release transition.

## Split Decision Gate

After BE-001LU-02, BE-001LU-03 must run single-leaf closeout.

Expected decision: `stop_split: true`, unless the extraction reveals a concrete owner with independent IO, state, or failure semantics. Line count alone is not a valid split trigger.

## Next Step

BE-001LU-02 backend.ops_governance.sandbox.verification_run.report_commit extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
