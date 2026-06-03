# v4.16.0 root.contracts.qrpc_core parent residual judgment selects event_envelope_proto

> Batch: BE-001PQ-01
> Node: `root.contracts.qrpc_core`
> Selected child: `root.contracts.qrpc_core.event_envelope_proto`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core` returns to its child queue after `contracts.qrpc_core.error_contract` closeout.

Decision:

`next_child: root.contracts.qrpc_core.event_envelope_proto`

## Closed Qrpc Core Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.error_contract` | Closed with `stop_split: true`; owns `qrpc_core/src/error.rs`. |

## Open Qrpc Core Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.event_envelope_proto` | Selected next. Owns internal event envelope proto schema. |
| `contracts.qrpc_core.plugin_contract` | Queued. Owns Rust plugin manifest/capability contract structures. |
| `contracts.qrpc_core.strategy_ir` | Queued. Owns Strategy IR structures and validation behavior. |
| `contracts.qrpc_core.protocol_primitives` | Queued. Owns primitives and version constants inside `lib.rs`. |
| `contracts.qrpc_core.runtime_protocol_config` | Queued. Owns runtime protocol config structures inside `lib.rs`. |
| `contracts.qrpc_core.artifact_specs` | Queued. Owns digest/run/backtest/artifact specs inside `lib.rs`. |
| `contracts.qrpc_core.runtime_io_contract` | Queued. Owns runtime DTO/output contracts inside `lib.rs`. |
| `contracts.qrpc_core.rfc_execution_contracts` | Queued. Owns RFC-style request/order/handoff contracts inside `lib.rs`. |

## Selection Rationale

`contracts.qrpc_core.event_envelope_proto` is selected because it is a compact schema leaf with a single physical owner:

- physical file: `qrpc_core/src/event_envelope.proto`;
- schema package: `quantpilot.events.v1`;
- message surface: `EventEnvelope`;
- enum surfaces: `ChainStage`, `Severity`, `RetentionClass`.

This selection does not own runtime event producer behavior, AsyncAPI runtime event stream schema, backend SSE handler behavior, or JSON serialization compatibility.

## Hard Boundaries

The next `root.contracts.qrpc_core.event_envelope_proto` closeout must not:

- edit `qrpc_core/src/event_envelope.proto`;
- change package name, field numbers, field names, enum values, or comments;
- change AsyncAPI, runtime event producers, backend SSE handlers, or runtime DTOs;
- change Strategy IR, plugin contracts, `lib.rs`, compiler/runtime/backend/executor behavior, or release transition.

## Next Step

BE-001PR-01 `root.contracts.qrpc_core.event_envelope_proto` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
