# v4.16.0 backend.ops_governance.sandbox.proposal_loader equivalence baseline and extraction plan

> Batch: BE-001MM-01
> Node: `backend.ops_governance.sandbox.proposal_loader`
> Parent: `backend.ops_governance.sandbox`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

This batch freezes the equivalence baseline for `backend.ops_governance.sandbox.proposal_loader`.

The planned child is not created in this batch.

Planned child file:

- `src/backend/ops_governance/sandbox/proposal_loader.rs`

Current owner location:

- `src/backend/ops_governance/sandbox/handlers.rs`

## White Box Boundary

`proposal_loader` owns one public-to-parent helper:

| Method | Input | Output | Caller | Invariant |
| --- | --- | --- | --- | --- |
| `load_or_fetch_ai_proposal` | `&AppState`, `proposal_id: &str` | `Result<RuntimeAiProposalRecord, (StatusCode, String)>` | `backend.ops_governance.sandbox.verification_run.proposal_gate` through the sandbox parent | Must keep memory-first lookup before disk fallback. |

Internal steps frozen by this baseline:

1. Read `state.ai_proposals`.
2. Look up `proposal_id`.
3. Clone and return the in-memory record when found.
4. Fall back to `load_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), proposal_id).await`.
5. Preserve the fallback loader's existing error mapping.

## Equivalence Contract

BE-001MM-02 is equivalent only if:

- the in-memory record still wins over disk;
- the fallback still uses `state.ai_proposal_store_dir.as_ref()`;
- the function keeps the same async signature and return type;
- `verification_run` continues to receive the helper through the sandbox parent boundary;
- no report disk loading logic moves with this child.

## Extraction Plan

BE-001MM-02 may:

- create `src/backend/ops_governance/sandbox/proposal_loader.rs`;
- move `load_or_fetch_ai_proposal` into that child;
- add `mod proposal_loader;` to `src/backend/ops_governance/sandbox.rs`;
- update the sandbox parent to import `proposal_loader::load_or_fetch_ai_proposal`;
- leave `load_sandbox_report_from_disk` in `handlers.rs` for the later `report_disk_loader` decision.

BE-001MM-02 must not:

- change the function body beyond mechanical module relocation;
- move `load_sandbox_report_from_disk`;
- change proposal id validation semantics;
- expose the child directly through the root compatibility bridge;
- alter `verification_run` closed children;
- start release transition behavior.

## Hard Boundaries

No sibling shortcut is allowed.

`proposal_loader` may only be called by `backend.ops_governance.sandbox` parent wiring. `verification_run.proposal_gate` must continue to depend on the parent-provided helper, not import this child directly.

## Next Step

BE-001MM-02 backend.ops_governance.sandbox.proposal_loader extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
