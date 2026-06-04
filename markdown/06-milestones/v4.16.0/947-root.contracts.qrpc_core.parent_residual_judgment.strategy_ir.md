# v4.16.0 root.contracts.qrpc_core parent residual judgment selects strategy_ir

> Batch: BE-001QF-01
> Node: `root.contracts.qrpc_core`
> Selected child: `root.contracts.qrpc_core.strategy_ir`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core` returns to its remaining child queue after `plugin_contract` parent closeout.

Decision:

`next_child: root.contracts.qrpc_core.strategy_ir`

## Closed Qrpc_Core Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.error_contract` | Closed with `stop_split: true`; owns typed core error behavior. |
| `contracts.qrpc_core.event_envelope_proto` | Closed with `stop_split: true`; owns the event envelope protobuf schema. |
| `contracts.qrpc_core.plugin_contract` | Closed with `close_parent: true`; owns the plugin contract parent facade and five closed child leaves. |

## Open Qrpc_Core Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.strategy_ir` | Selected next. Owns Strategy IR DTOs, validation behavior, known/unknown preservation, indicator kind surfaces, and gap annotations. |
| `contracts.qrpc_core.protocol_primitives` | Queued. Owns protocol version constants and primitive enums/defaults in `qrpc_core/src/lib.rs`. |
| `contracts.qrpc_core.runtime_protocol_config` | Queued. Owns runtime protocol config structs and defaults in `qrpc_core/src/lib.rs`. |
| `contracts.qrpc_core.artifact_specs` | Queued. Owns canonical digest and artifact specs in `qrpc_core/src/lib.rs`. |
| `contracts.qrpc_core.runtime_io_contract` | Queued. Owns runtime IO DTOs and output contracts in `qrpc_core/src/lib.rs`. |
| `contracts.qrpc_core.rfc_execution_contracts` | Queued. Owns RFC-style execution, allocation, order, feedback, and handoff contracts in `qrpc_core/src/lib.rs`. |

## Selection Rationale

`contracts.qrpc_core.strategy_ir` is selected because it is the next independent qrpc_core contract owner after the closed plugin contract:

- physical owner: `qrpc_core/src/strategy_ir.rs`;
- public version constant: `STRATEGY_IR_V0_VERSION`;
- public preservation wrapper: `KnownOrUnknown<T>` and `KnownOrUnknown::is_unknown`;
- public root DTO and validation: `StrategyIr`, `StrategyIr::validation_errors`, `StrategyIr::validate`, and `StrategyIrValidationError`;
- public indicator surfaces: `IndicatorKind`, `declared_indicator_kinds`, `supported_indicator_kinds`;
- public DTO families: metadata/source/signal/logic/risk/data/execution/gap/unknown structures;
- no ownership of plugin contract, protocol primitives in `lib.rs`, runtime protocol config, artifact specs, runtime IO, RFC execution contracts, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Hard Boundaries

The next `root.contracts.qrpc_core.strategy_ir` baseline must not:

- edit Rust source code;
- change Strategy IR fields, serde attributes, version strings, validation rule conditions, validation error text, known/unknown preservation, indicator kind lists, or tests;
- change closed error/proto/plugin contract children;
- change `qrpc_core/src/lib.rs` protocol primitives, runtime config, artifact specs, runtime IO, or RFC execution contracts;
- change `qrpc_core_ir`, `qrpc_compiler`, `qrpc_runtime`, `quantscript`, backend, executor, frontend, physical `plugins/*`, or release transition.

## Next Step

BE-001QG-01 `root.contracts.qrpc_core.strategy_ir` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
