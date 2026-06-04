# v4.16.0 root.contracts.qrpc_core.strategy_ir.version_unknown_error single leaf closeout

> Batch: BE-001QI-03
> Node: `root.contracts.qrpc_core.strategy_ir.version_unknown_error`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.version_unknown_error` has been evaluated after BE-001QI-02 extraction.

Decision:

`stop_split: true`

The node is now a compact child module with one responsibility: Strategy IR version identity, unknown preservation, and validation error diagnostics.

## Split Rule Evaluation

| Rule | Result |
| --- | --- |
| Physical owner | Stop. The selected surface is now isolated in `qrpc_core/src/strategy_ir/version_unknown_error.rs`. |
| Public method count | Stop. This child owns one compact helper method, `KnownOrUnknown::is_unknown`. |
| Mixed responsibility | Stop. Version identity, unknown preservation, and validation diagnostics are a small shared identity surface. |
| Parent-mediated dependency | Covered. Root validation and DTO families reach the selected surfaces through the Strategy IR parent re-export. |
| Future reopen rule | Allowed only when a concrete Strategy IR version string, unknown marker shape, `is_unknown` semantics, validation error carrier, or display/error behavior change is proposed. |

## White-Box Boundary

| Input | Processing owner | Output |
| --- | --- | --- |
| Strategy IR identity, unknown, or validation diagnostic proposal | `contracts.qrpc_core.strategy_ir.version_unknown_error` | Updated or verified version identity, unknown wrapper, and validation error behavior |

The leaf may describe and guard:

- `STRATEGY_IR_V0_VERSION`;
- `KnownOrUnknown<T>`;
- `KnownOrUnknown::is_unknown`;
- `StrategyIrValidationError`;
- `StrategyIrValidationError` Display/Error behavior.

## Non-Claims

This closeout does not claim:

- Strategy IR DTO families changed;
- root validation rules changed;
- indicator registries changed;
- gap annotations changed;
- protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition changed.

## Proof

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QJ-01 `root.contracts.qrpc_core.strategy_ir` parent_residual_judgment selects `contracts.qrpc_core.strategy_ir.metadata_source`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
