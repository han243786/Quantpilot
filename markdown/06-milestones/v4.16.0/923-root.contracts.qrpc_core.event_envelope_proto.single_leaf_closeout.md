# v4.16.0 root.contracts.qrpc_core.event_envelope_proto single leaf closeout

> Batch: BE-001PR-01
> Node: `root.contracts.qrpc_core.event_envelope_proto`
> Parent: `root.contracts.qrpc_core`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.event_envelope_proto` has been evaluated as the internal event envelope proto schema owner selected by BE-001PQ-01.

Decision:

`stop_split: true`

The node remains equivalent because `qrpc_core/src/event_envelope.proto` was not edited.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Independent physical owner | Stop. The internal event envelope schema is already one proto file. |
| Public/schema surface | Covered. The leaf owns package `quantpilot.events.v1`, `EventEnvelope`, `ChainStage`, `Severity`, and `RetentionClass`. |
| Separate message/enum families | Not enough to split. Message and enums form one protobuf schema contract with shared field numbers and compatibility rules. |
| Producer/consumer behavior | Outside this leaf. Runtime event producers, AsyncAPI schema, backend SSE handlers, and DTOs remain separate owners. |
| Future reopen rule | Allowed only when a concrete proto package, field number/name, enum value, or compatibility change is proposed. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Internal event envelope proto maintenance proposal | `contracts.qrpc_core.event_envelope_proto` | Updated or verified `qrpc_core/src/event_envelope.proto` schema |

The leaf may describe and guard:

- protobuf syntax and package;
- `EventEnvelope` field names and field numbers;
- `ChainStage` enum values;
- `Severity` enum values;
- `RetentionClass` enum values;
- backward compatibility rules around proto field numbering.

## Non-Claims

This closeout does not claim:

- proto schema content changed;
- AsyncAPI or OpenAPI content changed;
- runtime event producers or backend SSE handlers changed;
- Strategy IR, plugin contract, or `lib.rs` protocol contracts changed;
- compiler, runtime, backend, executor, frontend, or E2E behavior changed;
- release transition was opened.

## Next Step

BE-001PS-01 `root.contracts.qrpc_core` parent_residual_judgment selects `contracts.qrpc_core.plugin_contract`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
