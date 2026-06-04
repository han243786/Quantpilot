# v4.16.0 root.contracts.qrpc_core.strategy_ir parent residual judgment selects risk_contract

> Batch: BE-001QP-01
> Node: `root.contracts.qrpc_core.strategy_ir`
> Selected child: `root.contracts.qrpc_core.strategy_ir.risk_contract`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir` returns to its remaining child queue after `logic_position` closeout.

Decision:

`next_child: root.contracts.qrpc_core.strategy_ir.risk_contract`

## Closed Strategy_IR Children

| Child | Result |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.version_unknown_error` | Closed with `stop_split: true`; owns version identity, unknown preservation, and validation diagnostics. |
| `contracts.qrpc_core.strategy_ir.metadata_source` | Closed with `stop_split: true`; owns metadata/source DTO shape. |
| `contracts.qrpc_core.strategy_ir.signal_indicator` | Closed with `stop_split: true`; owns signal/indicator DTOs, taxonomy, and public indicator registry surfaces. |
| `contracts.qrpc_core.strategy_ir.logic_position` | Closed with `stop_split: true`; owns logic/action/position sizing/rebalance DTO shape. |

## Open Strategy_IR Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.risk_contract` | Selected next. Owns risk rule and risk profile DTOs. |
| `contracts.qrpc_core.strategy_ir.data_requirement` | Queued. Owns data requirement DTOs. |
| `contracts.qrpc_core.strategy_ir.execution_contract` | Queued. Owns execution DTOs. |
| `contracts.qrpc_core.strategy_ir.gap_unknown_annotation` | Queued. Owns gap annotation and unknown marker DTOs. |
| `contracts.qrpc_core.strategy_ir.root_validation` | Queued. Owns root `StrategyIr` DTO, validation behavior, private helpers, and local tests. |

## Selection Rationale

`contracts.qrpc_core.strategy_ir.risk_contract` is selected because it is the next independent Strategy IR DTO family:

- physical region: `qrpc_core/src/strategy_ir.rs`;
- public DTOs: `StrategyRiskRules` and `StrategyRiskProfileRef`;
- risk unknownable fields remain expressed through the parent-provided `KnownOrUnknown`;
- no ownership of root validation rules, private validation helpers, logic/signal/data/execution/gap DTOs, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, or release transition.

## Hard Boundaries

The next `root.contracts.qrpc_core.strategy_ir.risk_contract` baseline must not:

- edit Rust source code;
- change risk rule fields, risk profile fields, serde attributes, optional/default behavior, or unknownable value types;
- change root validation rule conditions, validation error text, private validation helper behavior, or closed `version_unknown_error`, `metadata_source`, `signal_indicator`, and `logic_position` children;
- change data/execution/gap DTOs;
- change closed error/proto/plugin contract children;
- change `qrpc_core/src/lib.rs`, `qrpc_core_ir`, `qrpc_compiler`, `qrpc_runtime`, `quantscript`, backend, executor, frontend, physical `plugins/*`, or release transition.

## Next Step

BE-001QQ-01 `root.contracts.qrpc_core.strategy_ir.risk_contract` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
