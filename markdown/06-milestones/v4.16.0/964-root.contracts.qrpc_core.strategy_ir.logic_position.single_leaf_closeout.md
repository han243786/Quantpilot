# v4.16.0 root.contracts.qrpc_core.strategy_ir.logic_position single leaf closeout

> Batch: BE-001QO-03
> Node: `root.contracts.qrpc_core.strategy_ir.logic_position`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.logic_position` has been evaluated after BE-001QO-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: Strategy IR logic rules, actions, position sizing, and rebalance DTO shape.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is now isolated in `qrpc_core/src/strategy_ir/logic_position.rs`. |
| Public method count | Stop. The child owns DTO/enums only and has no public methods that require a separate method node. |
| Mixed responsibility | Stop. Logic rules, action taxonomy, position sizing, and rebalance settings are one strategy logic contract family. |
| Parent-mediated dependency | Covered. The child receives `KnownOrUnknown` through the Strategy IR parent, and root validation reaches logic DTOs through the parent re-export. |
| Future reopen rule | Allowed only when a concrete logic field, action enum variant, position sizing field, sizing enum variant, rebalance field, serde attribute, or validation ownership proposal is made. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Strategy IR logic/action/position sizing/rebalance DTO proposal | `contracts.qrpc_core.strategy_ir.logic_position` | Updated or verified logic/action/position DTO shape |

The leaf may describe and guard:

- `StrategyLogic`;
- `LogicRule`;
- `LogicAction`;
- `PositionSizing`;
- `PositionSizingMethod`;
- `PositionSizingUnit`;
- `RebalanceRule`.

## Non-Claims

This closeout does not claim:

- root validation helper behavior changed;
- validation rule conditions or error text changed;
- signal/indicator registry behavior changed;
- risk/data/execution/gap DTOs changed;
- protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QP-01 `root.contracts.qrpc_core.strategy_ir` parent_residual_judgment selects `contracts.qrpc_core.strategy_ir.risk_contract`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
