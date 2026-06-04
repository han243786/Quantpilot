# v4.16.0 root.contracts.qrpc_core.strategy_ir.risk_contract single leaf closeout

> Batch: BE-001QQ-03
> Node: `root.contracts.qrpc_core.strategy_ir.risk_contract`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.risk_contract` has been evaluated after BE-001QQ-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: Strategy IR risk rule and risk profile DTO shape.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is now isolated in `qrpc_core/src/strategy_ir/risk_contract.rs`. |
| Public method count | Stop. The child owns DTOs only and has no public methods that require a separate method node. |
| Mixed responsibility | Stop. Risk limits and risk profile reference fields are one risk contract family. |
| Parent-mediated dependency | Covered. The child receives `KnownOrUnknown` through the Strategy IR parent, and root validation reaches risk DTOs through the parent re-export. |
| Future reopen rule | Allowed only when a concrete risk rule field, risk profile field, serde/default rule, unknownable risk value type, or validation ownership proposal is made. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Strategy IR risk rule/profile DTO proposal | `contracts.qrpc_core.strategy_ir.risk_contract` | Updated or verified risk rule/profile DTO shape |

The leaf may describe and guard:

- `StrategyRiskRules`;
- `StrategyRiskProfileRef`.

## Non-Claims

This closeout does not claim:

- root validation helper behavior changed;
- validation rule conditions or error text changed;
- logic/signal/data/execution/gap DTOs changed;
- protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QR-01 `root.contracts.qrpc_core.strategy_ir` parent_residual_judgment selects `contracts.qrpc_core.strategy_ir.data_requirement`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
