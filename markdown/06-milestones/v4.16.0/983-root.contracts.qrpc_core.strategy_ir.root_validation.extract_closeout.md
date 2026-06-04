# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation extract closeout

> Batch: BE-001QY-02
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Parent: `root.contracts.qrpc_core.strategy_ir`
> Stage: `extract_closeout`
> Movement: Actual Rust extraction.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.root_validation` has been extracted from the Strategy IR parent into a private child module.

Created:

- `qrpc_core/src/strategy_ir/root_validation.rs`

Updated:

- `qrpc_core/src/strategy_ir.rs`

## Extracted Surface

The new child module owns exactly:

- `StrategyIr`;
- `StrategyIr::validation_errors`;
- `StrategyIr::validate`;
- `validate_unique_ids`;
- `validate_logic_rule`;
- `indicator_kind_supported`;
- `validate_unknownable`;
- `validate_unknownable_opt`;
- the local Strategy IR validation tests and `SAMPLE_JSON` fixture.

## Parent Facade

`qrpc_core/src/strategy_ir.rs` now declares:

- `mod root_validation;`
- `pub use root_validation::*;`

The public import paths remain equivalent:

- `qrpc_core::strategy_ir::*`
- `qrpc_core::*`

## Preserved Boundaries

This extraction did not move or rewrite:

- any closed Strategy IR child DTO, enum, version constant, error type, or public indicator registry function;
- validation rule ordering, rule conditions, diagnostics, serde shape, or test assertions;
- `qrpc_core/src/lib.rs`, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Equivalence Notes

`root_validation.rs` imports closed Strategy IR child surfaces through the Strategy IR parent module. Closed children still do not import each other directly.

The parent `qrpc_core/src/strategy_ir.rs` is now a facade over Strategy IR child modules.

## Proof

Required proof for this batch:

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001QY-03 `root.contracts.qrpc_core.strategy_ir.root_validation` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
