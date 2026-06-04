# v4.16.0 root.contracts.qrpc_core.strategy_ir parent residual judgment selects signal_indicator

> Batch: BE-001QL-01
> Node: `root.contracts.qrpc_core.strategy_ir`
> Selected child: `root.contracts.qrpc_core.strategy_ir.signal_indicator`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir` returns to its remaining child queue after `metadata_source` closeout.

Decision:

`next_child: root.contracts.qrpc_core.strategy_ir.signal_indicator`

## Closed Strategy_IR Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.version_unknown_error` | Closed with `stop_split: true`; owns version identity, unknown preservation, and validation diagnostics. |
| `contracts.qrpc_core.strategy_ir.metadata_source` | Closed with `stop_split: true`; owns metadata/source DTO shape. |

## Open Strategy_IR Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.signal_indicator` | Selected next. Owns signal/indicator DTOs, indicator enum, and public indicator registry surfaces. |
| `contracts.qrpc_core.strategy_ir.logic_position` | Queued. Owns logic, action, position sizing, and rebalance DTOs. |
| `contracts.qrpc_core.strategy_ir.risk_contract` | Queued. Owns risk rule and risk profile DTOs. |
| `contracts.qrpc_core.strategy_ir.data_requirement` | Queued. Owns data requirement DTOs. |
| `contracts.qrpc_core.strategy_ir.execution_contract` | Queued. Owns execution DTOs. |
| `contracts.qrpc_core.strategy_ir.gap_unknown_annotation` | Queued. Owns gap annotation and unknown marker DTOs. |
| `contracts.qrpc_core.strategy_ir.root_validation` | Queued. Owns root `StrategyIr` DTO, validation behavior, private helpers, and local tests. |

## Selection Rationale

`contracts.qrpc_core.strategy_ir.signal_indicator` is selected because it is the next independent Strategy IR DTO and registry family:

- physical region: `qrpc_core/src/strategy_ir.rs`;
- public DTOs: `SignalDefinition` and `IndicatorDefinition`;
- public enum: `IndicatorKind`;
- public registry functions: `declared_indicator_kinds` and `supported_indicator_kinds`;
- owned constants: `DECLARED_INDICATOR_KINDS` and `SUPPORTED_INDICATOR_KINDS`;
- no ownership of root validation rules, private validation helpers, logic/risk/data/execution/gap DTOs, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Hard Boundaries

The next `root.contracts.qrpc_core.strategy_ir.signal_indicator` baseline must not:

- edit Rust source code;
- change signal/indicator fields, serde attributes, enum variants, enum rename rules, declared/supported indicator ordering, or public registry return values;
- change root validation rule conditions, validation error text, private validation helper behavior, closed `version_unknown_error` or `metadata_source` children;
- change logic/risk/data/execution/gap DTOs;
- change closed error/proto/plugin contract children;
- change `qrpc_core/src/lib.rs`, `qrpc_core_ir`, `qrpc_compiler`, `qrpc_runtime`, `quantscript`, backend, executor, frontend, physical `plugins/*`, or release transition.

## Next Step

BE-001QM-01 `root.contracts.qrpc_core.strategy_ir.signal_indicator` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
