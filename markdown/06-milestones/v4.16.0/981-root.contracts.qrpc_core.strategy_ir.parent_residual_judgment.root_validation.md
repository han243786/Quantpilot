# v4.16.0 root.contracts.qrpc_core.strategy_ir parent residual judgment selects root_validation

> Batch: BE-001QX-01
> Node: `root.contracts.qrpc_core.strategy_ir`
> Selected child: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir` returns to its remaining child queue after `gap_unknown_annotation` closeout.

Decision:

`next_child: root.contracts.qrpc_core.strategy_ir.root_validation`

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
| `contracts.qrpc_core.strategy_ir.gap_unknown_annotation` | Closed with `stop_split: true`; owns gap annotation DTOs, gap taxonomy enums, and strategy unknown marker DTOs. |

## Open Strategy_IR Residuals

| Residual | Status |
| --- | --- |
| `contracts.qrpc_core.strategy_ir.root_validation` | Selected next. Owns root `StrategyIr` DTO, validation behavior, private helpers, and local tests. |

## Selection Rationale

`contracts.qrpc_core.strategy_ir.root_validation` is selected because it is the final Strategy IR residual:

- physical region: remaining root surface in `qrpc_core/src/strategy_ir.rs`;
- public DTO: `StrategyIr`;
- public methods: `StrategyIr::validation_errors` and `StrategyIr::validate`;
- private helpers: `validate_unique_ids`, `validate_logic_rule`, `indicator_kind_supported`, `validate_unknownable`, and `validate_unknownable_opt`;
- local tests that prove parse/validation, unknown marker rejection, duplicate id rejection, execution profile finite-cost rejection, and declared custom indicator support.

## Hard Boundaries

The next `root.contracts.qrpc_core.strategy_ir.root_validation` baseline must not:

- edit Rust source code;
- change `StrategyIr` fields, serde attributes, validation rule conditions, validation error text, helper behavior, or test expectations;
- change closed Strategy IR child DTOs, enums, constants, or public registry functions;
- create sibling-to-sibling imports between closed Strategy IR children;
- change protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Next Step

BE-001QY-01 `root.contracts.qrpc_core.strategy_ir.root_validation` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
