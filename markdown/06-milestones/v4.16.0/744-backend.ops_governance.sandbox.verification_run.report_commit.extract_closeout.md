# v4.16.0 backend.ops_governance.sandbox.verification_run.report_commit actual extraction complete

> Batch: BE-001LU-02
> Node: `backend.ops_governance.sandbox.verification_run.report_commit`
> Parent: `backend.ops_governance.sandbox.verification_run`
> Stage: `extract_closeout`
> Movement: Code movement completed.

---

## Summary

`backend.ops_governance.sandbox.verification_run.report_commit` has been extracted into a private child module under the verification runner.

New owner file:

- `src/backend/ops_governance/sandbox/verification_run/report_commit.rs`

Updated parent file:

- `src/backend/ops_governance/sandbox/verification_run.rs`

The parent runner now assembles `SandboxVerificationReport`, calls `report_commit::commit_report(state, request, &report).await?`, and then returns `Ok(report)`.

## Preserved Behavior

BE-001LU-02 preserves:

- `ensure_storage_quota(Path::new("storage"), "sandbox-reports", StorageLifecycle::Transient)`;
- quota failure mapping through `io_error`;
- `persist_json(&state.sandbox_report_store_dir, &report.proposal_id, report)`;
- persistence failure mapping through `io_error`;
- `state.sandbox_reports.write().await.insert(request.proposal_id.clone(), report.clone())`;
- `state.evidence_metrics.report_generation_count.fetch_add(1, Ordering::Relaxed)`;
- parent runner return semantics after the commit succeeds.

## Parent-Child Boundary

`report_commit` is private to `verification_run`.

It is not exposed by:

- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/sandbox/report_api.rs`
- `src/sandbox_verification.rs`

No sibling child imports were introduced.

## Non-Movement

BE-001LU-02 did not move:

- proposal lookup or `StaticCheckPassed` gate;
- replay window generation;
- comparison metric computation;
- metric diff, verdict, or warning computation;
- `SandboxVerificationReport` assembly;
- route handler behavior;
- disk report loader behavior;
- runtime mutation trigger behavior;
- AppState owner or storage lifecycle owner;
- release transition policy.

## Next Step

BE-001LU-03 backend.ops_governance.sandbox.verification_run.report_commit single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
