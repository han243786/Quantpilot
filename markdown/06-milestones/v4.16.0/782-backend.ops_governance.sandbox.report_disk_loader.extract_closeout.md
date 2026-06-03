# v4.16.0 backend.ops_governance.sandbox.report_disk_loader actual extraction complete

> Batch: BE-001MO-02
> Node: `backend.ops_governance.sandbox.report_disk_loader`
> Parent: `backend.ops_governance.sandbox`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

`backend.ops_governance.sandbox.report_disk_loader` has been physically extracted.

New child file:

- `src/backend/ops_governance/sandbox/report_disk_loader.rs`

Moved helper:

- `load_sandbox_report_from_disk`

## Equivalence Notes

The extraction preserves the BE-001MO-01 baseline:

- proposal id guard still runs before path construction and disk read;
- report path construction still appends `.json` to the proposal id under the caller-provided store directory;
- unreadable files still map to the existing not_found JSON bad request;
- parse errors still map through `internal_error(anyhow::anyhow!("{}", error))`;
- root compatibility bridge still receives the helper through the sandbox parent boundary.

## Parent Wiring

`src/backend/ops_governance/sandbox.rs` now owns:

- `mod report_disk_loader;`
- `pub(crate) use report_disk_loader::load_sandbox_report_from_disk;`

`report_api`, root compatibility bridge, and runtime mutation callers were not rewired to import the child directly.

## Drained Legacy File

`src/backend/ops_governance/sandbox/handlers.rs` remains as a drained historical shell so existing documentation path references stay valid. It no longer owns concrete sandbox helper logic.

## Unmoved Boundaries

This batch did not move:

- proposal_loader closed leaf internals;
- report_api closed leaf internals;
- verification_run closed parent internals;
- metrics_evaluation closed leaf internals;
- comparison_metrics closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner;
- release transition policy.

## Next Step

BE-001MO-03 backend.ops_governance.sandbox.report_disk_loader single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
