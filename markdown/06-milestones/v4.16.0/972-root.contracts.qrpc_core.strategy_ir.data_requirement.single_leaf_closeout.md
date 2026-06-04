# v4.16.0 root.contracts.qrpc_core.strategy_ir.data_requirement single leaf closeout

> Batch: BE-001QS-03
> Node: `root.contracts.qrpc_core.strategy_ir.data_requirement`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.data_requirement` has been evaluated after BE-001QS-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: Strategy IR data requirement DTO shape and data type taxonomy.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is now isolated in `qrpc_core/src/strategy_ir/data_requirement.rs`. |
| Public method count | Stop. The child owns DTO/enums only and has no public methods that require a separate method node. |
| Mixed responsibility | Stop. Data requirement fields and data type taxonomy are one data requirement contract family. |
| Parent-mediated dependency | Covered. The child receives `KnownOrUnknown` through the Strategy IR parent, and root validation reaches data requirement DTOs through the parent re-export. |
| Future reopen rule | Allowed only when a concrete data requirement field, data requirement type enum variant, serde rule, unknownable data value type, or validation ownership proposal is made. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Strategy IR data requirement DTO proposal | `contracts.qrpc_core.strategy_ir.data_requirement` | Updated or verified data requirement DTO shape and data type taxonomy |

The leaf may describe and guard:

- `DataRequirement`;
- `DataRequirementType`.

## Non-Claims

This closeout does not claim:

- root validation helper behavior changed;
- validation rule conditions or error text changed;
- logic/signal/risk/execution/gap DTOs changed;
- protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QT-01 `root.contracts.qrpc_core.strategy_ir` parent_residual_judgment selects `contracts.qrpc_core.strategy_ir.execution_contract`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
