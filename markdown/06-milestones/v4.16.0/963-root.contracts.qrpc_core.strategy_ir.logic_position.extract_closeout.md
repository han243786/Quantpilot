# v4.16.0 root.contracts.qrpc_core.strategy_ir.logic_position extract closeout

> Batch: BE-001QO-02
> Node: `root.contracts.qrpc_core.strategy_ir.logic_position`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `extract_closeout`
> Movement: Actual Rust extraction.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.logic_position` has been extracted from the Strategy IR parent into a private child module.

Created:

- `qrpc_core/src/strategy_ir/logic_position.rs`

Updated:

- `qrpc_core/src/strategy_ir.rs`

## Extracted Surface

The new child module owns exactly:

- `StrategyLogic`;
- `LogicRule`;
- `LogicAction`;
- `PositionSizing`;
- `PositionSizingMethod`;
- `PositionSizingUnit`;
- `RebalanceRule`.

## Parent Facade

`qrpc_core/src/strategy_ir.rs` now declares:

- `mod logic_position;`
- `pub use logic_position::*;`

The public import paths remain equivalent:

- `qrpc_core::strategy_ir::*`
- `qrpc_core::*`

## Preserved Boundaries

This extraction did not move or rewrite:

- `StrategyIr`, `StrategyIr::validation_errors`, or `StrategyIr::validate`;
- `validate_logic_rule`, `validate_unknownable`, `validate_unknownable_opt`, `validate_unique_ids`, or `indicator_kind_supported`;
- closed `version_unknown_error`, `metadata_source`, or `signal_indicator` children;
- risk/data/execution/gap/unknown DTOs;
- validation rule conditions or error text;
- local tests, `qrpc_core/src/lib.rs`, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Equivalence Notes

`logic_position.rs` imports `KnownOrUnknown` through the Strategy IR parent boundary. This keeps the child dependent on the parent-provided contract surface instead of reaching into another sibling module.

## Proof

Required proof for this batch:

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QO-03 `root.contracts.qrpc_core.strategy_ir.logic_position` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
