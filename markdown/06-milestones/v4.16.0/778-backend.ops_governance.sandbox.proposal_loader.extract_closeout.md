# v4.16.0 backend.ops_governance.sandbox.proposal_loader actual extraction complete

> Batch: BE-001MM-02
> Node: `backend.ops_governance.sandbox.proposal_loader`
> Parent: `backend.ops_governance.sandbox`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

`backend.ops_governance.sandbox.proposal_loader` has been physically extracted.

New child file:

- `src/backend/ops_governance/sandbox/proposal_loader.rs`

Moved helper:

- `load_or_fetch_ai_proposal`

## Equivalence Notes

The extraction preserves the BE-001MM-01 baseline:

- in-memory `state.ai_proposals` lookup still runs before disk fallback;
- clone-on-hit semantics are unchanged;
- fallback still calls `load_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), proposal_id).await`;
- return type remains `Result<RuntimeAiProposalRecord, (StatusCode, String)>`;
- `verification_run.proposal_gate` still receives the helper through the sandbox parent boundary.

## Parent Wiring

`src/backend/ops_governance/sandbox.rs` now owns:

- `mod proposal_loader;`
- `use proposal_loader::load_or_fetch_ai_proposal;`

`verification_run` was not rewired to import the child directly.

## Unmoved Boundaries

This batch did not move:

- `load_sandbox_report_from_disk`;
- report_api closed leaf internals;
- verification_run closed parent internals;
- metrics_evaluation closed leaf internals;
- comparison_metrics closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner;
- release transition policy.

## Next Step

BE-001MM-03 backend.ops_governance.sandbox.proposal_loader single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
