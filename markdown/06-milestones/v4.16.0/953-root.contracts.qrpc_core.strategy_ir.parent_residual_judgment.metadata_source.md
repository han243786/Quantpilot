# v4.16.0 root.contracts.qrpc_core.strategy_ir parent residual judgment selects metadata_source

> Batch: BE-001QJ-01
> Node: `root.contracts.qrpc_core.strategy_ir`
> Selected child: `root.contracts.qrpc_core.strategy_ir.metadata_source`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir` returns to its remaining child queue after `version_unknown_error` closeout.

Decision:

`next_child: root.contracts.qrpc_core.strategy_ir.metadata_source`

## Closed Strategy_IR Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.version_unknown_error` | Closed with `stop_split: true`; owns version identity, unknown preservation, and validation diagnostics. |

## Open Strategy_IR Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.metadata_source` | Selected next. Owns strategy metadata and source DTO shape. |
| `contracts.qrpc_core.strategy_ir.signal_indicator` | Queued. Owns signal, indicator, and indicator registry surfaces. |
| `contracts.qrpc_core.strategy_ir.logic_position` | Queued. Owns logic, action, position sizing, and rebalance DTOs. |
| `contracts.qrpc_core.strategy_ir.risk_contract` | Queued. Owns risk rule and risk profile DTOs. |
| `contracts.qrpc_core.strategy_ir.data_requirement` | Queued. Owns data requirement DTOs. |
| `contracts.qrpc_core.strategy_ir.execution_contract` | Queued. Owns execution DTOs. |
| `contracts.qrpc_core.strategy_ir.gap_unknown_annotation` | Queued. Owns gap annotation and unknown marker DTOs. |
| `contracts.qrpc_core.strategy_ir.root_validation` | Queued. Owns root `StrategyIr` DTO, validation behavior, private helpers, and local tests. |

## Selection Rationale

`contracts.qrpc_core.strategy_ir.metadata_source` is selected because it is the next smallest Strategy IR DTO family:

- physical region: `qrpc_core/src/strategy_ir.rs`;
- public DTOs: `StrategyMetadata`, `StrategySource`, and `StrategySourceType`;
- public serde shape: metadata/source fields, defaulted authors/tags, and snake_case source type enum names;
- no ownership of root validation rules, signal/indicator DTOs, logic/risk/data/execution/gap DTOs, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Hard Boundaries

The next `root.contracts.qrpc_core.strategy_ir.metadata_source` baseline must not:

- edit Rust source code;
- change metadata/source fields, serde attributes, enum variants, enum rename rules, field ordering, defaults, or tests;
- change Strategy IR validation rules, indicator lists, gap annotations, root `StrategyIr` behavior, or closed `version_unknown_error` child;
- change closed error/proto/plugin contract children;
- change `qrpc_core/src/lib.rs`, `qrpc_core_ir`, `qrpc_compiler`, `qrpc_runtime`, `quantscript`, backend, executor, frontend, physical `plugins/*`, or release transition.

## Next Step

BE-001QK-01 `root.contracts.qrpc_core.strategy_ir.metadata_source` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
