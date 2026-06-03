# v4.16.0 root parent residual judgment selects contracts

> Batch: BE-001PD-01
> Node: `root`
> Selected child: `root.contracts`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root` returns to the remaining Rust-facing top-level residuals after `backend` closed.

Decision:

`next_child: root.contracts`

## Closed Root Children

Already closed in the current recursive scope:

- `root.system`;
- `root.backend`.

## Open Root Residuals

| Residual | Status |
| --- | --- |
| `root.contracts` | Selected next. Top-level statistics identify it as a white-box coverage gap. |
| `root.executor` | Queued. Executor state ownership remains delayed. |

## Selection Rationale

`root.contracts` is selected because it is a Rust/protocol parent area with an explicit module-tree gap:

- `contracts/` owns OpenAPI and AsyncAPI physical contracts;
- `qrpc_core/` owns QRPC protocol structures, errors, plugin metadata, Strategy IR, and event proto;
- `qrpc_core_ir/` owns Core IR structures;
- `qrpc_compiler/` owns compilation bridge behavior;
- `qrpc_runtime/` owns runtime support libraries used by backend and executor-facing flows;
- `quantscript/` owns QS syntax, HIR, lowering, diagnostics, static audit, and authoring samples.

This step selects the parent only. It does not alter schema, protocol data structures, compiler behavior, runtime behavior, or QS semantics.

## Hard Boundaries

The next `root.contracts` baseline must not:

- change OpenAPI, AsyncAPI, QRPC, Core IR, or QS schema semantics;
- change compiler/runtime behavior;
- move executor state ownership;
- start frontend extraction or E2E cleanup;
- introduce release transition sibling links.

## Next Step

BE-001PE-01 `root.contracts` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
