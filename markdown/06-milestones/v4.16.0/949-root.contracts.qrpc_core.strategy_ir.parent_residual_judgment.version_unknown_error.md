# v4.16.0 root.contracts.qrpc_core.strategy_ir parent residual judgment selects version_unknown_error

> Batch: BE-001QH-01
> Node: `root.contracts.qrpc_core.strategy_ir`
> Selected child: `root.contracts.qrpc_core.strategy_ir.version_unknown_error`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir` enters its child queue after BE-001QG-01 baseline.

Decision:

`next_child: root.contracts.qrpc_core.strategy_ir.version_unknown_error`

## Closed Strategy_IR Children

No `strategy_ir` children are closed yet.

## Open Strategy_IR Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.version_unknown_error` | Selected next. Owns Strategy IR version identity, unknown-preservation wrapper, and validation error diagnostic surface. |
| `contracts.qrpc_core.strategy_ir.metadata_source` | Queued. Owns metadata and source DTOs. |
| `contracts.qrpc_core.strategy_ir.signal_indicator` | Queued. Owns signal, indicator, and indicator registry surfaces. |
| `contracts.qrpc_core.strategy_ir.logic_position` | Queued. Owns logic, action, position sizing, and rebalance DTOs. |
| `contracts.qrpc_core.strategy_ir.risk_contract` | Queued. Owns risk rule and risk profile DTOs. |
| `contracts.qrpc_core.strategy_ir.data_requirement` | Queued. Owns data requirement DTOs. |
| `contracts.qrpc_core.strategy_ir.execution_contract` | Queued. Owns execution DTOs. |
| `contracts.qrpc_core.strategy_ir.gap_unknown_annotation` | Queued. Owns gap annotation and unknown marker DTOs. |
| `contracts.qrpc_core.strategy_ir.root_validation` | Queued. Owns root `StrategyIr` DTO, validation behavior, private helpers, and local tests. |

## Selection Rationale

`contracts.qrpc_core.strategy_ir.version_unknown_error` is selected first because it is the smallest Strategy IR contract surface:

- physical region: `qrpc_core/src/strategy_ir.rs`;
- public version constant: `STRATEGY_IR_V0_VERSION`;
- public wrapper: `KnownOrUnknown<T>` and `KnownOrUnknown::is_unknown`;
- public diagnostic type: `StrategyIrValidationError` plus display/error behavior;
- no ownership of root validation rules, DTO families, indicator registries, gap annotations, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Hard Boundaries

The next `root.contracts.qrpc_core.strategy_ir.version_unknown_error` baseline must not:

- edit Rust source code;
- change version string, serde shape, unknown marker semantics, error vector ordering, display formatting, or tests;
- change Strategy IR DTO families, validation rules, indicator lists, gap annotations, or root `StrategyIr` behavior;
- change closed error/proto/plugin contract children;
- change `qrpc_core/src/lib.rs`, `qrpc_core_ir`, `qrpc_compiler`, `qrpc_runtime`, `quantscript`, backend, executor, frontend, physical `plugins/*`, or release transition.

## Next Step

BE-001QI-01 `root.contracts.qrpc_core.strategy_ir.version_unknown_error` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
