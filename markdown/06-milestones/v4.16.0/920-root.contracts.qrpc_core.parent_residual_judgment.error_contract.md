# v4.16.0 root.contracts.qrpc_core parent residual judgment selects error_contract

> Batch: BE-001PO-01
> Node: `root.contracts.qrpc_core`
> Selected child: `root.contracts.qrpc_core.error_contract`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core` returns to the child queue created by BE-001PN-01.

Decision:

`next_child: root.contracts.qrpc_core.error_contract`

## Closed Qrpc Core Children

No `root.contracts.qrpc_core` children are closed yet.

## Open Qrpc Core Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.error_contract` | Selected next. Owns the typed core error enum and conversion/display behavior. |
| `contracts.qrpc_core.event_envelope_proto` | Queued. Owns internal event envelope proto schema. |
| `contracts.qrpc_core.plugin_contract` | Queued. Owns Rust plugin manifest/capability contract structures. |
| `contracts.qrpc_core.strategy_ir` | Queued. Owns Strategy IR structures and validation behavior. |
| `contracts.qrpc_core.protocol_primitives` | Queued. Owns primitives and version constants inside `lib.rs`. |
| `contracts.qrpc_core.runtime_protocol_config` | Queued. Owns runtime protocol config structures inside `lib.rs`. |
| `contracts.qrpc_core.artifact_specs` | Queued. Owns digest/run/backtest/artifact specs inside `lib.rs`. |
| `contracts.qrpc_core.runtime_io_contract` | Queued. Owns runtime DTO/output contracts inside `lib.rs`. |
| `contracts.qrpc_core.rfc_execution_contracts` | Queued. Owns RFC-style request/order/handoff contracts inside `lib.rs`. |

## Selection Rationale

`contracts.qrpc_core.error_contract` is selected first because it is the smallest qrpc_core child and has a compact, independent physical owner:

- physical file: `qrpc_core/src/error.rs`;
- public enum: `QuantPilotError`;
- behavior: `Display`, `std::error::Error::source`, and `From<std::io::Error>`;
- no dependency on Strategy IR, plugin manifests, event proto, runtime protocol DTOs, or artifact specs.

## Hard Boundaries

The next `root.contracts.qrpc_core.error_contract` closeout must not:

- edit `qrpc_core/src/error.rs`;
- change error variants, display strings, source behavior, or IO conversion;
- change `qrpc_core/src/lib.rs`, `strategy_ir.rs`, `plugin.rs`, or `event_envelope.proto`;
- change compiler, runtime, backend, executor, frontend, E2E, or release transition behavior.

## Next Step

BE-001PP-01 `root.contracts.qrpc_core.error_contract` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
