# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation single leaf closeout

> Batch: BE-001RC-03
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation`
> Parent: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` has been evaluated after BE-001RC-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: Strategy IR signal/detail, indicator support, logic rule, and logic unknown-marker validation before risk/data/execution validation.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is isolated in `qrpc_core/src/strategy_ir/root_validation/signal_logic_validation.rs`. |
| Public method count | Stop. The child exposes only one parent-local helper, `validate_signal_and_logic`. |
| Mixed responsibility | Stop. Signal detail validation, indicator support checks, logic rule validation, and logic unknown-marker checks are one contiguous signal/logic validation family. |
| State ownership | Stop. The child is stateless and appends diagnostics through the parent-provided `errors` vector. |
| Side effects | Stop. No IO, persistence, locks, runtime state, external calls, or public API mutation are involved. |
| Parent-mediated dependency | Covered. The child receives `StrategyIr` and parent-owned helpers through the root validation parent and does not import sibling validation children. |
| Shared helper pressure | Covered. `validate_unknownable` and `indicator_kind_supported` remain parent-owned, avoiding premature helper child extraction. |
| Future reopen rule | Allowed only when a concrete signal/detail, indicator support, logic rule, or logic unknown-marker validation proposal is made. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Root `StrategyIr` signal and logic state | `contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` | Appended validation errors for signal fields, indicator support, logic rule fields, and logic unknown markers |

The leaf may describe and guard:

- `validate_signal_and_logic`;
- private `validate_logic_rule`;
- parent-mediated use of `indicator_kind_supported`;
- parent-mediated use of `validate_unknownable`.

## Rust Local Fields

| Field | Result |
| --- | --- |
| Crate | `qrpc-core` |
| Child visibility | `pub(super)` only. |
| Public exports | No `pub use` changes. |
| Parent facade | `root_validation.rs` remains the validation coordinator. |
| Parent-owned helpers | `indicator_kind_supported` and `validate_unknownable` stay parent-owned for queued sibling validation children. |
| Sibling dependency check | Passed. The child imports no sibling validation modules. |

## Non-Claims

This closeout does not claim:

- risk validation changed;
- data requirement validation changed;
- execution validation changed;
- unknowns path/reason validation changed;
- public `StrategyIr` fields, serde shape, local validation tests, or closed Strategy IR child DTOs changed;
- protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p qrpc-core`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`

## Next Step

BE-001RD-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent_residual_judgment selects `contracts.qrpc_core.strategy_ir.root_validation.risk_validation`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p qrpc-core`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`

