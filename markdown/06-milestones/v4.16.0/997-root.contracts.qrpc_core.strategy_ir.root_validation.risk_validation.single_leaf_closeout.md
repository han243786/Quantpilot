# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation single leaf closeout

> Batch: BE-001RE-03
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation`
> Parent: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.root_validation.risk_validation` has been evaluated after BE-001RE-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: Strategy IR risk unknownable checks and risk profile id/numeric validation before data/execution validation.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is isolated in `qrpc_core/src/strategy_ir/root_validation/risk_validation.rs`. |
| Public method count | Stop. The child exposes only one parent-local helper, `validate_risk`. |
| Mixed responsibility | Stop. Risk unknownable validation and risk profile validation are one contiguous risk validation family. |
| State ownership | Stop. The child is stateless and appends diagnostics through the parent-provided `errors` vector. |
| Side effects | Stop. No IO, persistence, locks, runtime state, external calls, or public API mutation are involved. |
| Parent-mediated dependency | Covered. The child receives `StrategyIr` and parent-owned helpers through the root validation parent and does not import sibling validation children. |
| Shared helper pressure | Covered. `validate_unknownable` and `validate_unknownable_opt` remain parent-owned for queued data/execution validation and unknown marker validation. |
| Future reopen rule | Allowed only when a concrete risk unknownable or risk profile validation proposal is made. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Root `StrategyIr` risk state | `contracts.qrpc_core.strategy_ir.root_validation.risk_validation` | Appended validation errors for risk unknown markers and risk profile constraints |

The leaf may describe and guard:

- `validate_risk`;
- parent-mediated use of `validate_unknownable`;
- parent-mediated use of `validate_unknownable_opt`;
- risk profile `global`, finite, greater-than-zero, and leverage lower-bound checks.

## Rust Local Fields

| Field | Result |
| --- | --- |
| Crate | `qrpc-core` |
| Child visibility | `pub(super)` only. |
| Public exports | No `pub use` changes. |
| Parent facade | `root_validation.rs` remains the validation coordinator. |
| Parent-owned helpers | `validate_unknownable` and `validate_unknownable_opt` stay parent-owned for queued sibling validation children. |
| Sibling dependency check | Passed. The child imports no sibling validation modules. |

## Non-Claims

This closeout does not claim:

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

BE-001RF-01 `root.contracts.qrpc_core.strategy_ir.root_validation` parent_residual_judgment selects `contracts.qrpc_core.strategy_ir.root_validation.data_execution_validation`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p qrpc-core`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
