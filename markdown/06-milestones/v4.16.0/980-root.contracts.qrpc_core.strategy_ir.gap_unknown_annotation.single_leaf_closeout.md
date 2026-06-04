# v4.16.0 root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation single leaf closeout

> Batch: BE-001QW-03
> Node: `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` has been evaluated after BE-001QW-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: Strategy IR gap annotation DTOs, gap taxonomy enums, and strategy unknown marker DTOs.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is now isolated in `qrpc_core/src/strategy_ir/gap_unknown_annotation.rs`. |
| Public method count | Stop. The child owns DTOs/enums only and has no public methods that require a separate method node. |
| Mixed responsibility | Stop. Gap annotations and strategy unknown markers are one missing-knowledge annotation family. |
| Parent-mediated dependency | Covered. Root validation reaches `StrategyUnknown` and gap annotations through the Strategy IR parent re-export; the child has no sibling imports. |
| Future reopen rule | Allowed only when a concrete gap field, gap taxonomy variant, strategy unknown marker field, serde rule, or validation ownership proposal is made. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Strategy IR gap or unknown-marker DTO proposal | `contracts.qrpc_core.strategy_ir.gap_unknown_annotation` | Updated or verified gap annotation and strategy unknown marker shape |

The leaf may describe and guard:

- `GapAnnotation`;
- `GapType`;
- `GapSeverity`;
- `StrategyUnknown`.

## Non-Claims

This closeout does not claim:

- root validation helper behavior changed;
- validation rule conditions or error text changed;
- root `StrategyIr` changed;
- version/metadata/signal/logic/risk/data/execution DTOs changed;
- protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QX-01 `root.contracts.qrpc_core.strategy_ir` parent_residual_judgment selects `contracts.qrpc_core.strategy_ir.root_validation`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
