# v4.16.0 root.contracts.qrpc_core.strategy_ir.logic_position baseline plan

> Batch: BE-001QO-01
> Node: `root.contracts.qrpc_core.strategy_ir.logic_position`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.logic_position` is frozen as the Strategy IR logic/action/position sizing/rebalance DTO owner after BE-001QN-01 selection.

BE-001QO-01 does not move code. It defines the exact baseline and allowed movement for BE-001QO-02.

## Current Owner

Current physical owner:

- `qrpc_core/src/strategy_ir.rs`

Current selected boundary:

- `StrategyLogic`;
- `LogicRule`;
- `LogicAction`;
- `PositionSizing`;
- `PositionSizingMethod`;
- `PositionSizingUnit`;
- `RebalanceRule`.

Current parent callers:

- `StrategyIr` embeds `StrategyLogic`;
- `StrategyIr::validation_errors` reads entry/exit rules, position sizing value, and optional rebalance frequency;
- `validate_logic_rule` reads `LogicRule` but remains a private root validation helper;
- `validate_unknownable` reads `PositionSizing.value` and `RebalanceRule.frequency` but remains a private root validation helper;
- tests parse logic/action/position sizing/rebalance JSON through the public Strategy IR import path.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Public method count | Stop after extraction. This child owns DTO/enums only and has no public methods. |
| Mixed responsibility | Stop after extraction. Logic rules, actions, position sizing, and rebalance settings are one strategy logic contract family. |
| Physical owner | Continue now. The selected surface still lives in `qrpc_core/src/strategy_ir.rs` and should be isolated. |
| Private helper pressure | Defer. `validate_logic_rule`, `validate_unknownable`, and `validate_unknownable_opt` remain under `root_validation`. |
| Future reopen rule | Allowed only when a concrete logic field, action enum variant, position sizing field, sizing enum variant, rebalance field, serde attribute, or validation ownership proposal is made. |

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Strategy logic DTO | `StrategyLogic` keeps `entry_rules`, defaulted `exit_rules`, `position_sizing`, and `rebalance_rule`. |
| Logic rule DTO | `LogicRule` keeps `rule_id`, `condition`, and `action`. |
| Logic action enum | `LogicAction` keeps all six variants and `snake_case` serde rename behavior. |
| Position sizing DTO | `PositionSizing` keeps `method`, `value`, and `unit`; `value` continues to use `KnownOrUnknown<f64>`. |
| Position sizing enums | `PositionSizingMethod` and `PositionSizingUnit` keep all variants and `snake_case` serde rename behavior. |
| Rebalance DTO | `RebalanceRule` keeps `frequency` and `condition`; `frequency` continues to use `KnownOrUnknown<String>`. |

## Allowed BE-001QO-02 Movement

BE-001QO-02 may:

- create `qrpc_core/src/strategy_ir/logic_position.rs`;
- add a private `mod logic_position;` declaration in `qrpc_core/src/strategy_ir.rs`;
- re-export the selected child surface from the Strategy IR parent with `pub use logic_position::*;`;
- move only `StrategyLogic`, `LogicRule`, `LogicAction`, `PositionSizing`, `PositionSizingMethod`, `PositionSizingUnit`, and `RebalanceRule` into the child module;
- import `KnownOrUnknown` from the Strategy IR parent into the child module;
- keep all public imports from `qrpc_core::strategy_ir::*` and `qrpc_core::*` equivalent.

## Forbidden BE-001QO-02 Movement

BE-001QO-02 must not move or rewrite:

- `StrategyIr`, `StrategyIr::validation_errors`, or `StrategyIr::validate`;
- `validate_logic_rule`, `validate_unknownable`, `validate_unknownable_opt`, `validate_unique_ids`, `indicator_kind_supported`, or any other private validation helper;
- closed `version_unknown_error`, `metadata_source`, or `signal_indicator` children;
- risk/data/execution/gap/unknown DTOs or root validation children;
- validation rule conditions or error text;
- tests unless needed only to preserve module visibility;
- closed error/proto/plugin contract children, `qrpc_core/src/lib.rs`, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Parent-Child Rule

Allowed call paths:

- Strategy IR parent -> private `logic_position` child re-export;
- root `StrategyIr` DTO and validation -> logic/position DTOs through the Strategy IR parent-local public surface;
- `logic_position` child -> parent-provided `KnownOrUnknown` only;
- external callers -> logic/position DTOs through `qrpc_core::strategy_ir::*` or `qrpc_core::*`.

Forbidden call paths:

Any logic/position child import from future Strategy IR sibling modules, qrpc runtime/compiler modules, backend, executor, or release-transition paths that bypasses the Strategy IR parent.

## Proof

BE-001QO-02 must prove equivalence with:

- `cargo test -p qrpc-core`
- `cargo check -p quantpilot`

## Next Step

BE-001QO-02 `root.contracts.qrpc_core.strategy_ir.logic_position` extract_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
