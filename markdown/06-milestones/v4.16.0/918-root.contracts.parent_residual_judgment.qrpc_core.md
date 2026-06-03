# v4.16.0 root.contracts parent residual judgment selects qrpc_core

> Batch: BE-001PM-01
> Node: `root.contracts`
> Selected child: `root.contracts.qrpc_core`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts` returns to its L2 residual queue after `contracts.api_surface` parent closeout.

Decision:

`next_child: root.contracts.qrpc_core`

## Closed Contracts Children

| Child | Result |
| --- | --- |
| `contracts.api_surface` | Closed. OpenAPI HTTP and AsyncAPI runtime event schemas are separated and closed. |

## Open Contracts Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core` | Selected next. Runtime protocol structs, artifact/version constants, digest helpers, Strategy IR, plugin metadata contract, event envelope proto, and typed core errors. |
| `contracts.core_ir` | Queued. Core IR and v4 graph/backtest artifact contracts. |
| `contracts.compiler_bridge` | Queued. Validation/lowering bridge into Core IR. |
| `contracts.runtime_support` | Queued. Runtime support crate; must not steal backend or executor state ownership. |
| `contracts.quantscript` | Queued. Formal QS parser/HIR/lowering/audit contract. |
| `contracts.plugin_metadata` | Queued. Physical plugin registry placeholders and any plugin contract residue after qrpc_core baseline. |
| `root.executor` | Queued outside contracts. |

## Selection Rationale

`contracts.qrpc_core` is selected because it is the next broad Rust-facing protocol contract owner:

- `qrpc_core/src/lib.rs` owns runtime protocol data structures, artifact/version constants, digest helpers, run/backtest specs, runtime outputs, execution/order/allocation contracts, and portfolio/event contracts;
- `qrpc_core/src/strategy_ir.rs` owns legacy Strategy IR contract structures and validation behavior;
- `qrpc_core/src/plugin.rs` owns plugin manifest, capability contract, extension point, security, dependency, and registry data contracts;
- `qrpc_core/src/error.rs` owns typed core error variants;
- `qrpc_core/src/event_envelope.proto` owns the internal event envelope protobuf schema.

This step selects the parent only. It does not alter protocol structs, validation semantics, digest behavior, plugin contracts, event proto, compiler/runtime behavior, backend behavior, or schema files.

## Hard Boundaries

The next `root.contracts.qrpc_core` baseline must not:

- edit `qrpc_core/src/*`;
- change public struct/enum fields, serde shape, version constants, validation behavior, digest behavior, or proto schema;
- move plugin registry placeholders from `plugins/*`;
- change Core IR, compiler, runtime support, QuantScript, backend, executor, frontend, or E2E behavior;
- introduce release transition sibling links.

## Next Step

BE-001PN-01 `root.contracts.qrpc_core` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
