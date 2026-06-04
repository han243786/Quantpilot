# v4.16.0 root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation extract closeout

> Batch: BE-001RA-02
> Node: `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation`
> Parent: `root.contracts.qrpc_core.strategy_ir.root_validation`
> Stage: `extract_closeout`
> Movement: Actual Rust extraction.

---

## Summary

`root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` has been extracted from `root_validation.rs` into a private child module.

Created:

- `qrpc_core/src/strategy_ir/root_validation/identity_required_validation.rs`

Updated:

- `qrpc_core/src/strategy_ir/root_validation.rs`

## Extracted Surface

The new child module owns exactly:

- version check against `STRATEGY_IR_V0_VERSION`;
- metadata required-field checks;
- top-level required collection checks;
- duplicate id checks for signals, data requirements, and logic rules;
- private `validate_unique_ids`.

## Parent Facade

`qrpc_core/src/strategy_ir/root_validation.rs` now declares:

- `mod identity_required_validation;`

The public `StrategyIr::validation_errors` method calls:

- `identity_required_validation::validate_identity_and_required_fields(self, &mut errors);`

## Preserved Boundaries

This extraction did not move or rewrite:

- signal detail validation, logic rule validation, risk validation, data validation, execution validation, or unknown marker validation;
- public `StrategyIr` fields, serde attributes, `StrategyIr::validation_errors`, `StrategyIr::validate`, or local tests beyond calling the extracted helper;
- validation ordering, diagnostics, duplicate-id labels, or duplicate-id helper behavior;
- closed Strategy IR child DTOs/enums/constants/public functions, protocol primitives, runtime IO, compiler/runtime/backend/executor/frontend behavior, physical `plugins/*`, or release transition.

## Equivalence Notes

The child receives `StrategyIr` and `STRATEGY_IR_V0_VERSION` through the root validation parent. No sibling validation child imports were introduced.

## Proof

Required proof for this batch:

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`

## Next Step

BE-001RA-03 `root.contracts.qrpc_core.strategy_ir.root_validation.identity_required_validation` single_leaf_closeout.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p qrpc-core`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
