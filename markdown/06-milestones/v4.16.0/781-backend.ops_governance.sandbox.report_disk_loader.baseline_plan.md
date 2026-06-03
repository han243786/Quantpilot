# v4.16.0 backend.ops_governance.sandbox.report_disk_loader equivalence baseline and extraction plan

> Batch: BE-001MO-01
> Node: `backend.ops_governance.sandbox.report_disk_loader`
> Parent: `backend.ops_governance.sandbox`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

This batch freezes the equivalence baseline for `backend.ops_governance.sandbox.report_disk_loader`.

The planned child is not created in this batch.

Planned child name:

- `report_disk_loader`

Current owner location:

- `src/backend/ops_governance/sandbox/handlers.rs`

## White Box Boundary

`report_disk_loader` owns one parent-facing helper:

| Method | Input | Output | Caller | Invariant |
| --- | --- | --- | --- | --- |
| `load_sandbox_report_from_disk` | `&FsPath`, `proposal_id: &str` | `Result<SandboxVerificationReport, (StatusCode, String)>` | `report_api` and root compatibility bridge through the sandbox parent | Must keep path guard before file path construction and disk read. |

Internal steps frozen by this baseline:

1. Reject proposal ids containing `..`.
2. Reject proposal ids containing `/`.
3. Reject proposal ids containing `\`.
4. Reject empty proposal ids.
5. Reject proposal ids longer than 128 characters.
6. Build the report path by appending `.json` to the proposal id under the provided store directory.
7. Read the JSON bytes with `fs::read`.
8. Map missing/unreadable files to the existing `json_bad_request("not_found", ...)` response.
9. Parse with `serde_json::from_slice`.
10. Map parse errors with `internal_error(anyhow::anyhow!("{}", error))`.

## Equivalence Contract

BE-001MO-02 is equivalent only if:

- the guard order and conditions are unchanged;
- file path construction stays under the caller-provided `store_dir`;
- unreadable files keep the same not_found mapping;
- parse errors keep the same internal error mapping;
- the function keeps the same async signature and return type;
- the root compatibility bridge continues to receive the helper through the sandbox parent boundary.

## Extraction Plan

BE-001MO-02 may:

- create a `report_disk_loader` child file under the sandbox source directory;
- move `load_sandbox_report_from_disk` into that child;
- add `mod report_disk_loader;` to the sandbox parent;
- update the sandbox parent to re-export `report_disk_loader::load_sandbox_report_from_disk`.

BE-001MO-02 must not:

- move `proposal_loader`;
- change the error strings or status codes;
- change `report_api` handler behavior;
- change the root compatibility bridge API;
- alter runtime mutation callers;
- start release transition behavior.

## Hard Boundaries

No sibling shortcut is allowed.

`report_disk_loader` may only be surfaced through `backend.ops_governance.sandbox` parent wiring. `report_api`, root compatibility bridge, and runtime mutation callers must not import this child directly.

## Next Step

BE-001MO-02 backend.ops_governance.sandbox.report_disk_loader extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
