# v4.16.0 root.contracts.qrpc_core.strategy_ir baseline plan

> Batch: BE-001QG-01
> Node: `root.contracts.qrpc_core.strategy_ir`
> Parent: `root.contracts.qrpc_core`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir` is frozen as the Strategy IR contract owner after BE-001QF-01 selection.

Decision:

`baseline_frozen: true`

Next candidate:

`root.contracts.qrpc_core.strategy_ir.version_unknown_error`

## Current Owner

Current physical owner:

- `qrpc_core/src/strategy_ir.rs`

Current selected boundary:

- Strategy IR version constant;
- `KnownOrUnknown<T>` unknown-preservation wrapper;
- `StrategyIr` root DTO and validation behavior;
- `StrategyIrValidationError`;
- metadata and source DTOs;
- signal and indicator DTOs plus indicator kind registries;
- logic, action, position sizing, and rebalance DTOs;
- risk rule and risk profile DTOs;
- data requirement DTOs;
- execution DTOs;
- gap annotation and unknown marker DTOs;
- private validation helpers and local tests.

## Key Public Surfaces To Track

| Surface | Public contract |
| --- | --- |
| Identity, unknown, and error | `STRATEGY_IR_V0_VERSION`, `KnownOrUnknown<T>`, `KnownOrUnknown::is_unknown`, `StrategyIrValidationError`. |
| Root validation | `StrategyIr`, `StrategyIr::validation_errors`, `StrategyIr::validate`. |
| Metadata/source | `StrategyMetadata`, `StrategySource`, `StrategySourceType`. |
| Signal/indicator | `SignalDefinition`, `IndicatorDefinition`, `IndicatorKind`, `declared_indicator_kinds`, `supported_indicator_kinds`. |
| Logic/position | `StrategyLogic`, `LogicRule`, `LogicAction`, `PositionSizing`, `PositionSizingMethod`, `PositionSizingUnit`, `RebalanceRule`. |
| Risk | `StrategyRiskRules`, `StrategyRiskProfileRef`. |
| Data requirement | `DataRequirement`, `DataRequirementType`. |
| Execution | `StrategyExecution`, `StrategyExecutionProfileRef`. |
| Gap and unknown annotations | `GapAnnotation`, `GapType`, `GapSeverity`, `StrategyUnknown`. |

## Recursive Child Queue

| Order | Child | Stage to enter | Split note |
| --- | --- | --- | --- |
| 1 | `root.contracts.qrpc_core.strategy_ir.version_unknown_error` | `baseline_plan` | Compact identity and diagnostic surface, but it still needs physical isolation before closeout. |
| 2 | `root.contracts.qrpc_core.strategy_ir.metadata_source` | `baseline_plan` | Metadata/source DTO group; likely extractable. |
| 3 | `root.contracts.qrpc_core.strategy_ir.signal_indicator` | `baseline_plan` | Signal DTOs, indicator enum, and indicator registries. |
| 4 | `root.contracts.qrpc_core.strategy_ir.logic_position` | `baseline_plan` | Logic rules, actions, position sizing, and rebalance DTOs. |
| 5 | `root.contracts.qrpc_core.strategy_ir.risk_contract` | `baseline_plan` | Risk rule and profile references. |
| 6 | `root.contracts.qrpc_core.strategy_ir.data_requirement` | `baseline_plan` | Data requirement DTOs. |
| 7 | `root.contracts.qrpc_core.strategy_ir.execution_contract` | `baseline_plan` | Execution DTOs and execution profile reference. |
| 8 | `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` | `baseline_plan` | Gap annotations and unknown marker DTOs. |
| 9 | `root.contracts.qrpc_core.strategy_ir.root_validation` | `baseline_plan` | Root `StrategyIr` DTO, validation behavior, private validation helpers, and local tests. |

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Public method count | Continue split. The file owns multiple public behavior methods and registry functions. |
| Mixed responsibility | Continue split. DTO families, indicator registries, unknown preservation, validation, and diagnostics are independent white-box owners. |
| Physical owner | Continue split. A single file currently contains several Strategy IR child owners. |
| Parent-child communication | Hard rule. Children must communicate through the `strategy_ir` parent facade and public re-exports, not through sibling file paths. |
| Release transition | Closed. No sibling shortcut or performance connection may be proposed without an explicit developer release-transition decision. |

## Allowed Future Movement

Future extraction steps may:

- introduce private child modules under `qrpc_core/src/strategy_ir/` or an equivalent Strategy IR module layout;
- keep `qrpc_core/src/strategy_ir.rs` or `qrpc_core/src/strategy_ir/mod.rs` as the parent facade;
- move one selected child owner at a time while preserving all public re-exports from `qrpc_core::strategy_ir::*` and `qrpc_core::*`;
- move or colocate tests only when the selected child owns the tested behavior;
- preserve every serde shape, version string, validation condition, validation error string, indicator kind list, and public method signature.

## Forbidden Movement

This baseline and its immediate child selection must not:

- edit Rust source code;
- change Strategy IR fields, serde attributes, version strings, validation rule conditions, validation error text, known/unknown preservation, indicator kind lists, or tests;
- change closed error/proto/plugin contract children;
- change `qrpc_core/src/lib.rs` protocol primitives, runtime config, artifact specs, runtime IO, or RFC execution contracts;
- change `qrpc_core_ir`, `qrpc_compiler`, `qrpc_runtime`, `quantscript`, backend, executor, frontend, physical `plugins/*`, or release transition;
- create direct child-to-child imports that bypass the Strategy IR parent.

## Equivalence Evidence

No Rust source is changed in this batch. Equivalence is proven by unchanged source files plus the standard gates:

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`

## Next Step

BE-001QH-01 `root.contracts.qrpc_core.strategy_ir` parent_residual_judgment selects `contracts.qrpc_core.strategy_ir.version_unknown_error`.
