# v4.16.0 root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation extract closeout

> Batch: BE-001QW-02
> Node: `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `extract_closeout`
> Movement: Actual Rust extraction.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` has been extracted from the Strategy IR parent into a private child module.

Created:

- `qrpc_core/src/strategy_ir/gap_unknown_annotation.rs`

Updated:

- `qrpc_core/src/strategy_ir.rs`

## Extracted Surface

The new child module owns exactly:

- `GapAnnotation`;
- `GapType`;
- `GapSeverity`;
- `StrategyUnknown`.

## Parent Facade

`qrpc_core/src/strategy_ir.rs` now declares:

- `mod gap_unknown_annotation;`
- `pub use gap_unknown_annotation::*;`

The public import paths remain equivalent:

- `qrpc_core::strategy_ir::*`
- `qrpc_core::*`

## Preserved Boundaries

This extraction did not move or rewrite:

- `StrategyIr`, `StrategyIr::validation_errors`, or `StrategyIr::validate`;
- `validate_unknownable`, `validate_unknownable_opt`, `validate_logic_rule`, `validate_unique_ids`, or `indicator_kind_supported`;
- any closed Strategy IR child;
- validation rule conditions or error text;
- local tests, `qrpc_core/src/lib.rs`, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Equivalence Notes

`gap_unknown_annotation.rs` owns pure serde DTO/enums and does not require sibling imports. Root validation continues to reach `StrategyUnknown` through the Strategy IR parent re-export.

## Proof

Required proof for this batch:

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QW-03 `root.contracts.qrpc_core.strategy_ir.gap_unknown_annotation` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
