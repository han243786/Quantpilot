# v4.16.0 root.contracts parent residual judgment selects api_surface

> Batch: BE-001PF-01
> Node: `root.contracts`
> Selected child: `root.contracts.api_surface`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts` returns to the L2 child queue created by BE-001PE-01.

Decision:

`next_child: root.contracts.api_surface`

## Closed Contracts Children

No `root.contracts` children are closed yet.

## Open Contracts Residuals

| Residual | Status |
| --- | --- |
| `contracts.api_surface` | Selected next. Smallest schema-only surface. |
| `contracts.qrpc_core` | Queued. Runtime protocol structs and artifact/version constants. |
| `contracts.core_ir` | Queued. Core IR and v4 graph/backtest artifact contracts. |
| `contracts.compiler_bridge` | Queued. Validation/lowering bridge into Core IR. |
| `contracts.runtime_support` | Queued. Runtime support crate; must not steal backend or executor state. |
| `contracts.quantscript` | Queued. Formal QS parser/HIR/lowering/audit contract. |
| `contracts.plugin_metadata` | Queued. Plugin manifest and registry placeholder contract. |
| `root.executor` | Queued outside contracts. |

## Selection Rationale

`contracts.api_surface` is selected first because it is a compact, schema-only leaf:

- `contracts/openapi/root.yaml` owns the HTTP API contract surface;
- `contracts/asyncapi/runtime-events.yaml` owns the runtime event stream contract surface;
- no Rust source movement is needed;
- no schema semantics are changed in this selection step;
- the next closeout can decide whether this leaf should stop splitting or whether OpenAPI and AsyncAPI deserve separate children.

## Hard Boundaries

The next `root.contracts.api_surface` closeout must not:

- edit OpenAPI or AsyncAPI schema fields, paths, channels, examples, tags, or versions;
- change backend route handlers or runtime event producers;
- change QRPC/Core IR/compiler/runtime/QS behavior;
- move `qrpc_session` or executor session ownership;
- start frontend extraction, E2E cleanup, or test asset retirement;
- introduce release transition sibling links.

## Next Step

BE-001PG-01 `root.contracts.api_surface` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
