# v4.16.0 root.contracts.qrpc_core.strategy_ir parent residual judgment selects gap_unknown_annotation

> Batch: BE-001QV-01
> Node: `root.contracts.qrpc_core.strategy_ir`
> Selected child: `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir` returns to its remaining child queue after `execution_contract` closeout.

Decision:

`next_child: root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation`

## Closed Strategy_IR Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.version_unknown_error` | Closed with `stop_split: true`; owns version identity, unknown preservation, and validation diagnostics. |
| `contracts.qrpc_core.strategy_ir.metadata_source` | Closed with `stop_split: true`; owns metadata/source DTO shape. |
| `contracts.qrpc_core.strategy_ir.signal_indicator` | Closed with `stop_split: true`; owns signal/indicator DTOs, taxonomy, and public indicator registry surfaces. |
| `contracts.qrpc_core.strategy_ir.logic_position` | Closed with `stop_split: true`; owns logic/action/position sizing/rebalance DTO shape. |
| `contracts.qrpc_core.strategy_ir.risk_contract` | Closed with `stop_split: true`; owns risk rule/profile DTO shape. |
| `contracts.qrpc_core.strategy_ir.data_requirement` | Closed with `stop_split: true`; owns data requirement DTO shape and data type taxonomy. |
| `contracts.qrpc_core.strategy_ir.execution_contract` | Closed with `stop_split: true`; owns execution DTO and execution profile reference shape. |

## Open Strategy_IR Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.gap_unknown_annotation` | Selected next. Owns gap annotation DTOs and strategy unknown marker DTOs. |
| `contracts.qrpc_core.strategy_ir.root_validation` | Queued. Owns root `StrategyIr` DTO, validation behavior, private helpers, and local tests. |

## Selection Rationale

`contracts.qrpc_core.strategy_ir.gap_unknown_annotation` is selected because it is the final non-root Strategy IR DTO family:

- physical region: `qrpc_core/src/strategy_ir.rs`;
- public DTOs: `GapAnnotation` and `StrategyUnknown`;
- public enums: `GapType` and `GapSeverity`;
- no ownership of root validation rules, private validation helpers, logic/signal/risk/data/execution DTOs, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Hard Boundaries

The next `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` baseline must not:

- edit Rust source code;
- change gap annotation fields, unknown marker fields, serde attributes, enum variants, or enum rename rules;
- change root validation rule conditions, validation error text, private validation helper behavior, or any closed Strategy IR child;
- change `StrategyIr`, validation tests, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Next Step

BE-001QW-01 `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
